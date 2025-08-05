use handlebars::{Handlebars, RenderErrorReason};
use termimad::*;

use crate::markdown::{build_table_header, build_table_row, set_table_style, get_row_alignment, markdown_fmt_row};
use crate::schemas::{AddressType, Connection};
use crate::utils::{pretty_print_syntax_error};
use crate::{sout, utils};


/// Marks localhost and unspecified IP addresses (i.e., 0.0.0.0) using Markdown formatting
///
/// * `address_type` == Localhost -> *italic* + "localhost"
/// * `address_type` == Unspecified -> *italic*
/// * `address_type` == Extern -> not formatted
///
/// # Arguments
/// * `remote_address`: The remote address.
/// * `address_type`: The address type as an AddressType enum.
///
/// # Example
/// ```
/// let address = "127.0.0.1".to_string();
/// let address_type = AddressType::Localhost;
/// let formatted = format_known_address(&address, &address_type);
/// assert_eq!(formatted, "*127.0.0.1 localhost*");
/// ```
///
/// # Returns
/// A Markdown formatted string based on the address-type.
fn format_known_address(remote_address: &str, address_type: &AddressType) -> String {
    match address_type {
        AddressType::Unspecified => {
            format!("*{remote_address}*")
        }
        AddressType::Localhost => {
            format!("*{remote_address} localhost*")
        }
        AddressType::Extern => remote_address.to_string(),
    }
}


/// Prints all current connections in a pretty Markdown table.
///
/// # Arguments
/// * `all_connections`: A list containing all current connections as a `Connection` struct.
///
/// # Returns
/// None
pub fn print_connections_table(all_connections: &[Connection], use_compact_mode: bool) {
    let skin: MadSkin = set_table_style();

    const COLUMN_NAMES: &[&str] = &[
        "**#**",
        "**proto**",
        "**local port**",
        "**remote address**",
        "**remote port**",
        "**pid** *program*",
        "**state**"
    ];
    const NUM_TABLE_COLUMNS: usize = COLUMN_NAMES.len();

    let markdown_row_seperator = &markdown_fmt_row(NUM_TABLE_COLUMNS, get_row_alignment(use_compact_mode));
    let mut output_table = String::new();

    output_table.push_str(markdown_row_seperator);
    output_table.push_str(&build_table_header(COLUMN_NAMES));
    output_table.push_str(markdown_row_seperator);

    for (idx, connection) in all_connections.iter().enumerate() {
        let formatted_remote_address: String =
            format_known_address(&connection.remote_address, &connection.address_type);
        let formatted_pid_info: String = format!("{} *{}*", connection.pid, connection.program);

        output_table.push_str(&build_table_row(idx + 1, &[
            &connection.proto,
            &connection.local_port,
            &formatted_remote_address,
            &connection.remote_port,
            &formatted_pid_info,
            &connection.state,
        ]));

        if !use_compact_mode && idx < all_connections.len() - 1 {
            output_table.push_str(markdown_row_seperator);
        }
    }

    output_table.push_str(markdown_row_seperator);

    sout!("{}", skin.term_text(&output_table));
    utils::pretty_print_info(&format!("{} Connections", all_connections.len()))
}


/// Prints all current connections in a json format.
///
/// # Arguments
/// * `all_connections`: A list containing all current connections as a `Connection` struct.
///
/// # Returns
/// None
pub fn get_connections_json(all_connections: &Vec<Connection>) -> String {
    serde_json::to_string_pretty(all_connections).unwrap()
}

/// Prints all current connections in a custom format.
///
/// # Arguments
/// * `all_connections`: A list containing all current connections as a `Connection` struct.
/// * `template_string`: A string template format for an output
///
/// # Returns
/// None
pub fn get_connections_formatted(
    all_connections: &Vec<Connection>,
    template_string: &String,
) -> String {
    let mut registry = Handlebars::new();
    registry.set_strict_mode(true);

    if let Err(err) = registry.register_template_string("connection_template", template_string) {
        let (line_no, column_no) = err.pos().unwrap_or((1, 1));

        pretty_print_syntax_error(
            "Invalid template syntax.",
            template_string,
            line_no,
            column_no,
        );
        std::process::exit(2);
    }

    let mut rendered_lines = Vec::new();

    for connection in all_connections {
        let json_value = serde_json::to_value(connection).unwrap();
        let rendered_line = registry.render("connection_template", &json_value);

        if let Err(err) = rendered_line {
            let (line_no, column_no) = (err.line_no.unwrap_or(1), err.column_no.unwrap_or(1));

            match err.reason() {
                RenderErrorReason::MissingVariable(Some(var_name)) => {
                    pretty_print_syntax_error(
                        &format!("Invalid template variable '{var_name}'."),
                        template_string,
                        line_no,
                        column_no,
                    );
                }
                _ => {
                    pretty_print_syntax_error(
                        &format!("Template error - {}", err.reason()),
                        template_string,
                        line_no,
                        column_no,
                    );
                }
            }
            std::process::exit(2);
        }

        rendered_lines.push(rendered_line.unwrap());
    }

    rendered_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use super::*;

    #[test]
    fn test_format_known_address_localhost() {
        let addr = "127.0.0.1".to_string();
        let result = format_known_address(&addr, &AddressType::Localhost);
        assert_eq!(result, "*127.0.0.1 localhost*");
    }

    #[test]
    fn test_format_known_address_unspecified() {
        let addr = "0.0.0.0".to_string();
        let result = format_known_address(&addr, &AddressType::Unspecified);
        assert_eq!(result, "*0.0.0.0*");
    }

    #[test]
    fn test_format_known_address_extern() {
        let addr = "123.123.123".to_string();
        let result = format_known_address(&addr, &AddressType::Extern);
        assert_eq!(result, "123.123.123");
    }

    #[test]
    fn test_get_connections_formatted() {
        let connections = vec![
            Connection {
                proto: "tcp".to_string(),
                local_port: "44796".to_string(),
                remote_address: "192.168.1.0".to_string(),
                remote_port: "443".to_string(),
                program: "firefox".to_string(),
                pid: "200".to_string(),
                state: "established".to_string(),
                address_type: AddressType::Localhost,
                ipvx_raw: Ipv4Addr::new(192, 168, 1, 0).into(),
            },
            Connection {
                proto: "tcp".to_string(),
                local_port: "33263".to_string(),
                remote_address: "[::ffff:65.9.95.5]".to_string(),
                remote_port: "443".to_string(),
                program: "-".to_string(),
                pid: "-".to_string(),
                state: "timewait".to_string(),
                address_type: AddressType::Extern,
                ipvx_raw: Ipv6Addr::new(0, 0, 0, 0xffff, 65, 9, 95, 5).into(),
            },
        ];

        let template_and_expected_result = [
            ("PID: {{pid}}, Protocol: {{proto}}, Remote Address: {{remote_address}}".to_string(),
             "PID: 200, Protocol: tcp, Remote Address: 192.168.1.0\nPID: -, Protocol: tcp, Remote Address: [::ffff:65.9.95.5]".to_string()),
            ("Protocol: {{proto}}, Local Port: {{local_port}}, Remote Address: {{remote_address}}, Remote Port: {{remote_port}}, Program: {{program}}, PID: {{pid}}, State: {{state}}, Address Type: {{address_type}}".to_string(),
             "Protocol: tcp, Local Port: 44796, Remote Address: 192.168.1.0, Remote Port: 443, Program: firefox, PID: 200, State: established, Address Type: Localhost\nProtocol: tcp, Local Port: 33263, Remote Address: [::ffff:65.9.95.5], Remote Port: 443, Program: -, PID: -, State: timewait, Address Type: Extern".to_string()),
        ];

        for (template, expected_result) in &template_and_expected_result {
            let result = get_connections_formatted(&connections, template);

            assert_eq!(result.as_str(), expected_result.as_str());
        }
    }
}
