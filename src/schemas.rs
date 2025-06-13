use std::net::SocketAddr;

/// Represents the type of an IP address.
///
/// # Variants
/// * `Localhost`: Represents the localhost/127.0.0.1 address.
/// * `Unspecified`: Represents an unspecified or wildcard address.
/// * `Extern`: Represents an external address.
#[derive(Debug, PartialEq)]
pub enum AddressType {
    Localhost,
    Unspecified,
    Extern,
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
    pub address_type: AddressType,
}

/// General struct type for TCP and UDP entries.
#[derive(Debug)]
pub struct NetEntry {
    pub protocol: String,
    pub local_address: SocketAddr,
    pub remote_address: SocketAddr,
    pub state: String,
    pub inode: u64,
}

/// Contains options for filtering a `Conntection`.
#[derive(Debug, Default)]
pub struct FilterOptions {
    pub by_proto: Option<String>,
    pub by_program: Option<String>,
    pub by_pid: Option<String>,
    pub by_remote_address: Option<String>,
    pub by_remote_port: Option<String>,
    pub by_local_port: Option<String>,
    pub by_open: bool,
    pub by_listen: bool,
    pub exclude_ipv6: bool,
}
