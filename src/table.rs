use termimad::crossterm::style::{Color::*, Attribute::*};
use termimad::*;

use crate::connections;
use crate::address_checkers;
use crate::string_utils;


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

    skin
}


/// Adds abusiveness information to the remote address based on the abuse score.
/// 
/// * `abuse_score` >= 50 -> high abuse confidence
/// * `abuse_score` >= 25 -> moderate abuse confidence
/// * `abuse_score` >=  1 -> low abuse confidence
/// * `abuse_score` ==  0 -> no abuse danger
/// 
/// # Arguments
/// * `remote_address`: The remote address checked for abusivness.
/// * `abuse_score`: The abuse score delivered by AbuseIPDB.com
/// 
/// 
/// # Example
/// ```
/// let address = "127.0.0.1".to_string();
/// let score = Some(75);
/// let formatted = format_abuse_checked_address(&address, score);
/// assert_eq!(formatted, "127.0.0.1 ~~high abuse score: 75~~"); 
/// ```
/// 
/// # Returns
/// A Markdown formatted string containing the remote address and abusiveness information.
fn format_abuse_checked_address(remote_address: &String, abuse_score: Option<i64>) -> String {
    let checked_remote_address: String;
    if abuse_score >= Some(50) {
        checked_remote_address = format!("{} ~~high abuse score: {}~~", remote_address, abuse_score.unwrap());
    }
    else if abuse_score > Some(25) {
        checked_remote_address = format!("{} `moderate abuse score: {}`", remote_address, abuse_score.unwrap());
    }
    else if abuse_score >= Some(1) {
        checked_remote_address = format!("{} *low abuse score: {}*", remote_address, abuse_score.unwrap());
    }
    else if abuse_score == Some(0) {
        checked_remote_address = format!("{} **✓**", remote_address);
    }
    else {
        checked_remote_address = (&remote_address).to_string();
    }

    checked_remote_address
}


/// Marks localhost and unspecified IP addresses (ie. 0.0.0.0) using Markdown formatting.

/// * `address_type` == Localhost -> *italic* + "localhost" 
/// * `address_type` == Unspecified -> *italic*
/// * `address_type` == Extern -> not formatted
/// 
/// # Arguments
/// * `remote_address`: The remote address.
/// * `address_type`: The address type as an IPType enum.
/// 
/// # Example
/// ```
/// let address = "127.0.0.1".to_string();
/// let address_type = address_checkers::IPType::Localhost;
/// let formatted = format_known_address(&address, &address_type);
/// assert_eq!(formatted, "*127.0.0.1 localhost*"); 
/// ```
/// 
/// # Returns
/// A Markdown formatted string based on the address-type.
fn format_known_address(remote_address: &String, address_type: &address_checkers::IPType) -> String {
    match address_type {
        address_checkers::IPType::Unspecified => {
            format!("*{}*", remote_address)
        }
        address_checkers::IPType::Localhost => {
            format!("*{} localhost*", remote_address)
        }
        address_checkers::IPType::Extern => {
            remote_address.to_string()
        }
    }
}

/// Prints all current connections in a pretty Markdown table.
/// 
/// # Arguments
/// * `all_connections`: A list containing all current connections as a `Connection` struct.
/// 
/// # Returns
/// None
pub fn get_connections_table(all_connections: &Vec<connections::Connection>) {
    let skin: MadSkin = create_table_style();
    let (terminal_width, _) = terminal_size();

    // print amount of connections (after filter)
    string_utils::pretty_print_info(&format!("Connections: **{}**", all_connections.len()));

    // add table headers
    static CENTER_MARKDOWN_ROW: &str = "| :-: | :-: | :-: | :-: | :-: | :-: | :-: |\n";
    let mut markdown = CENTER_MARKDOWN_ROW.to_string();
    markdown.push_str("| **#** | **proto** | **local port** | **remote address** | **remote port** | **program***/pid* | **state** |\n");

    // iterate over all connections to build the table
    for (idx, connection) in all_connections.iter().enumerate() {
        markdown.push_str(CENTER_MARKDOWN_ROW);
 
        // check if the remote IP is a DNS server
        let remote_address = &connection.remote_address;

        // add abusiveness information to remote address
        let mut formatted_remote_address: String = format_known_address(remote_address, &connection.address_type);
        formatted_remote_address = format_abuse_checked_address(&formatted_remote_address, connection.abuse_score);

        // add row with connection information
        markdown.push_str(&format!("| *{}* | {} | {} | {} | {} | {}*/{}* | {} |\n",
            idx + 1, connection.proto, connection.local_port,  &formatted_remote_address, connection.remote_port, connection.program, connection.pid, connection.state
        ));
    }

    // create an empty row that forces the table to fit the terminal with respect to how much space
    // each column should receive based on the max length of each column (in the array below)
    let max_column_spaces: [u16; 7] = [5, 5, 7, 32, 7, 24, 13];
    let terminal_filling_row: String = string_utils::fill_terminal_width(terminal_width, max_column_spaces);
    markdown.push_str(&terminal_filling_row);
    markdown.push_str(CENTER_MARKDOWN_ROW);

    println!("{}\n", skin.term_text(&markdown));
}