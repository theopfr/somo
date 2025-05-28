use procfs::process::Stat;
use procfs::process::FDTarget;
use std::collections::HashMap;

use crate::string_utils;
use crate::address_checkers;


/// Contains options for filtering a `Conntection`.
#[derive(Debug)]
pub struct FilterOptions {
    pub by_proto: Option<String>,
    pub by_program: Option<String>,
    pub by_pid: Option<String>,
    pub by_remote_address: Option<String>,
    pub by_remote_port: Option<String>,
    pub by_local_port: Option<String>,
    pub by_open: bool,
    pub exclude_ipv6: bool
}

/// Represents a processed socket connection with all its attributes.
#[derive(Debug)]
pub struct Connection {
    pub proto: String,
    pub local_port: String,
    pub remote_address: String,
    pub remote_port: String,
    pub program: String,
    pub pid: String,
    pub state: String,
    pub address_type: address_checkers::IPType,
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
        Some(filter_remote_port) if &connection_details.remote_port != filter_remote_port => return true,
        _ => { }
    }
    match &filter_options.by_local_port {
        Some(filter_local_port) if &connection_details.local_port != filter_local_port => return true,
        _ => { }
    }
    match &filter_options.by_remote_address {
        Some(filter_remote_address) if &connection_details.remote_address != filter_remote_address => return true,
        _ => { }
    }
    match &filter_options.by_program {
        Some(filter_program) if &connection_details.program != filter_program => return true,
        _ => { }
    }
    match &filter_options.by_pid {
        Some(filter_pid) if &connection_details.pid != filter_pid => return true,
        _ => { }
    }
    if filter_options.by_open && connection_details.state == "close" {
        return true;
    }

    false
}


/// Gets all currently open TCP connections using the "procfs" crate and processes them.
/// 
/// # Arguments
/// * `all_processes`: A map of all running processes on the system.
/// * `filter_options`: The filter options provided by the user.
/// 
/// # Returns
/// All processed and filtered TCP connections as a `Connection` struct in a vector.
async fn get_tcp_connections(all_processes: &HashMap<u64, Stat>, filter_options: &FilterOptions) -> Vec<Connection> {
    let mut tcp = procfs::net::tcp().unwrap();
    if !filter_options.exclude_ipv6 {
        tcp.extend(procfs::net::tcp6().unwrap());
    }

    let mut all_tcp_connections: Vec<Connection> = Vec::new();
    for entry in tcp {

        // process the remote-address and remote-port by spliting them at ":"
        let (_, local_port) = string_utils::get_address_parts(&format!("{}", entry.local_address));
        let (remote_address, remote_port) = string_utils::get_address_parts(&format!("{}", entry.remote_address));
        let state = format!("{:?}", entry.state).to_ascii_lowercase();
        
        // check if there is no program/pid information
        let program: String;
        let pid: String;
        if let Some(stat) = all_processes.get(&entry.inode) {
            program = stat.comm.to_string();
            pid = stat.pid.to_string();
        } else {
            program = "-".to_string();
            pid = "-".to_string();
        }

        let address_type: address_checkers::IPType = address_checkers::check_address_type(&remote_address);

        let connection: Connection = Connection {
            proto: "tcp".to_string(),
            local_port,
            remote_address: remote_address.to_string(),
            remote_port,
            program,
            pid,
            state,
            address_type,
        };

        // check if connection should be filtered out
        let filter_connection: bool = filter_out_connection(&connection, filter_options);
        if filter_connection {
            continue;
        }

        all_tcp_connections.push(connection);
    }

    all_tcp_connections
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
async fn get_udp_connections(all_processes: &HashMap<u64, Stat>, filter_options: &FilterOptions) -> Vec<Connection> {
    let mut udp = procfs::net::udp().unwrap();
    if !filter_options.exclude_ipv6 {
        udp.extend(procfs::net::udp6().unwrap());
    }

    let mut all_udp_connections: Vec<Connection> = Vec::new();
    for entry in udp {

        // process the remote-address and remote-port by spliting them at ":"
        let (_, local_port) = string_utils::get_address_parts(&format!("{}", entry.local_address));
        let (remote_address, remote_port) = string_utils::get_address_parts(&format!("{}", entry.remote_address));
        let state = format!("{:?}", entry.state).to_ascii_lowercase();
        
        // check if there is no program/pid information
        let program: String;
        let pid: String;
        if let Some(stat) = all_processes.get(&entry.inode) {
            program = stat.comm.to_string();
            pid = stat.pid.to_string();
        }
        else {
            program = "-".to_string();
            pid = "-".to_string();
        }

        let address_type: address_checkers::IPType = address_checkers::check_address_type(&remote_address);

        let connection: Connection = Connection {
            proto: "udp".to_string(),
            local_port,
            remote_address: remote_address.to_string(),
            remote_port,
            program,
            pid,
            state,
            address_type,
        };

        // check if connection should be filtered out
        let filter_connection: bool = filter_out_connection(&connection, filter_options);
        if filter_connection {
            continue;
        }

        all_udp_connections.push(connection);
    }

    all_udp_connections
}

 

/// Gets both TCP and UDP connections and combines them based on the `proto` filter option.
/// 
/// # Arguments
/// * `filter_options`: The filter options provided by the user.
/// 
/// # Returns
/// All processed and filtered TCP/UDP connections as a `Connection` struct in a vector.
pub async fn get_all_connections(filter_options: &FilterOptions) -> Vec<Connection> {
    let all_processes: HashMap<u64, Stat> = get_processes();

    match &filter_options.by_proto {
        Some(filter_proto) if filter_proto == "tcp" => return get_tcp_connections(&all_processes, filter_options).await,
        Some(filter_proto) if filter_proto == "udp" => return get_udp_connections(&all_processes, filter_options).await,
        _ => { }
    }

    let mut all_connections = get_tcp_connections(&all_processes, filter_options).await;
    let all_udp_connections = get_udp_connections(&all_processes, filter_options).await;
    all_connections.extend(all_udp_connections);

    all_connections
}

