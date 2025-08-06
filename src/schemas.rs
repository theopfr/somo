use std::net::IpAddr;

/// Represents the type of an IP address.
///
/// # Variants
/// * `Localhost`: Represents the localhost/127.0.0.1 address.
/// * `Unspecified`: Represents an unspecified or wildcard address.
/// * `Extern`: Represents an external address.
#[derive(Debug, PartialEq, serde::Serialize)]
pub enum AddressType {
    Localhost,
    Unspecified,
    Extern,
}

/// Contains which type(s) of protocols the user wants to see.
#[derive(Debug, Default)]
pub struct Protocols {
    pub tcp: bool,
    pub udp: bool,
}

/// Represents a processed socket connection with all its attributes.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Connection {
    pub proto: String,
    pub local_port: String,
    pub remote_address: String,
    pub remote_port: String,
    pub program: String,
    pub pid: String,
    pub state: String,
    pub address_type: AddressType,

    /// Internal variable used only for ordering operations of raw ipv4/6 addresses
    #[serde(skip_serializing)]
    pub ipvx_raw: IpAddr,
}

/// Contains options for filtering a `Connection`.
#[derive(Debug, Default)]
pub struct FilterOptions {
    pub by_proto: Protocols,
    pub by_program: Option<String>,
    pub by_pid: Option<String>,
    pub by_remote_address: Option<String>,
    pub by_remote_port: Option<String>,
    pub by_local_port: Option<String>,
    pub by_open: bool,
    pub by_listen: bool,
    pub exclude_ipv6: bool,
}

/// Represents the types of network protocols.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl std::str::FromStr for Protocol {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "tcp" => Ok(Protocol::Tcp),
            "udp" => Ok(Protocol::Udp),
            _ => Err(()),
        }
    }
}
