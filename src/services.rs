use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    sync::LazyLock,
};

static SVC: LazyLock<HashMap<(u16, &'static str), String>> = LazyLock::new(load_services);

/// Normalizes a protocol string to either "tcp" or "udp".
#[inline]
fn normalize_proto(proto: &str) -> &'static str {
    if proto.eq_ignore_ascii_case("udp") {
        "udp"
    } else {
        "tcp"
    }
}

/// Creates a table that maps port to their service names based on the /etc/services file.
///
/// # Arguments
/// None
///
/// # Returns
/// A hashmap mapping a (port, protocol) pair to service names.
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

/// Retrieves a service name for a given (port, protocol) pair using the table generated from /etc/services.
#[inline]
fn svc_from_file(port: u16, proto: &str) -> Option<String> {
    let key = (port, normalize_proto(proto));
    SVC.get(&key).cloned()
}

/// Retrieves a service name for a given (port, protocol) pair using libc and the services-database.
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

/// Not supported for non-unix systems.
#[cfg(not(unix))]
fn svc_from_libc(_port: u16, _proto: &str) -> Option<String> {
    None
}

/// Retrieves a service name for a given (port, protocol) pair.
fn service_name(port: u16, proto: &str) -> Option<String> {
    svc_from_file(port, proto).or_else(|| svc_from_libc(port, proto))
}

/// Checks wether a port lies in the ephemeral port range.
fn is_ephemeral(port: u16) -> bool {
    (49152..=65535).contains(&port)
}

/// Retrieves a service name for a given (port, protocol) pair.
///
/// # Arguments
/// * `port`: The port
/// * `proto`: The protocol (either tcp or udp) as a string
///
/// # Returns
/// The mapped service name if it exists.
pub fn get_port_annotation(port_str: &str, proto: &str) -> Option<String> {
    let Ok(port) = port_str.parse::<u16>() else {
        return None;
    };
    if port == 0 {
        return None;
    }
    if is_ephemeral(port) {
        return Some("ephemeral".to_string());
    }
    service_name(port, proto)
}

#[cfg(test)]
mod tests {
    use super::{get_port_annotation, normalize_proto};

    #[test]
    fn non_numeric_returns_none() {
        assert_eq!(get_port_annotation("-", "tcp"), None);
    }

    #[test]
    fn port_zero_returns_none() {
        assert_eq!(get_port_annotation("0", "tcp"), None);
    }

    #[test]
    fn annotates_service_name() {
        assert_eq!(get_port_annotation("443", "tcp"), Some("https".to_string()));
    }

    #[test]
    fn annotates_service_name_invalid_proto() {
        assert_eq!(
            get_port_annotation("22", "notaproto"),
            Some("ssh".to_string())
        );
    }

    #[test]
    fn marks_ephemeral_range() {
        assert_eq!(
            get_port_annotation("59345", "tcp"),
            Some("ephemeral".to_string())
        );
    }

    #[test]
    fn out_of_ephemeral_range_returns_none() {
        assert_eq!(get_port_annotation("1000000", "tcp"), None);
    }

    #[test]
    fn normalize_uppercase_proto() {
        assert_eq!(normalize_proto("UDp"), "udp");
    }

    #[test]
    fn normalize_non_existant_proto_to_tcp() {
        assert_eq!(normalize_proto("notaproto"), "tcp");
    }
}
