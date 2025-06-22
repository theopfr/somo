use handlebars::Handlebars;
use termimad::crossterm::style::{Attribute::*, Color::*};
use termimad::*;

use crate::schemas::{AddressType, Connection};
use crate::{soutln, utils};

/// Uses the termimad crate to create a custom appearance for Markdown text in the console.
///
/// # Appearance
/// * **bold** text -> bold and cyan
/// * *italic* text -> italic and light gray
/// * ~~strikeout~~ text -> not struck out, red and blinking
/// * `inline code` text -> not code formatted, yellow
///
/// # Arguments
/// None
///
/// # Returns
/// A custom markdown "skin".
fn create_table_style(use_compact_mode: bool) -> MadSkin {
    let mut skin = MadSkin::default();
    skin.bold.set_fg(Cyan);
    skin.italic.set_fg(gray(11));
    skin.strikeout = CompoundStyle::new(Some(Red), None, RapidBlink.into());
    skin.paragraph.align = Alignment::Left;
    skin.table.align = if use_compact_mode {
        Alignment::Left
    } else {
        Alignment::Center
    };
    skin.inline_code = CompoundStyle::new(Some(Yellow), None, Encircled.into());

    skin
}

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
fn format_known_address(remote_address: &String, address_type: &AddressType) -> String {
    match address_type {
        AddressType::Unspecified => {
            format!("*{}*", remote_address)
        }
        AddressType::Localhost => {
            format!("*{} localhost*", remote_address)
        }
        AddressType::Extern => remote_address.to_string(),
    }
}

/// Creates a Markdown table row with just empty characters with the width of the terminal window.
///
/// # Argument
/// * `terminal_width`: The current width of the terminal.
/// * `max_column_spaces`: An array in which the values represent the max-width of each of the 7 Markdown table rows.
///
/// # Returns
/// A Markdown table row string in which each column is filled with as many empty characters needed to fit in content and as well fill out the terminal width.
fn fill_terminal_width(terminal_width: u16, max_column_spaces: [u16; 7]) -> String {
    let total_column_spaces: u16 = max_column_spaces.iter().sum();

    let calculate_column_width = |column_space: u16| {
        (column_space as f64 / total_column_spaces as f64) * (terminal_width as f64)
    };
    let empty_character = "\u{2800}";

    let mut row: String = String::new();
    for &max_column_space in &max_column_spaces {
        row.push_str(&format!(
            "| {} ",
            empty_character.repeat(calculate_column_width(max_column_space) as usize)
        ));
    }
    row.push_str("|\n");

    row
}

/// Prints all current connections in a pretty Markdown table.
///
/// # Arguments
/// * `all_connections`: A list containing all current connections as a `Connection` struct.
///
/// # Returns
/// None
pub fn print_connections_table(all_connections: &[Connection], use_compact_mode: bool) {
    let skin: MadSkin = create_table_style(use_compact_mode);
    let (terminal_width, _) = terminal_size();

    // Add table headers
    static CENTER_MARKDOWN_ROW: &str = "| :-: | :-: | :-: | :-: | :-: | :-: | :-: |\n";
    let mut markdown = CENTER_MARKDOWN_ROW.to_string();
    markdown.push_str("| **#** | **proto** | **local port** | **remote address** | **remote port** | **pid** *program* | **state** |\n");
    markdown.push_str(CENTER_MARKDOWN_ROW);

    // iterate over all connections to build the table
    for (idx, connection) in all_connections.iter().enumerate() {
        let formatted_remote_address: String =
            format_known_address(&connection.remote_address, &connection.address_type);

        markdown.push_str(&format!(
            "| *{}* | {} | {} | {} | {} | {} *{}* | {} |\n",
            idx + 1,
            connection.proto,
            connection.local_port,
            &formatted_remote_address,
            connection.remote_port,
            connection.pid,
            connection.program,
            connection.state
        ));

        if !use_compact_mode && idx < all_connections.len() - 1 {
            markdown.push_str(CENTER_MARKDOWN_ROW);
        }
    }

    if !use_compact_mode {
        // Create an empty row that forces the table to fit the terminal with respect to how much space ...
        // ... each column should receive based on the max length of each column (in the array below)
        let max_column_spaces: [u16; 7] = [5, 8, 8, 28, 7, 24, 13];
        let terminal_filling_row: String = fill_terminal_width(terminal_width, max_column_spaces);
        markdown.push_str(&terminal_filling_row);
    }

    markdown.push_str(CENTER_MARKDOWN_ROW);

    soutln!("{}", skin.term_text(&markdown));
    utils::pretty_print_info(&format!("**{} Connections**", all_connections.len()))
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
    let _ = registry.register_template_string("connection_template", template_string);

    let mut rendered_lines = Vec::new();

    for connection in all_connections {
        let json_value = serde_json::to_value(connection).unwrap();
        let rendered_line = registry.render("connection_template", &json_value);

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
    fn test_fill_terminal_width() {
        let row = fill_terminal_width(80, [5, 8, 8, 28, 7, 24, 13]);
        let columns = row.matches('|').count();
        assert_eq!(columns, 8); // 7 columns + final pipe
    }

    #[test]
    fn test_table_style_alignment() {
        let compact_skin = create_table_style(true);
        assert_eq!(compact_skin.table.align, Alignment::Left);

        let non_compact_skin = create_table_style(false);
        assert_eq!(non_compact_skin.table.align, Alignment::Center);
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
