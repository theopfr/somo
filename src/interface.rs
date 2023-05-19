use termimad::crossterm::style::{Color::*, Attribute::*};
use termimad::*;

use crate::connections;
use crate::address_checkers;
use crate::string_utils;



pub fn cli(all_connections: &Vec<connections::Connection>) {
    // create layout and styles for the table
    let mut skin = MadSkin::default();
    skin.bold.set_fg(Green);
    skin.italic.set_fg(gray(11));
    skin.strikeout = CompoundStyle::new(Some(Red), None, RapidBlink.into());
    skin.paragraph.align = Alignment::Left;
    skin.table.align = Alignment::Center;
    skin.inline_code = CompoundStyle::new(None, None, Encircled.into());
    let (terminal_width, _) = terminal_size();

    // add table headers
    static CENTER_MARKDOWN_ROW: &str = "| :-: | :-: | :-: | :-: | :-: | :-: |\n";
    let mut markdown = format!("\nConnections: **{}**\n", all_connections.len());
    markdown.push_str(CENTER_MARKDOWN_ROW);
    markdown.push_str("| **type** | **local port** | **remote address** | **remote port** | **program***/pid* | **state** |\n");

    // track if IPs were checked in order to print out information
    let mut checked_ip_status: i16 = 0;

    // iterate over all connections to build the table
    for connection in all_connections.into_iter() {
        markdown.push_str(CENTER_MARKDOWN_ROW);
 
        // check if the remote IP is a DNS server
        let remote_address = &connection.remote_address;
        let remote_address_new = &address_checkers::check_if_known(remote_address);
        
        // check if the remote IP is malicious using the AbuseIpDb API
        let (marked_remote_addess, checked) = address_checkers::check_if_malicious(&remote_address_new);
        checked_ip_status = checked;

        // add row with connection information
        markdown.push_str(&format!("| {} | {} | {} | {} | {}*/{}* | {} |\n",
        connection.conn_type, connection.local_port, marked_remote_addess, connection.remote_port, connection.program, connection.pid, connection.state
        ));
    }
    
    // create an empty row that forces the table to fit the terminal with respect to how much space
    // each column should receive based on the max length of each column (in the array below)
    let max_column_spaces: [u16; 6] = [5, 7, 32, 7, 24, 13];
    let terminal_filling_row: String = string_utils::fill_terminal_width(terminal_width, max_column_spaces);
    markdown.push_str(&terminal_filling_row);
    markdown.push_str(CENTER_MARKDOWN_ROW);

    // print information about checking malicious IPs
    markdown.push_str("\n*Info:*");
    if checked_ip_status == 1 {
        markdown.push_str("\n*Successfully checked remote IPs with the AbuseIpDB API.*\n");
    }
    else if checked_ip_status == 0 {
        markdown.push_str("\n*If you want somo to automatically check for malicious IP addresses in your connections, make an account at `www.abuseipdb.com` and add your API key as an env variable: `ABUSEIPDB_API_KEY={your-api-key}`.*\n");
    }
    else if checked_ip_status == -1 {
        markdown.push_str("\n~~A~~ *Couldn't reach the AbuseIpDB API to check for malicious IP address in your connections.*\n");
        markdown.push_str("*Possible problems:*\n");
        markdown.push_str("*1. API down or new non-backward compatible changes -> check if there is a new version of somo avaialble *\n");
        markdown.push_str("*2. wrong or expired API key stored in the `ABUSEIPDB_API_KEY` env variable *\n");
    }

    println!("{}\n", skin.term_text(&markdown));
}