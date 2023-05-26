use termimad::crossterm::style::{Color::*, Attribute::*};
use termimad::*;

use crate::connections;
use crate::address_checkers;
use crate::string_utils;


/// prints all connections in a pretty table
pub fn get_connections_table(all_connections: &Vec<connections::Connection>) {

    // create layout and styles for the table
    let mut skin = MadSkin::default();
    skin.bold.set_fg(Cyan);
    skin.italic.set_fg(gray(11));
    skin.strikeout = CompoundStyle::new(Some(Red), None, RapidBlink.into());
    skin.paragraph.align = Alignment::Left;
    skin.table.align = Alignment::Center;
    skin.inline_code = CompoundStyle::new(Some(Yellow), None, Encircled.into());
    let (terminal_width, _) = terminal_size();

    // print amount of connections (after filter)
    string_utils::pretty_print_info(&format!("Connections: **{}**", all_connections.len()));

    // add table headers
    static CENTER_MARKDOWN_ROW: &str = "| :-: | :-: | :-: | :-: | :-: | :-: | :-: |\n";
    let mut markdown = CENTER_MARKDOWN_ROW.to_string();
    markdown.push_str("| **#** | **proto** | **local port** | **remote address** | **remote port** | **program***/pid* | **state** |\n");

    // iterate over all connections to build the table
    for (idx, connection) in all_connections.into_iter().enumerate() {
        markdown.push_str(CENTER_MARKDOWN_ROW);
 
        // check if the remote IP is a DNS server
        let remote_address = &connection.remote_address;

        let ip_code: u8 = address_checkers::check_for_known_ip(remote_address);
        let remote_address_known: String;
        
        // gray out 0.0.0.0 and localhost IP addresses since they are less important
        // ip_code = 1 represents "unspecified"/0.0.0.0, ip_code = 2 represents localhost
        if ip_code == 1 {
            remote_address_known = format!("*{}*", &remote_address);
        }
        else if ip_code == 2 {
            remote_address_known = format!("*{} localhost*", &remote_address);
        }
        else {
            remote_address_known = remote_address.to_string();
        }

        // add abusiveness information to remote address
        let checked_remote_address: String;
        if connection.abuse_score >= Some(50) {
            checked_remote_address = format!("{} ~~high abuse score: {}~~", &remote_address_known, &connection.abuse_score.unwrap());
        }
        else if connection.abuse_score > Some(25) {
            checked_remote_address = format!("{} `moderate abuse score: {}`", &remote_address_known, &connection.abuse_score.unwrap());
        }
        else if connection.abuse_score >= Some(1) {
            checked_remote_address = format!("{} *low abuse score: {}*", &remote_address_known, &connection.abuse_score.unwrap());
        }
        else if connection.abuse_score == Some(0) {
            checked_remote_address = format!("{} **✓**", &remote_address_known);
        }
        else {
            checked_remote_address = (&remote_address_known).to_string();
        }

        // add row with connection information
        markdown.push_str(&format!("| *{}* | {} | {} | {} | {} | {}*/{}* | {} |\n",
            idx + 1, connection.proto, connection.local_port,  &checked_remote_address, connection.remote_port, connection.program, connection.pid, connection.state
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