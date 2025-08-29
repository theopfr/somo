use crate::connections::common::{filter_out_connection, get_address_type};
use crate::schemas::{Connection, FilterOptions};
use libproc::libproc::proc_pid;
use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo as NetstatSocketInfo,
    SocketInfo,
};
use std::collections::HashSet;

/// Retrieves the name of a process given its PID on macOS using the libproc library.
///
/// # Arguments
/// * `pid`: The process ID for which to obtain the process name.
///
/// # Returns
/// A string containing the process name if found, or "-" if the name cannot be retrieved.
fn get_process_name(pid: i32) -> String {
    match proc_pid::name(pid) {
        Ok(name) => name,
        Err(_) => "-".to_string(),
    }
}

/// Parses and filters TCP and/or UDP connections using socket information.
///
/// # Arguments
/// * `sockets_info`: List of socket information coming from the netstat2 crate
/// * `filter_options`: The filter options provided by the user.
///
/// # Returns
/// All filtered TCP/UDP connections as a `Connection` struct in a vector.
fn parse_connections(
    sockets_info: &[SocketInfo],
    filter_options: &FilterOptions,
) -> Vec<Connection> {
    // Temporary storage for connections, for deduplication
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

            let (program, pid) = if let Some(first_pid) = si.associated_pids.first() {
                let proc_name = get_process_name(*first_pid as i32);
                (proc_name, first_pid.to_string())
            } else {
                ("-".to_string(), "-".to_string())
            };

            // Create a unique key for deduplication
            let connection_key =
                format!("{proto}:{local_port}:{remote_address}:{remote_port}:{state}:{pid}");

            // If the connection has already been processed, skip it
            if !seen_connections.insert(connection_key) {
                return None;
            }

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

            if filter_out_connection(&conn, filter_options) {
                None
            } else {
                Some(conn)
            }
        })
        .collect()
}

/// Gets and filters TCP and/or UDP connections using socket information from the netstat2 crate.
///
/// # Arguments
/// * `filter_options`: The filter options provided by the user.
///
/// # Returns
/// All processed and filtered TCP/UDP connections as a `Connection` struct in a vector.
pub fn get_connections(filter_options: &FilterOptions) -> Vec<Connection> {
    let (ipv4_only, ipv6_only, take_both) = resolve_ip_version(filter_options);
    let mut af_flags = AddressFamilyFlags::empty();
    if ipv4_only {
        af_flags |= AddressFamilyFlags::IPV4;
    } else if ipv6_only {
        af_flags |= AddressFamilyFlags::IPV6;
    } else {
        af_flags |= AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    }

    let mut proto_flags = ProtocolFlags::empty();
    if filter_options.by_proto.tcp {
        proto_flags |= ProtocolFlags::TCP;
    }
    if filter_options.by_proto.udp {
        proto_flags |= ProtocolFlags::UDP;
    }

    let sockets_info = match get_sockets_info(af_flags, proto_flags) {
        Ok(sockets) => sockets,
        Err(_) => return Vec::new(),
    };

    parse_connections(&sockets_info, filter_options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::Protocols;
    use netstat2::{ProtocolSocketInfo, SocketInfo, TcpSocketInfo, TcpState};
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_parse_connections_tcp() {
        let mock_socket = SocketInfo {
            protocol_socket_info: ProtocolSocketInfo::Tcp(TcpSocketInfo {
                local_port: 8080,
                remote_port: 443,
                local_addr: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                remote_addr: IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)),
                state: TcpState::Established,
            }),
            associated_pids: vec![1234],
        };

        let filter_options = FilterOptions {
            exclude_ipv6: false,
            by_proto: Protocols {
                tcp: true,
                udp: false,
            },
            ..Default::default()
        };

        let connections = parse_connections(&vec![mock_socket], &filter_options);

        assert_eq!(connections.len(), 1);
        let conn = &connections[0];
        assert_eq!(conn.proto, "tcp");
        assert_eq!(conn.local_port, "8080");
        assert_eq!(conn.remote_port, "443");
        assert_eq!(conn.remote_address, "93.184.216.34");
        assert_eq!(conn.state, "established");
        assert_eq!(conn.pid, "1234");
    }

    #[test]
    fn test_parse_connections_udp() {
        let mock_socket = SocketInfo {
            protocol_socket_info: ProtocolSocketInfo::Udp(netstat2::UdpSocketInfo {
                local_port: 53,
                local_addr: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            }),
            associated_pids: vec![5678],
        };

        let filter_options = FilterOptions {
            by_proto: Protocols {
                tcp: false,
                udp: true,
            },
            ..Default::default()
        };

        let connections = parse_connections(&vec![mock_socket], &filter_options);

        assert_eq!(connections.len(), 1);
        let conn = &connections[0];
        assert_eq!(conn.proto, "udp");
        assert_eq!(conn.local_port, "53");
        assert_eq!(conn.remote_port, "-");
        assert_eq!(conn.remote_address, "0.0.0.0");
        assert_eq!(conn.state, "-");
        assert_eq!(conn.pid, "5678");
    }
}
