use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

static SVC: Lazy<HashMap<(u16, &'static str), String>> = Lazy::new(|| load_services());

fn load_services() -> HashMap<(u16, &'static str), String> {
    let mut map = HashMap::new();
    let candidates = ["/etc/services"];
    for p in candidates {
        if Path::new(p).exists() {
            if let Ok(f) = File::open(p) {
                let r = BufReader::new(f);
                for line in r.lines().flatten() {
                    let s = line.trim();
                    if s.is_empty() || s.starts_with('#') {
                        continue;
                    }
                    let mut it = s.split_whitespace();
                    let name = match it.next() {
                        Some(x) => x,
                        None => continue,
                    };
                    let port_proto = match it.next() {
                        Some(x) => x,
                        None => continue,
                    };
                    if let Some((port_s, proto)) = port_proto.split_once('/') {
                        if let Ok(port) = port_s.parse::<u16>() {
                            let proto = if proto.eq_ignore_ascii_case("udp") {
                                "udp"
                            } else {
                                "tcp"
                            };
                            map.entry((port, proto)).or_insert_with(|| name.to_string());
                        }
                    }
                }
            }
        }
    }
    map
}

#[inline]
fn svc_from_file(port: u16, proto: &str) -> Option<String> {
    let key = (
        port,
        if proto.eq_ignore_ascii_case("udp") {
            "udp"
        } else {
            "tcp"
        },
    );
    SVC.get(&key).cloned()
}

#[cfg(unix)]
fn svc_from_libc(port: u16, proto: &str) -> Option<String> {
    use libc::{getservbyport, servent, setservent};
    use std::{
        ffi::{CStr, CString},
        ptr,
    };
    unsafe { setservent(1) }
    let be = (port as i32).to_be();
    unsafe {
        // try with proto
        if let Ok(c) = CString::new(if proto.eq_ignore_ascii_case("udp") {
            "udp"
        } else {
            "tcp"
        }) {
            let se: *mut servent = getservbyport(be, c.as_ptr());
            if !se.is_null() {
                return Some(CStr::from_ptr((*se).s_name).to_string_lossy().into_owned());
            }
        }
        // try without proto
        let se: *mut servent = getservbyport(be, ptr::null());
        if se.is_null() {
            None
        } else {
            Some(CStr::from_ptr((*se).s_name).to_string_lossy().into_owned())
        }
    }
}

#[cfg(not(unix))]
fn svc_from_libc(_port: u16, _proto: &str) -> Option<String> {
    None
}

pub fn service_name(port: u16, proto: &str) -> Option<String> {
    svc_from_file(port, proto).or_else(|| svc_from_libc(port, proto))
}

pub fn is_ephemeral(port: u16) -> bool {
    (49152..=65535).contains(&port)
}

pub fn format_remote_port(port_str: &str, proto: &str, annotate: bool) -> String {
    if !annotate {
        return port_str.to_string();
    }
    let Ok(port) = port_str.parse::<u16>() else {
        return port_str.to_string();
    };
    let mut tags = Vec::new();
    if let Some(s) = service_name(port, proto) {
        tags.push(s);
    }
    if is_ephemeral(port) {
        tags.push("ephemeral".to_string());
    }
    if tags.is_empty() {
        port_str.to_string()
    } else {
        format!("{port_str} ({})", tags.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::format_remote_port;

    #[test]
    fn annotate_disabled_passthrough() {
        assert_eq!(format_remote_port("443", "tcp", false), "443");
    }

    #[test]
    fn marks_ephemeral_range() {
        assert_eq!(
            format_remote_port("59345", "tcp", true),
            "59345 (ephemeral)"
        );
    }

    #[test]
    fn non_numeric_port_stays_as_is() {
        assert_eq!(format_remote_port("-", "tcp", true), "-");
    }
}
