use std::collections::HashSet;
use std::convert::TryInto;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use libc;
use libproc::bsd_info::BSDInfo;
use libproc::file_info::{pidfdinfo, ListFDs, ProcFDType};
use libproc::net_info::{SocketFDInfo, TcpSIState};
use libproc::proc_pid::{listpidinfo, pidinfo};
use libproc::processes::{pids_by_type, ProcFilter};

use crate::connections::common::{filter_out_connection, get_address_type};
use crate::schemas::{Connection, FilterOptions};

/// Convert TcpSIState to connection state string
fn get_tcp_state(status: TcpSIState) -> String {
    // Match the TcpSIState enum to appropriate string representations
    match status {
        TcpSIState::Established => "established".to_string(),
        TcpSIState::Listen => "listening".to_string(),
        TcpSIState::SynSent => "syn_sent".to_string(),
        TcpSIState::SynReceived => "syn_received".to_string(),
        TcpSIState::FinWait1 => "fin_wait1".to_string(),
        TcpSIState::FinWait2 => "fin_wait2".to_string(),
        TcpSIState::TimeWait => "time_wait".to_string(),
        TcpSIState::Closed => "closed".to_string(),
        TcpSIState::CloseWait => "close_wait".to_string(),
        TcpSIState::LastAck => "last_ack".to_string(),
        TcpSIState::Closing => "closing".to_string(),
        _ => "unknown".to_string(),
    }
}

/// Create a connection object from TCP socket information
///
/// # Arguments
/// * `pid`: Process ID
/// * `program`: Process name
/// * `socket_info`: Socket information
/// * `exclude_ipv6`: Whether to exclude IPv6 connections
///
/// # Returns
/// A Connection object if the socket is a TCP socket, otherwise None
fn tcp_socket_to_connection(
    pid: i32,
    program: String,
    socket_info: &SocketFDInfo,
    exclude_ipv6: bool,
) -> Option<Connection> {
    unsafe {
        // Check if this is a TCP socket of the appropriate family (IPv4 or IPv6)
        match socket_info.psi.soi_family {
            libc::AF_INET if socket_info.psi.soi_protocol == libc::IPPROTO_TCP => {
                let tcp_info = socket_info.psi.soi_proto.pri_tcp;

                // Get the local port number, handling byte order conversion
                // Swap bytes to convert from network byte order to host byte order
                let local_port = (((tcp_info.tcpsi_ini.insi_lport >> 8) & 0xff)
                    | ((tcp_info.tcpsi_ini.insi_lport << 8) & 0xff00))
                    .to_string();

                // Get IPv4 address - convert in_addr to u32
                let in_addr = tcp_info.tcpsi_ini.insi_faddr.ina_46.i46a_addr4.s_addr;
                let ip_addr = IpAddr::V4(Ipv4Addr::from(u32::from_be(in_addr)));

                // Extract readable form of address and port
                let remote_addr = format!("{}", ip_addr);

                // Get the remote port number, handling byte order conversion
                let remote_port = (((tcp_info.tcpsi_ini.insi_fport >> 8) & 0xff)
                    | ((tcp_info.tcpsi_ini.insi_fport << 8) & 0xff00))
                    .to_string();

                // Get the connection state
                let state = get_tcp_state(tcp_info.tcpsi_state.into());

                // Calculate address type (private, local, etc.)
                let addr_type = get_address_type(&remote_addr);

                // Return a new Connection object
                Some(Connection {
                    proto: "tcp".to_string(),
                    local_port,
                    remote_address: remote_addr,
                    remote_port,
                    program,
                    pid: pid.to_string(),
                    state,
                    address_type: addr_type,
                    ipvx_raw: ip_addr,
                })
            }
            libc::AF_INET6
                if socket_info.psi.soi_protocol == libc::IPPROTO_TCP && !exclude_ipv6 =>
            {
                let tcp_info = socket_info.psi.soi_proto.pri_tcp;

                // Get the local port number, handling byte order conversion
                let local_port = (((tcp_info.tcpsi_ini.insi_lport >> 8) & 0xff)
                    | ((tcp_info.tcpsi_ini.insi_lport << 8) & 0xff00))
                    .to_string();

                // Extract IPv6 address
                let addr = tcp_info.tcpsi_ini.insi_faddr.ina_6;

                // Build IPv6 address by combining byte pairs into 16-bit segments
                let segments = [
                    ((addr.s6_addr[0] as u16) << 8) | (addr.s6_addr[1] as u16),
                    ((addr.s6_addr[2] as u16) << 8) | (addr.s6_addr[3] as u16),
                    ((addr.s6_addr[4] as u16) << 8) | (addr.s6_addr[5] as u16),
                    ((addr.s6_addr[6] as u16) << 8) | (addr.s6_addr[7] as u16),
                    ((addr.s6_addr[8] as u16) << 8) | (addr.s6_addr[9] as u16),
                    ((addr.s6_addr[10] as u16) << 8) | (addr.s6_addr[11] as u16),
                    ((addr.s6_addr[12] as u16) << 8) | (addr.s6_addr[13] as u16),
                    ((addr.s6_addr[14] as u16) << 8) | (addr.s6_addr[15] as u16),
                ];

                // Create IPv6 address from segments
                let ip_addr = IpAddr::V6(Ipv6Addr::from(segments));

                // Extract readable form of address
                let remote_addr = format!("{}", ip_addr);

                // Get the remote port number, handling byte order conversion
                let remote_port = (((tcp_info.tcpsi_ini.insi_fport >> 8) & 0xff)
                    | ((tcp_info.tcpsi_ini.insi_fport << 8) & 0xff00))
                    .to_string();

                // Get the connection state
                let state = get_tcp_state(tcp_info.tcpsi_state.into());

                // Calculate address type (private, local, etc.)
                let addr_type = get_address_type(&remote_addr);

                // Return a new Connection object
                Some(Connection {
                    proto: "tcp".to_string(),
                    local_port,
                    remote_address: remote_addr,
                    remote_port,
                    program,
                    pid: pid.to_string(),
                    state,
                    address_type: addr_type,
                    ipvx_raw: ip_addr,
                })
            }
            _ => None,
        }
    }
}

/// Create a connection object from UDP socket information
///
/// # Arguments
/// * `pid`: Process ID
/// * `program`: Process name
/// * `socket_info`: Socket information
/// * `exclude_ipv6`: Whether to exclude IPv6 connections
///
/// # Returns
/// A Connection object if the socket is a UDP socket, otherwise None
fn udp_socket_to_connection(
    pid: i32,
    program: String,
    socket_info: &SocketFDInfo,
    exclude_ipv6: bool,
) -> Option<Connection> {
    unsafe {
        // Check if this is a UDP socket of the appropriate family (IPv4 or IPv6)
        match socket_info.psi.soi_family {
            libc::AF_INET if socket_info.psi.soi_protocol == libc::IPPROTO_UDP => {
                let udp_info = socket_info.psi.soi_proto.pri_in;

                // Get the local port number, handling byte order conversion
                let local_port = (((udp_info.insi_lport >> 8) & 0xff)
                    | ((udp_info.insi_lport << 8) & 0xff00))
                    .to_string();

                // Get IPv4 address - convert in_addr to u32
                let in_addr = udp_info.insi_faddr.ina_46.i46a_addr4.s_addr;
                let ip_addr = IpAddr::V4(Ipv4Addr::from(u32::from_be(in_addr)));

                // Extract readable form of address and port
                let remote_addr = format!("{}", ip_addr);

                // Get the remote port number, handling byte order conversion
                let remote_port = (((udp_info.insi_fport >> 8) & 0xff)
                    | ((udp_info.insi_fport << 8) & 0xff00))
                    .to_string();

                // For UDP, we infer state based on remote address and port
                // If both are 0, it's a listening socket, otherwise established
                let state = if remote_addr == "0.0.0.0" && remote_port == "0" {
                    "listening".to_string()
                } else {
                    "established".to_string()
                };

                // Calculate address type (private, local, etc.)
                let addr_type = get_address_type(&remote_addr);

                // Return a new Connection object
                Some(Connection {
                    proto: "udp".to_string(),
                    local_port,
                    remote_address: remote_addr,
                    remote_port,
                    program,
                    pid: pid.to_string(),
                    state,
                    address_type: addr_type,
                    ipvx_raw: ip_addr,
                })
            }
            libc::AF_INET6
                if socket_info.psi.soi_protocol == libc::IPPROTO_UDP && !exclude_ipv6 =>
            {
                let udp_info = socket_info.psi.soi_proto.pri_in;

                // Get the local port number, handling byte order conversion
                let local_port = (((udp_info.insi_lport >> 8) & 0xff)
                    | ((udp_info.insi_lport << 8) & 0xff00))
                    .to_string();

                // Extract IPv6 address
                let addr = udp_info.insi_faddr.ina_6;

                // Build IPv6 address by combining byte pairs into 16-bit segments
                let segments = [
                    ((addr.s6_addr[0] as u16) << 8) | (addr.s6_addr[1] as u16),
                    ((addr.s6_addr[2] as u16) << 8) | (addr.s6_addr[3] as u16),
                    ((addr.s6_addr[4] as u16) << 8) | (addr.s6_addr[5] as u16),
                    ((addr.s6_addr[6] as u16) << 8) | (addr.s6_addr[7] as u16),
                    ((addr.s6_addr[8] as u16) << 8) | (addr.s6_addr[9] as u16),
                    ((addr.s6_addr[10] as u16) << 8) | (addr.s6_addr[11] as u16),
                    ((addr.s6_addr[12] as u16) << 8) | (addr.s6_addr[13] as u16),
                    ((addr.s6_addr[14] as u16) << 8) | (addr.s6_addr[15] as u16),
                ];

                // Create IPv6 address from segments
                let ip_addr = IpAddr::V6(Ipv6Addr::from(segments));

                // Extract readable form of address
                let remote_addr = format!("{}", ip_addr);

                // Get the remote port number, handling byte order conversion
                let remote_port = (((udp_info.insi_fport >> 8) & 0xff)
                    | ((udp_info.insi_fport << 8) & 0xff00))
                    .to_string();

                // For UDP IPv6, we infer state based on remote address and port
                // If using the IPv6 unspecified address (::) and port 0, it's a listening socket
                let state = if remote_addr == "::" && remote_port == "0" {
                    "listening".to_string()
                } else {
                    "established".to_string()
                };

                // Calculate address type (private, local, etc.)
                let addr_type = get_address_type(&remote_addr);

                // Return a new Connection object
                Some(Connection {
                    proto: "udp".to_string(),
                    local_port,
                    remote_address: remote_addr,
                    remote_port,
                    program,
                    pid: pid.to_string(),
                    state,
                    address_type: addr_type,
                    ipvx_raw: ip_addr,
                })
            }
            _ => None,
        }
    }
}

/// Create a unique connection identifier for deduplication
///
/// # Arguments
/// * `conn`: Connection object
///
/// # Returns
/// A unique connection identifier
fn create_connection_key(conn: &Connection) -> String {
    // Format: protocol:local_port:remote_address:remote_port.state:pid
    format!(
        "{}:{}:{}:{}.{}:{}",
        conn.proto, conn.local_port, conn.remote_address, conn.remote_port, conn.state, conn.pid
    )
}

/// Get process name from its PID
///
/// # Arguments
/// * `pid`: Process ID
///
/// # Returns
/// The process name
fn get_process_name(pid: i32) -> String {
    // Try to get the process name, return "unknown[pid]" if it fails
    match libproc::proc_pid::name(pid) {
        Ok(name) => name,
        Err(_) => format!("unknown[{}]", pid),
    }
}

/// Get network connection information on macOS using the libproc library
///
/// # Arguments
/// * `filter_options`: User-provided filter options
///
/// # Returns
/// All processed and filtered TCP/UDP connections as Vec<Connection>
pub fn get_connections(filter_options: &FilterOptions) -> Vec<Connection> {
    // Return early if no protocols are requested
    if !filter_options.by_proto.tcp && !filter_options.by_proto.udp {
        return Vec::new();
    }

    // Get all process PIDs
    let pids = match pids_by_type(ProcFilter::All) {
        Ok(pids) => pids,
        Err(_) => return Vec::new(),
    };

    // HashSet to track seen connections for deduplication
    let mut seen_connections = HashSet::new();
    let mut results = Vec::new();

    // Iterate through all processes
    for &pid in &pids {
        // Skip kernel process
        if pid == 0 {
            continue;
        }

        // Process connections for this PID
        process_pid_connections(pid, filter_options, &mut seen_connections, &mut results);
    }

    results
}

/// Process connections for a single PID
///
/// # Arguments
/// * `pid`: Process ID
/// * `filter_options`: User-provided filter options
/// * `seen_connections`: HashSet to track seen connections for deduplication
/// * `results`: Vector to store the results
fn process_pid_connections(
    pid: u32,
    filter_options: &FilterOptions,
    seen_connections: &mut HashSet<String>,
    results: &mut Vec<Connection>,
) {
    // Convert u32 type pid to i32 for API calls
    let i_pid: i32 = match pid.try_into() {
        Ok(p) => p,
        Err(_) => return, // Skip pids that can't be converted
    };

    // Get process name
    let process_name = get_process_name(i_pid);

    // Get BSD info to determine the number of file descriptors
    let bsd_info = match pidinfo::<BSDInfo>(i_pid, 0) {
        Ok(info) => info,
        Err(_) => return,
    };

    // Get the list of file descriptors for the process
    let fds = match listpidinfo::<ListFDs>(i_pid, bsd_info.pbi_nfiles as usize) {
        Ok(fds) => fds,
        Err(_) => return,
    };

    // Process each file descriptor
    for fd in fds {
        // Only process socket type file descriptors
        if let ProcFDType::Socket = fd.proc_fdtype.into() {
            process_socket_fd(
                i_pid,
                fd.proc_fd,
                &process_name,
                filter_options,
                seen_connections,
                results,
            );
        }
    }
}

/// Process a socket file descriptor
///
/// # Arguments
/// * `pid`: Process ID
/// * `fd`: File descriptor
/// * `process_name`: Process name
/// * `filter_options`: User-provided filter options
fn process_socket_fd(
    pid: i32,
    fd: i32,
    process_name: &str,
    filter_options: &FilterOptions,
    seen_connections: &mut HashSet<String>,
    results: &mut Vec<Connection>,
) {
    // Get detailed socket information
    let socket_info = match pidfdinfo::<SocketFDInfo>(pid, fd) {
        Ok(info) => info,
        Err(_) => return,
    };

    // Process TCP connections if requested
    if filter_options.by_proto.tcp && socket_info.psi.soi_protocol == libc::IPPROTO_TCP {
        if let Some(conn) = tcp_socket_to_connection(
            pid,
            process_name.to_string(),
            &socket_info,
            filter_options.exclude_ipv6,
        ) {
            add_connection_if_new(conn, filter_options, seen_connections, results);
        }
    }

    // Process UDP connections if requested
    if filter_options.by_proto.udp && socket_info.psi.soi_protocol == libc::IPPROTO_UDP {
        if let Some(conn) = udp_socket_to_connection(
            pid,
            process_name.to_string(),
            &socket_info,
            filter_options.exclude_ipv6,
        ) {
            add_connection_if_new(conn, filter_options, seen_connections, results);
        }
    }
}

/// Add a connection to the results if it's new and passes filters
///
/// # Arguments
/// * `conn`: Connection object
/// * `filter_options`: User-provided filter options
/// * `seen_connections`: HashSet to track seen connections for deduplication
/// * `results`: Vector to store the results
fn add_connection_if_new(
    conn: Connection,
    filter_options: &FilterOptions,
    seen_connections: &mut HashSet<String>,
    results: &mut Vec<Connection>,
) {
    // Create a unique key for deduplication
    let connection_key = create_connection_key(&conn);

    // Skip if we've seen this connection before
    if !seen_connections.insert(connection_key) {
        return;
    }

    // Add to results if it passes the filter
    if !filter_out_connection(&conn, filter_options) {
        results.push(conn);
    }
}
