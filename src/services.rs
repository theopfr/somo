use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    sync::LazyLock,
};

static SVC: LazyLock<HashMap<(u16, &'static str), String>> = LazyLock::new(load_services);

#[inline]
fn normalize_proto(proto: &str) -> &'static str {
    if proto.eq_ignore_ascii_case("udp") {
        "udp"
    } else {
        "tcp"
    }
}

fn load_services() -> HashMap<(u16, &'static str), String> {
    let mut map = HashMap::new();
    let candidates = ["/etc/services"];
    for p in candidates {
        if Path::new(p).exists() {
            if let Ok(f) = File::open(p) {
                let r = BufReader::new(f);
                for line in r.lines().map_while(Result::ok) {
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
                            let proto = normalize_proto(proto);
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
    let key = (port, normalize_proto(proto));
    SVC.get(&key).cloned()
}

#[cfg(unix)]
fn svc_from_libc(port: u16, proto: &str) -> Option<String> {
    use libc::{endservent, getservbyport, setservent};
    use std::{
        ffi::{CStr, CString},
        ptr,
    };

    // keep DB open during lookup; close after
    unsafe { setservent(1) }

    let be = (port as i32).to_be();
    let proto_c = CString::new(normalize_proto(proto)).ok();

    let se_ptr = unsafe {
        // try with proto first
        let with_proto = proto_c
            .as_ref()
            .map(|c| getservbyport(be, c.as_ptr()))
            .unwrap_or(ptr::null_mut());
        if !with_proto.is_null() {
            with_proto
        } else {
            // fallback: no proto
            getservbyport(be, ptr::null())
        }
    };

    let out = if se_ptr.is_null() {
        None
    } else {
        // SAFETY: se_ptr validated non-null
        Some(
            unsafe { CStr::from_ptr((*se_ptr).s_name) }
                .to_string_lossy()
                .into_owned(),
        )
    };

    unsafe { endservent() }
    out
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

pub fn get_port_annotation(port_str: &str, proto: &str) -> Option<String> {
    let Ok(port) = port_str.parse::<u16>() else {
        return None;
    };
    let mut tags = Vec::new();
    if let Some(s) = service_name(port, proto) {
        tags.push(s);
    }
    if is_ephemeral(port) {
        tags.push("ephemeral".to_string());
    }
    if tags.is_empty() {
        None
    } else {
        Some(tags.join(", "))
    }
}

pub fn format_remote_port(port_str: &str, proto: &str, annotate: bool) -> String {
    if !annotate {
        return port_str.to_string();
    }
    let Ok(port) = port_str.parse::<u16>() else {
        return port_str.to_string();
    };
    if is_ephemeral(port) {
        return format!("{port_str} (ephemeral)");
    }
    if let Some(s) = service_name(port, proto) {
        format!("{port_str} ({s})")
    } else {
        port_str.to_string()
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
            "59345 (ephemeral)",
        );
    }

    #[test]
    fn ephemeral_overrides_service() {
        assert_eq!(
            format_remote_port("65535", "tcp", true),
            "65535 (ephemeral)",
        );
    }

    #[test]
    fn annotates_service_name() {
        assert_eq!(format_remote_port("443", "tcp", true), "443 (https)");
    }
    #[test]
    fn non_numeric_port_stays_as_is() {
        assert_eq!(format_remote_port("-", "tcp", true), "-");
    }
}
