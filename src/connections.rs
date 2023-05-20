
use procfs;
use procfs::process::Stat;
use procfs::process::FDTarget;

use std::collections::HashMap;

use crate::string_utils;



pub struct FilterOptions {
    pub by_proto: Option<String>,
    pub by_program: Option<String>,
    pub by_pid: Option<String>,
    pub by_remote_address: Option<String>,
    pub by_remote_port: Option<String>,
    pub exclude_ipv6: bool
}


#[derive(Debug)]
pub struct Connection {
    pub proto: String,
    pub local_port: String,
    pub remote_address: String,
    pub remote_port: String,
    pub program: String,
    pub pid: String,
    pub state: String,
}



fn get_processes() -> HashMap<u64, Stat> {
    /* gets all running processes on the system */

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
    return map;
}



fn filter_connection(connection_details: &Connection, filter_options: &FilterOptions) -> bool {
    /* filter connections by remote-port, remote-address, program-name or pid */

    match &filter_options.by_remote_port {
        Some(filter_remote_port) if &connection_details.remote_port != filter_remote_port => return true,
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

    return false;
}



fn get_tcp_connections(all_processes: &HashMap<u64, Stat>, filter_options: &FilterOptions) -> Vec<Connection> {
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

        let connection: Connection = Connection {
            proto: "tcp".to_string(),
            local_port: local_port,
            remote_address: remote_address,
            remote_port: remote_port,
            program: program,
            pid: pid,
            state: state,
        };

        let filter_connection: bool = filter_connection(&connection, filter_options);
        if filter_connection {
            continue;
        }

        all_tcp_connections.push(connection);
    }

    return all_tcp_connections;
}


fn get_udp_connections(all_processes: &HashMap<u64, Stat>, filter_options: &FilterOptions) -> Vec<Connection> {
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
        } else {
            program = "-".to_string();
            pid = "-".to_string();
        }

        let connection: Connection = Connection {
            proto: "udp".to_string(),
            local_port: local_port,
            remote_address: remote_address,
            remote_port: remote_port,
            program: program,
            pid: pid,
            state: state,
        };

        let filter_connection: bool = filter_connection(&connection, filter_options);
        if filter_connection {
            continue;
        }

        all_udp_connections.push(connection);
    }

    return all_udp_connections;
}




pub fn get_all_connections(filter_options: &FilterOptions) -> Vec<Connection> {

    let all_processes: HashMap<u64, Stat> = get_processes();

    match &filter_options.by_proto {
        Some(filter_proto) if filter_proto == "tcp" => return get_tcp_connections(&all_processes, filter_options),
        Some(filter_proto) if filter_proto == "udp" => return get_udp_connections(&all_processes, filter_options),
        _ => { }
    }

    let mut all_connections = get_tcp_connections(&all_processes, filter_options);
    let all_udp_connections = get_udp_connections(&all_processes, filter_options);
    all_connections.extend(all_udp_connections);

    return all_connections;
}

