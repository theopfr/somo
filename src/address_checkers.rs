
/// Represents the type of an IP address.
///
/// # Variants
/// * `Localhost`: Represents the localhost/127.0.0.1 address.
/// * `Unspecified`: Represents an unspecified or wildcard address.
/// * `Extern`: Represents an external address.
#[derive(Debug)]
pub enum IPType {
    Localhost,
    Unspecified,
    Extern,
}

/// Checks if a given IP address is either "unspecified", localhost or an extern address.
///
/// * `0.0.0.0` or `[::]` -> unspecified
/// * `127.0.0.1` or `[::1]` -> localhost
/// * else -> extern address
///
/// # Arguments
/// * `remote_address`: The address to be checked.
///
/// # Returns
/// The address-type as an IPType enum.
pub fn check_address_type(remote_address: &str) -> IPType {
    if remote_address == "127.0.0.1" || remote_address == "[::1]" {
        return IPType::Localhost;
    } else if remote_address == "0.0.0.0" || remote_address == "[::]" {
        return IPType::Unspecified;
    }
    IPType::Extern
}
