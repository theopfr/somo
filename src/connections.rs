
use procfs;
use procfs::process::Stat;
use std::collections::HashMap;

use crate::string_utils;


pub struct FilterOptions {
    pub by_conn_type: Option<String>,
    pub by_program: Option<String>,
    pub by_pid: Option<String>,
    pub by_remote_address: Option<String>,
    pub by_remote_port: Option<String>,
    pub exclude_ipv6: bool
}



#[derive(Debug)]
pub struct Connection {
    pub conn_type: String,
    pub local_port: String,
    pub remote_address: String,
    pub remote_port: String,
    pub program: String,
    pub pid: String,
    pub state: String,
}

pub struct Connections {
    all_processes: HashMap<u64, Stat>,
    filter_options: FilterOptions,
}


impl Connections {
    pub fn new(all_processes: HashMap<u64, Stat>, filter_options: FilterOptions) -> Self {
        Connections {
            all_processes: all_processes,
            filter_options: filter_options,
        }
    }

    pub fn get_all_connections(&self) -> Vec<Connection> {
        // TODO filter tcp, udp

        let mut all_tcp_connections: Vec<Connection> = Vec::new();


        match &self.filter_options.by_conn_type {
            Some(filter_conn_type) if filter_conn_type == "tcp" => return self.get_tcp_connections(),
            Some(filter_conn_type) if filter_conn_type == "udp" => return self.get_udp_connections(),
            _ => { }
        }

        let mut all_tcp_connections = self.get_tcp_connections();
        let all_udp_connections = self.get_udp_connections();
        all_tcp_connections.extend(all_udp_connections);

        return all_tcp_connections;
    }

    fn filter_connection(&self, connection_details: &Connection) -> bool {
        /* filter connections by remote-port, remote-address, program-name or pid */

        match &self.filter_options.by_remote_port {
            Some(filter_remote_port) if &connection_details.remote_port != filter_remote_port => return true,
            _ => { }
        }
        match &self.filter_options.by_remote_address {
            Some(filter_remote_address) if &connection_details.remote_address != filter_remote_address => return true,
            _ => { }
        }
        match &self.filter_options.by_program {
            Some(filter_program) if &connection_details.program != filter_program => return true,
            _ => { }
        }
        match &self.filter_options.by_pid {
            Some(filter_pid) if &connection_details.pid != filter_pid => return true,
            _ => { }
        }

        return false;
    }


    fn get_tcp_connections(&self) -> Vec<Connection>{
        let mut tcp = procfs::net::tcp().unwrap();
        if !self.filter_options.exclude_ipv6 {
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
            if let Some(stat) = self.all_processes.get(&entry.inode) {
                program = stat.comm.to_string();
                pid = stat.pid.to_string();
            } else {
                program = "-".to_string();
                pid = "-".to_string();
            }

            let connection: Connection = Connection {
                conn_type: "tcp".to_string(),
                local_port: local_port,
                remote_address: remote_address,
                remote_port: remote_port,
                program: program,
                pid: pid,
                state: state,
            };

            let filter_connection: bool = self.filter_connection(&connection);
            if filter_connection {
                continue;
            }

            all_tcp_connections.push(connection);
        }

        return all_tcp_connections;
    }


    fn get_udp_connections(&self) -> Vec<Connection>{
        let mut udp = procfs::net::udp().unwrap();
        if !self.filter_options.exclude_ipv6 {
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
            if let Some(stat) = self.all_processes.get(&entry.inode) {
                program = stat.comm.to_string();
                pid = stat.pid.to_string();
            } else {
                program = "-".to_string();
                pid = "-".to_string();
            }

            let connection: Connection = Connection {
                conn_type: "udp".to_string(),
                local_port: local_port,
                remote_address: remote_address,
                remote_port: remote_port,
                program: program,
                pid: pid,
                state: state,
            };

            let filter_connection: bool = self.filter_connection(&connection);
            if filter_connection {
                continue;
            }

            all_udp_connections.push(connection);
        }

        return all_udp_connections;
    }

}
