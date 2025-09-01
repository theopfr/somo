use crate::connections::common::{filter_out_connection, get_address_type};
use crate::schemas::{Connection, FilterOptions};
use procfs::net::{TcpNetEntry, UdpNetEntry};
use procfs::process::FDTarget;
use procfs::process::Stat;
use std::collections::HashMap;
use std::net::SocketAddr;

/// General struct type for TCP and UDP entries.
#[derive(Debug)]
pub struct NetEntry {
    pub protocol: String,
    pub local_address: SocketAddr,
    pub remote_address: SocketAddr,
    pub state: String,
    pub inode: u64,
}

/// Splits a string combined of an IP address and port with a ":" delimiter into two parts.
///
/// # Arguments
/// * `address`: The combination of address and port joined by a ":", e.g. "127.0.0.1:5432"
///
/// # Example
/// ```
/// let address_port_1 = "127.0.0.1:5432".to_string();
/// assert_eq!(split_address(address_port_1), Some(("5432", "127.0.0.1")));
///
/// let address_port_2 = "fails.com".to_string();
/// assert_eq!(split_address(address_port_2), None);
/// ```
///
/// # Returns
/// If the string can be successfully split,
/// it will return a tuple containing the address and the port, if not `None`.
fn split_address(address: &str) -> Option<(&str, &str)> {
    static DELIMITER: &str = ":";

    let mut address_parts = address.rsplitn(2, DELIMITER);
    match (address_parts.next(), address_parts.next()) {
        (Some(first), Some(second)) => Some((second, first)),
        _ => None,
    }
}

/// Handles the output of the `split_address` function by replacing the port with a "-" if the string couldn't be split.
/// ###### TODO: maybe combine it with the `split_address` function.
///
/// # Arguments
/// * `address`: The address-port combination which should be split.
///
/// # Example
/// ```
/// let address_port_1 = "127.0.0.1:5432".to_string();
/// assert_eq!(get_address_parts(address_port_1), ("5432", "127.0.0.1"));
///
/// let address_port_2 = "fails.com".to_string();
/// assert_eq!(get_address_parts(address_port_1), ("-", "127.0.0.1"));
/// ```
///
/// # Returns
/// A tuple containing the address and port or just the address and a "-" if there wasn't a port.
fn get_address_parts(address: &str) -> (String, String) {
    split_address(address)
        .map(|(a, p)| (a.to_string(), p.to_string()))
        .unwrap_or((address.to_string(), "-".to_string()))
}

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

fn get_connection_data(net_entry: NetEntry, all_processes: &HashMap<u64, Stat>) -> Connection {
    let local_address_full = format!("{}", net_entry.local_address);
    let (_, local_port) = get_address_parts(&local_address_full);

    let remote_address_full = format!("{}", net_entry.remote_address);
    let (remote_address, remote_port) = get_address_parts(&remote_address_full);
    let state = net_entry.state;

    let (program, pid) = all_processes
        .get(&net_entry.inode)
        .map(|stat| (stat.comm.to_string(), stat.pid.to_string()))
        .unwrap_or(("-".to_string(), "-".to_string()));

    let address_type = get_address_type(&remote_address);

    let connection: Connection = Connection {
        proto: net_entry.protocol,
        local_port,
        remote_address,
        remote_port,
        program,
        pid,
        state,
        address_type,
        ipvx_raw: net_entry.remote_address.ip(),
    };

    connection
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
    let mut tcp_entries: Vec<TcpNetEntry> = Vec::new();

    if filter_options.by_ip_version.ipv4 {
        if let Ok(v4) = procfs::net::tcp() {
            tcp_entries.extend(v4);
        }
    }

    if filter_options.by_ip_version.ipv6 {
        if let Ok(v6) = procfs::net::tcp6() {
            tcp_entries.extend(v6);
        }
    }

    tcp_entries
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
        .collect()
}

/// Gets all currently open UDP connections using the "procfs" crate and processes them.
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
    let mut udp_entries: Vec<UdpNetEntry> = Vec::new();

    if filter_options.by_ip_version.ipv4 {
        if let Ok(v4) = procfs::net::udp() {
            udp_entries.extend(v4);
        }
    }

    if filter_options.by_ip_version.ipv6 {
        if let Ok(v6) = procfs::net::udp6() {
            udp_entries.extend(v6);
        }
    }

    udp_entries
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
        .collect()
}

/// Gets both TCP and UDP connections and combines them based on protocol filter options.
///
/// # Arguments
/// * `filter_options`: The filter options provided by the user.
///
/// # Returns
/// All processed and filtered TCP/UDP connections as a `Connection` struct in a vector.
pub fn get_connections(filter_options: &FilterOptions) -> Vec<Connection> {
    let all_processes = get_processes();

    let mut connections = Vec::new();
    if filter_options.by_proto.tcp {
        connections.extend(get_tcp_connections(&all_processes, filter_options))
    }
    if filter_options.by_proto.udp {
        connections.extend(get_udp_connections(&all_processes, filter_options))
    }

    connections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_address_valid() {
        let addr = "127.0.0.1:5432";
        assert_eq!(split_address(addr), Some(("127.0.0.1", "5432")));

        let addr = "[::1]:8080";
        assert_eq!(split_address(addr), Some(("[::1]", "8080")));
    }

    #[test]
    fn test_split_address_invalid() {
        let addr = "localhost";
        assert_eq!(split_address(addr), None);
        let addr = "192.168.0.1";
        assert_eq!(split_address(addr), None);
    }

    #[test]
    fn test_get_address_parts_valid() {
        let addr = "192.168.0.1:80";
        let (address, port) = get_address_parts(addr);
        assert_eq!(address, "192.168.0.1");
        assert_eq!(port, "80");
    }

    #[test]
    fn test_get_address_parts_invalid() {
        let addr = "example.com";
        let (address, port) = get_address_parts(addr);
        assert_eq!(address, "example.com");
        assert_eq!(port, "-");
    }
}
