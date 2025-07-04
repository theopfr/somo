pub mod common;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

use crate::schemas::Connection;
use crate::schemas::FilterOptions;

/// Retrieves all TCP/UDP network connections based on the current operating system (Linux or macOS).
///
/// # Arguments
/// * `filter_options`: The filter options provided by the user.
///
/// # Returns
/// All processed and filtered TCP/UDP connections as a `Connection` struct in a vector.
pub fn get_all_connections(filter_options: &FilterOptions) -> Vec<Connection> {
    #[cfg(target_os = "linux")]
    {
        linux::get_connections(filter_options)
    }

    #[cfg(target_os = "macos")]
    {
        macos::get_connections(filter_options)
    }
}
