use termimad::crossterm::style::{Attribute::*, Color::*};
use termimad::*;

use crate::schemas::{AddressType, Connection};
use crate::utils;

/// Uses the termimad crate to create a custom appearence for Mardown text in the console.
///
/// # Appearence
/// * **bold** text -> bold and cyan
/// * *italic* text -> italiv and light gray
/// * ~~strikeout~~ text -> not striked out, red and blinking
/// * `inline code` text -> not code formatted, yellow
///
/// # Arguments
/// None
///
/// # Returns
/// A custom markdow "skin".
fn create_table_style() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.bold.set_fg(Cyan);
    skin.italic.set_fg(gray(11));
    skin.strikeout = CompoundStyle::new(Some(Red), None, RapidBlink.into());
    skin.paragraph.align = Alignment::Left;
    skin.table.align = Alignment::Center;
    skin.inline_code = CompoundStyle::new(Some(Yellow), None, Encircled.into());

    return skin;
}

/// Marks localhost and unspecified IP addresses (ie. 0.0.0.0) using Markdown formatting.

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
/// A Markdown table row string in which each column is filled with as much empty characters needed to fit in content and as well fill out the terminal width.
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

    return row;
}

/// Prints all current connections in a pretty Markdown table.
///
/// # Arguments
/// * `all_connections`: A list containing all current connections as a `Connection` struct.
///
/// # Returns
/// None
pub fn print_connections_table(all_connections: &Vec<Connection>) {
    let skin: MadSkin = create_table_style();
    let (terminal_width, _) = terminal_size();

    // Add table headers
    static CENTER_MARKDOWN_ROW: &str = "| :-: | :-: | :-: | :-: | :-: | :-: | :-: |\n";
    let mut markdown = CENTER_MARKDOWN_ROW.to_string();
    markdown.push_str("| **#** | **proto** | **local port** | **remote address** | **remote port** | **pid** *program* | **state** |\n");

    // iterate over all connections to build the table
    for (idx, connection) in all_connections.iter().enumerate() {
        markdown.push_str(CENTER_MARKDOWN_ROW);

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
    }

    // Create an empty row that forces the table to fit the terminal with respect to how much space ...
    // ... each column should receive based on the max length of each column (in the array below)
    let max_column_spaces: [u16; 7] = [5, 8, 8, 28, 7, 24, 13];
    let terminal_filling_row: String = fill_terminal_width(terminal_width, max_column_spaces);
    markdown.push_str(&terminal_filling_row);
    markdown.push_str(CENTER_MARKDOWN_ROW);

    println!("{}", skin.term_text(&markdown));

    utils::pretty_print_info(&format!("**{} Connections**", all_connections.len()));
}

#[cfg(test)]
mod tests {
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
}
