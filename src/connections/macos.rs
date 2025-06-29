use libproc::libproc::proc_pid;
use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo as NetstatSocketInfo,
};
use std::collections::HashSet;

use crate::schemas::{Connection, FilterOptions};

use crate::connections::common::{filter_out_connection, get_address_type};

/// Gets the process name for a given PID on macOS
fn get_process_name(pid: i32) -> String {
    match proc_pid::name(pid) {
        Ok(name) => name,
        Err(_) => "-".to_string(),
    }
}

/// Gets network connection information on macOS using the netstat2 library.
///
/// # Arguments
/// * `filter_options`: The filter options provided by the user.
///
/// # Returns
/// All processed and filtered TCP/UDP connections as a `Connection` struct in a vector.
pub fn get_connections(filter_options: &FilterOptions) -> Vec<Connection> {
    // Determine which address families to include
    let mut af_flags = AddressFamilyFlags::empty();
    if !filter_options.exclude_ipv6 {
        af_flags |= AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    } else {
        af_flags |= AddressFamilyFlags::IPV4;
    }

    // Determine which protocols to include
    let mut proto_flags = ProtocolFlags::empty();
    if filter_options.by_proto.tcp {
        proto_flags |= ProtocolFlags::TCP;
    }
    if filter_options.by_proto.udp {
        proto_flags |= ProtocolFlags::UDP;
    }

    // Get the socket information using netstat2
    let sockets_info = match get_sockets_info(af_flags, proto_flags) {
        Ok(sockets) => sockets,
        Err(_) => return Vec::new(),
    };

    // Temporary storage for connection features, for deduplication
    let mut seen_connections = HashSet::new();

    // Convert the socket information to our Connection type
    sockets_info
        .iter()
        .filter_map(|si| {
            let (proto, local_port, remote_address, remote_port, state) =
                match &si.protocol_socket_info {
                    NetstatSocketInfo::Tcp(tcp_si) => {
                        let state = format!("{}", tcp_si.state).to_ascii_lowercase();
                        (
                            "tcp".to_string(),
                            tcp_si.local_port.to_string(),
                            tcp_si.remote_addr.to_string(),
                            tcp_si.remote_port.to_string(),
                            state,
                        )
                    }
                    NetstatSocketInfo::Udp(udp_si) => (
                        "udp".to_string(),
                        udp_si.local_port.to_string(),
                        "0.0.0.0".to_string(),
                        "-".to_string(),
                        "-".to_string(),
                    ),
                };

            // Get program and PID info
            let (program, pid) = if let Some(first_pid) = si.associated_pids.first() {
                // Get process name
                let proc_name = get_process_name(*first_pid as i32);
                (proc_name, first_pid.to_string())
            } else {
                ("-".to_string(), "-".to_string())
            };

            // Create a unique key for deduplication
            let connection_key = format!(
                "{}:{}:{}:{}.{}:{}",
                proto, local_port, remote_address, remote_port, state, pid
            );

            // If the connection has already been processed, skip it
            if !seen_connections.insert(connection_key) {
                return None;
            }

            // Create our connection data structure
            let conn = Connection {
                proto,
                local_port,
                remote_address: remote_address.clone(),
                remote_port,
                program,
                pid,
                state,
                address_type: get_address_type(&remote_address),
                ipvx_raw: si.local_addr(),
            };

            // Filter connections based on the provided options
            if filter_out_connection(&conn, filter_options) {
                None
            } else {
                Some(conn)
            }
        })
        .collect()
}
