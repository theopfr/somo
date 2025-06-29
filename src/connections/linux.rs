use procfs::process::FDTarget;
use procfs::process::Stat;
use std::collections::HashMap;

use crate::schemas::{Connection, FilterOptions, NetEntry};

use crate::connections::common::{filter_out_connection, get_address_type};

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
    // process the remote-address and remote-port by spliting them at ":"
    let local_address = format!("{}", net_entry.local_address);
    let local_parts: Vec<&str> = local_address.split(':').collect();
    let local_port = local_parts.last().unwrap_or(&"-").to_string();

    let remote_address_full = format!("{}", net_entry.remote_address);
    let remote_parts: Vec<&str> = remote_address_full.split(':').collect();
    let remote_port = remote_parts.last().unwrap_or(&"-").to_string();
    let remote_address = if remote_parts.len() > 1 {
        remote_parts[0..remote_parts.len() - 1].join(":")
    } else {
        remote_address_full.clone()
    };
    let state = net_entry.state;

    // check if there is no program/pid information
    let (program, pid) = all_processes
        .get(&net_entry.inode)
        .map(|stat| (stat.comm.to_string(), stat.pid.to_string()))
        .unwrap_or(("-".to_string(), "-".to_string()));

    let address_type = get_address_type(&remote_address);

    let connection: Connection = Connection {
        proto: net_entry.protocol,
        local_port,
        remote_address: remote_address.to_string(),
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
    let mut tcp_entries = procfs::net::tcp().unwrap();
    if !filter_options.exclude_ipv6 {
        tcp_entries.extend(procfs::net::tcp6().unwrap());
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

/// Gets both TCP and UDP connections and combines them based on the `proto` filter option.
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
