use procfs::process::FDTarget;
use procfs::process::Stat;
use std::collections::HashMap;

use crate::schemas::AddressType;
use crate::schemas::Connection;
use crate::schemas::FilterOptions;
use crate::schemas::NetEntry;
use crate::utils;

/// Gets all running processes on the system using the "procfs" crate.
/// This code is taken from the "procfs" crate documentation.
///
/// # Arguments
/// None
///
/// # Returns
/// A map of all current processes.
fn get_processes() -> HashMap<u64, Stat> {
    let all_procs = procfs::process::all_processes().unwrap();

    let mut map: HashMap<u64, Stat> = HashMap::new();
    for p in all_procs {
        let process = p.unwrap();
        if let (Ok(stat), Ok(fds)) = (process.stat(), process.fd()) {
            for fd in fds {
                if let FDTarget::Socket(inode) = fd.unwrap().target {
                    map.insert(inode, stat.clone());
                }
            }
        }
    }
    map
}

/// Checks if a connection should be filtered out based on options provided by the user.
///
/// # Arguments
/// * `connection_details`: The connection to check for filtering.
/// * `filter_options`: The filter options provided by the user.
///
/// # Returns
/// `true` if the connection should be filtered out, `false` if not.
fn filter_out_connection(connection_details: &Connection, filter_options: &FilterOptions) -> bool {
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

    return false;
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
fn get_address_type(remote_address: &str) -> AddressType {
    if remote_address == "127.0.0.1" || remote_address == "[::1]" {
        return AddressType::Localhost;
    } else if remote_address == "0.0.0.0" || remote_address == "[::]" {
        return AddressType::Unspecified;
    }
    AddressType::Extern
}

fn get_connection_data(net_entry: NetEntry, all_processes: &HashMap<u64, Stat>) -> Connection {
    // process the remote-address and remote-port by spliting them at ":"
    let (_, local_port) = utils::get_address_parts(&format!("{}", net_entry.local_address));
    let (remote_address, remote_port) =
        utils::get_address_parts(&format!("{}", net_entry.remote_address));
    let state = net_entry.state;

    // check if there is no program/pid information
    let (program, pid) = all_processes
        .get(&net_entry.inode)
        .map(|stat| (stat.comm.to_string(), stat.pid.to_string()))
        .unwrap_or(("-".to_string(), "-".to_string()));

    let address_type: AddressType = get_address_type(&remote_address);

    let connection: Connection = Connection {
        proto: net_entry.protocol,
        local_port,
        remote_address: remote_address.to_string(),
        remote_port,
        program,
        pid,
        state,
        address_type,
    };

    return connection;
}

/// Gets all currently open TCP connections using the "procfs" crate and processes them.
///
/// # Arguments
/// * `all_processes`: A map of all running processes on the system.
/// * `filter_options`: The filter options provided by the user.
///
/// # Returns
/// All processed and filtered TCP connections as a `Connection` struct in a vector.
fn get_tcp_connections(
    all_processes: &HashMap<u64, Stat>,
    filter_options: &FilterOptions,
) -> Vec<Connection> {
    let mut tcp_entries = procfs::net::tcp().unwrap();
    if !filter_options.exclude_ipv6 {
        tcp_entries.extend(procfs::net::tcp6().unwrap());
    }

    return tcp_entries
        .iter()
        .filter_map(|entry| {
            let tcp_entry: NetEntry = NetEntry {
                protocol: "tcp".to_string(),
                local_address: entry.local_address,
                remote_address: entry.remote_address,
                state: format!("{:?}", entry.state).to_ascii_lowercase(),
                inode: entry.inode,
            };
            let connection = get_connection_data(tcp_entry, all_processes);

            let filter_connection: bool = filter_out_connection(&connection, filter_options);
            if !filter_connection {
                Some(connection)
            } else {
                None
            }
        })
        .collect();
}

/// Gets all currently open UDP connections using the "procfs" crate and processes them.
/// ###### TODO: combine with the `get_tcp_connections` function if possible.
///
/// # Arguments
/// * `all_processes`: A map of all running processes on the system.
/// * `filter_options`: The filter options provided by the user.
///
/// # Returns
/// All processed and filtered UDP connections as a `Connection` struct in a vector.
fn get_udp_connections(
    all_processes: &HashMap<u64, Stat>,
    filter_options: &FilterOptions,
) -> Vec<Connection> {
    let mut udp_entries = procfs::net::udp().unwrap();
    if !filter_options.exclude_ipv6 {
        udp_entries.extend(procfs::net::udp6().unwrap());
    }

    return udp_entries
        .iter()
        .filter_map(|entry| {
            let udp_entry: NetEntry = NetEntry {
                protocol: "udp".to_string(),
                local_address: entry.local_address,
                remote_address: entry.remote_address,
                state: format!("{:?}", entry.state).to_ascii_lowercase(),
                inode: entry.inode,
            };
            let connection: Connection = get_connection_data(udp_entry, all_processes);

            let filter_connection: bool = filter_out_connection(&connection, filter_options);
            if !filter_connection {
                Some(connection)
            } else {
                None
            }
        })
        .collect();
}

/// Gets both TCP and UDP connections and combines them based on the `proto` filter option.
///
/// # Arguments
/// * `filter_options`: The filter options provided by the user.
///
/// # Returns
/// All processed and filtered TCP/UDP connections as a `Connection` struct in a vector.
pub fn get_all_connections(filter_options: &FilterOptions) -> Vec<Connection> {
    let all_processes = get_processes();

    let mut connections = Vec::new();

    match filter_options.by_proto.as_deref() {
        Some("tcp") => connections.extend(get_tcp_connections(&all_processes, filter_options)),
        Some("udp") => connections.extend(get_udp_connections(&all_processes, filter_options)),
        _ => {
            connections.extend(get_tcp_connections(&all_processes, filter_options));
            connections.extend(get_udp_connections(&all_processes, filter_options));
        }
    }

    return connections;
}

#[cfg(test)]
mod tests {
    use super::*;

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
        };

        let filter_by_matching_port = FilterOptions {
            by_local_port: Some("8080".to_string()),
            ..Default::default()
        };
        assert_eq!(
            filter_out_connection(&conn, &filter_by_matching_port),
            false
        );

        let filter_by_non_matching_port = FilterOptions {
            by_local_port: Some("8181".to_string()),
            ..Default::default()
        };
        assert_eq!(
            filter_out_connection(&conn, &filter_by_non_matching_port),
            true
        );
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
        };

        let filter_by_open_state = FilterOptions {
            by_open: true,
            ..Default::default()
        };
        assert_eq!(filter_out_connection(&conn, &filter_by_open_state), true);

        let no_active_open_filter = FilterOptions {
            by_open: false,
            ..Default::default()
        };
        assert_eq!(filter_out_connection(&conn, &no_active_open_filter), false);

        conn.state = "listen".to_string();

        let filter_by_listen_state = FilterOptions {
            by_listen: true,
            ..Default::default()
        };
        assert_eq!(filter_out_connection(&conn, &filter_by_listen_state), false);

        let no_active_listen_filter = FilterOptions {
            by_listen: false,
            ..Default::default()
        };
        assert_eq!(
            filter_out_connection(&conn, &no_active_listen_filter),
            false
        );
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
        };

        let filter_by_open_state = FilterOptions {
            by_pid: Some("123".to_string()),
            ..Default::default()
        };
        assert_eq!(filter_out_connection(&conn, &filter_by_open_state), false);

        let no_active_open_filter = FilterOptions {
            by_program: Some("postgres".to_string()),
            ..Default::default()
        };
        assert_eq!(filter_out_connection(&conn, &no_active_open_filter), true);
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
        };

        let filter_by_multiple_conditions = FilterOptions {
            by_local_port: Some("8080".to_string()),
            by_pid: Some("123".to_string()),
            by_program: Some("python".to_string()),
            by_listen: true,
            ..Default::default()
        };
        assert_eq!(
            filter_out_connection(&conn, &filter_by_multiple_conditions),
            false
        );

        conn.state = "close".to_string();
        assert_eq!(
            filter_out_connection(&conn, &filter_by_multiple_conditions),
            true
        );
    }
}
