use crate::schemas::{AddressType, Connection, FilterOptions};

/// Checks if a connection should be filtered out based on options provided by the user.
///
/// # Arguments
/// * `connection_details`: The connection to check for filtering.
/// * `filter_options`: The filter options provided by the user.
///
/// # Returns
/// `true` if the connection should be filtered out, `false` if not.
pub fn filter_out_connection(
    connection_details: &Connection,
    filter_options: &FilterOptions,
) -> bool {
    match &filter_options.by_remote_port {
        Some(filter_remote_port) if &connection_details.remote_port != filter_remote_port => {
            return true
        }
        _ => {}
    }
    match &filter_options.by_local_port {
        Some(filter_local_port) if &connection_details.local_port != filter_local_port => {
            return true
        }
        _ => {}
    }
    match &filter_options.by_remote_address {
        Some(filter_remote_address)
            if &connection_details.remote_address != filter_remote_address =>
        {
            return true
        }
        _ => {}
    }
    match &filter_options.by_program {
        Some(filter_program) if &connection_details.program != filter_program => return true,
        _ => {}
    }
    match &filter_options.by_pid {
        Some(filter_pid) if &connection_details.pid != filter_pid => return true,
        _ => {}
    }
    if filter_options.by_listen && connection_details.state != "listen" {
        return true;
    }
    if filter_options.by_open && connection_details.state == "close" {
        return true;
    }
    if filter_options.by_established && connection_details.state != "established" {
        return true;
    }

    false
}

/// Checks if a given IP address is either "unspecified", localhost or an extern address.
///
/// * `0.0.0.0` or `[::]` -> unspecified
/// * `127.0.0.1` or `[::1]` -> localhost
/// * else -> extern address
///
/// # Arguments
/// * `remote_address`: The address to be checked.
///
/// # Returns
/// The address-type as an AddressType enum.
pub fn get_address_type(remote_address: &str) -> AddressType {
    if remote_address == "127.0.0.1" || remote_address == "[::1]" || remote_address == "::1" {
        return AddressType::Localhost;
    } else if remote_address == "0.0.0.0" || remote_address == "[::]" || remote_address == "::" {
        return AddressType::Unspecified;
    }
    AddressType::Extern
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_get_address_type() {
        use crate::schemas::AddressType;

        assert_eq!(get_address_type("127.0.0.1"), AddressType::Localhost);
        assert_eq!(get_address_type("[::1]"), AddressType::Localhost);
        assert_eq!(get_address_type("0.0.0.0"), AddressType::Unspecified);
        assert_eq!(get_address_type("[::]"), AddressType::Unspecified);
        assert_eq!(get_address_type("8.8.8.8"), AddressType::Extern);
    }

    #[test]
    fn test_filter_out_connection_by_port() {
        use crate::schemas::{AddressType, Connection, FilterOptions};

        let conn = Connection {
            proto: "tcp".to_string(),
            local_port: "8080".to_string(),
            remote_port: "443".to_string(),
            remote_address: "8.8.8.8".to_string(),
            program: "nginx".to_string(),
            pid: "123".to_string(),
            state: "established".to_string(),
            address_type: AddressType::Extern,
            ipvx_raw: Some(Ipv4Addr::new(8, 8, 8, 8).into()),
        };

        let filter_by_matching_port = FilterOptions {
            by_local_port: Some("8080".to_string()),
            ..Default::default()
        };
        assert!(!filter_out_connection(&conn, &filter_by_matching_port));

        let filter_by_non_matching_port = FilterOptions {
            by_local_port: Some("8181".to_string()),
            ..Default::default()
        };
        assert!(filter_out_connection(&conn, &filter_by_non_matching_port));
    }

    #[test]
    fn test_filter_out_connection_by_state() {
        use crate::schemas::{AddressType, Connection, FilterOptions};

        let mut conn = Connection {
            proto: "udp".to_string(),
            local_port: "8080".to_string(),
            remote_port: "443".to_string(),
            remote_address: "8.8.8.8".to_string(),
            program: "nginx".to_string(),
            pid: "123".to_string(),
            state: "close".to_string(),
            address_type: AddressType::Extern,
            ipvx_raw: Some(Ipv4Addr::new(8, 8, 8, 8).into()),
        };

        let filter_by_open_state = FilterOptions {
            by_open: true,
            ..Default::default()
        };
        assert!(filter_out_connection(&conn, &filter_by_open_state));

        let no_active_open_filter = FilterOptions {
            by_open: false,
            ..Default::default()
        };
        assert!(!filter_out_connection(&conn, &no_active_open_filter));

        conn.state = "listen".to_string();

        let filter_by_listen_state = FilterOptions {
            by_listen: true,
            ..Default::default()
        };
        assert!(!filter_out_connection(&conn, &filter_by_listen_state));

        let no_active_listen_filter = FilterOptions {
            by_listen: false,
            ..Default::default()
        };
        assert!(!filter_out_connection(&conn, &no_active_listen_filter));
    }

    #[test]
    fn test_filter_out_connection_by_pid_and_program() {
        use crate::schemas::{AddressType, Connection, FilterOptions};

        let conn = Connection {
            proto: "tcp".to_string(),
            local_port: "8080".to_string(),
            remote_port: "443".to_string(),
            remote_address: "8.8.8.8".to_string(),
            program: "nginx".to_string(),
            pid: "123".to_string(),
            state: "close".to_string(),
            address_type: AddressType::Extern,
            ipvx_raw: Some(Ipv4Addr::new(8, 8, 8, 8).into()),
        };

        let filter_by_open_state = FilterOptions {
            by_pid: Some("123".to_string()),
            ..Default::default()
        };
        assert!(!filter_out_connection(&conn, &filter_by_open_state));

        let no_active_open_filter = FilterOptions {
            by_program: Some("postgres".to_string()),
            ..Default::default()
        };
        assert!(filter_out_connection(&conn, &no_active_open_filter));
    }

    #[test]
    fn test_filter_out_connection_by_multiple_conditions() {
        use crate::schemas::{AddressType, Connection, FilterOptions};

        let mut conn = Connection {
            proto: "tcp".to_string(),
            local_port: "8080".to_string(),
            remote_port: "443".to_string(),
            remote_address: "8.8.8.8".to_string(),
            program: "python".to_string(),
            pid: "123".to_string(),
            state: "listen".to_string(),
            address_type: AddressType::Extern,
            ipvx_raw: Some(Ipv4Addr::new(8, 8, 8, 8).into()),
        };

        let filter_by_multiple_conditions = FilterOptions {
            by_local_port: Some("8080".to_string()),
            by_pid: Some("123".to_string()),
            by_program: Some("python".to_string()),
            by_listen: true,
            ..Default::default()
        };
        assert!(!filter_out_connection(
            &conn,
            &filter_by_multiple_conditions
        ));

        conn.state = "close".to_string();
        assert!(filter_out_connection(&conn, &filter_by_multiple_conditions));
    }
}
