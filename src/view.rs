use crate::markdown::{get_row_alignment, Padding, Table, TableCell};
use crate::schemas::Connection;
use crate::services::get_port_annotation;
use crate::utils::{format_known_address, pretty_print_syntax_error};
use handlebars::{Handlebars, RenderErrorReason};

/// Builds a Markdown formatted table with all current connections.
///
/// # Arguments
/// * `all_connections`: A list containing all current connections as a `Connection` struct.
/// * `is_compact`: Whether the table should be rendered compact, i.e., without horizontal row separators.
/// * `annotate_remote_port`: Whether to append IANA service names to the remote port column (e.g., `443 (https)`).
///
/// # Returns
/// A string containing the Markdown formatted connections table.
pub fn get_connections_table(
    all_connections: &[Connection],
    is_compact: bool,
    annotate_remote_port: bool,
) -> String {
    let column_names: Vec<TableCell> = vec![
        TableCell::header("#", None, Padding::Auto),
        TableCell::header("proto", None, Padding::Auto),
        TableCell::header("local port", None, Padding::Auto),
        TableCell::header("remote address", None, Padding::Auto),
        TableCell::header(
            "remote port",
            if annotate_remote_port {
                Some("service".to_string())
            } else {
                None
            },
            Padding::Auto,
        ),
        TableCell::header("pid", Some("program".to_owned()), Padding::Auto),
        TableCell::header("state", None, Padding::Auto),
    ];

    let mut somo_table: Table = Table::new(column_names.len(), get_row_alignment(is_compact));
    somo_table.add_header(column_names);

    for (idx, connection) in all_connections.iter().enumerate() {
        let add_row_separator = !is_compact || idx + 1 == all_connections.len();

        somo_table.add_row(
            vec![
                TableCell::body(&format!("*{}*", idx + 1), None, Padding::NoPad),
                TableCell::body(&connection.proto, None, Padding::Auto),
                TableCell::body(&connection.local_port, None, Padding::Auto),
                TableCell::body(
                    &format_known_address(&connection.remote_address, &connection.address_type),
                    None,
                    Padding::Auto,
                ),
                TableCell::body(
                    &connection.remote_port,
                    if annotate_remote_port {
                        get_port_annotation(&connection.remote_port, &connection.proto)
                    } else {
                        None
                    },
                    Padding::Auto,
                ),
                TableCell::body(
                    &connection.pid,
                    Some(connection.program.clone()),
                    Padding::Auto,
                ),
                TableCell::body(&connection.state, None, Padding::Auto),
            ],
            add_row_separator,
        )
    }

    somo_table.build()
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
    use super::*;
    use crate::schemas::AddressType;
    use std::net::{Ipv4Addr, Ipv6Addr};

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