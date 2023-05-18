use termimad::crossterm::style::{Color::*};
use termimad::*;

use crate::connections;
use crate::address_checkers;
use crate::string_utils;

pub fn cli_interface(all_connections: &Vec<connections::Connection>) {

    let mut skin = MadSkin::default();
    skin.bold.set_fg(Green);
    skin.italic.set_fg(gray(11));
    skin.strikeout.set_fg(Red);
    skin.paragraph.align = Alignment::Left;
    skin.table.align = Alignment::Left;
    let (width, _) = terminal_size();

    // add table headers
    static CENTER_MARKDOWN_ROW: &str = "| :-: | :-: | :-: | :-: | :-: | :-: |\n";
    let mut markdown = format!("\nConnections: **{}**\n", all_connections.len());
    markdown.push_str(CENTER_MARKDOWN_ROW);
    markdown.push_str("| **type** | **local port** | **remote address** | **remote port** | **program***/pid* | **state** |\n");

    let mut checked_ip_status: i16 = 0;

    // iterate over all connections to build the table
    for connection in all_connections.into_iter() {
        markdown.push_str(CENTER_MARKDOWN_ROW);
 
        // check if the remote IP is a DNS server
        let remote_address = &connection.remote_address;
        let mut remote_address_new = &address_checkers::check_if_known(remote_address);
        
        // check if the remote IP is malicious using the AbuseIpDb API
        let (is_malicious, checked) = address_checkers::check_if_malicious(&remote_address);
        if is_malicious {
            remote_address_new = &format!("{} ~~A~~ malicious", remote_address_new)
        }
        checked_ip_status = checked;

        // add row with connection information
        markdown.push_str(&format!("| {} | {} | {} | {} | {}*/{}* | {} |\n",
        connection.conn_type, connection.local_port, remote_address, connection.remote_port, connection.program, connection.pid, connection.state
        ));
    }
    
    // format columns and rows to make the table responsive and fill the terminal width
    static TYPE_WIDTH: u16 = 5;
    static PORT_WIDTH: u16 = 7;
    static TABLE_BORDER_SPACE: u16 = 7;

    let empty_character: String = string_utils::str_from_bytes(&[0xE2, 0xA0, 0x80]);
    let spacing = empty_character.repeat(usize::from(width - TABLE_BORDER_SPACE - TYPE_WIDTH - PORT_WIDTH) / 3);
    markdown.push_str(CENTER_MARKDOWN_ROW);
    markdown.push_str(&format!(
        "| {} | {} | {} | {} | {} | {} |\n",
        empty_character.repeat(usize::from(TYPE_WIDTH)), empty_character.repeat(usize::from(PORT_WIDTH)), spacing, empty_character.repeat(usize::from(PORT_WIDTH)), spacing, spacing
    ));

    // print information about checking malicious IPs
    if checked_ip_status == 1 {
        markdown.push_str("\n*Successfully checked remote IPs with the AbuseIpDB API.*\n");
    }
    else if checked_ip_status == 0 {
        markdown.push_str("\n~~A~~ *If you want somo to automatically check for malicious IP addresses in your connections, go make an account at `www.abuseipdb.com`.*\n");
        markdown.push_str("*Then add your API key as an env variable: `ABUSEIPDB_API_KEY={your-api-key}.`*\n");
    }
    else if checked_ip_status == -1 {
        markdown.push_str("\n~~A~~ *Couldn't reach the AbuseIpDB API to check for malicious IP address in your connections.*\n");
        markdown.push_str("*Possible problems:*\n");
        markdown.push_str("*1. API down or new non-backward compatible changes -> check if there is a new version of somo avaialble *\n");
        markdown.push_str("*2. wrong or expired API key stored in the `ABUSEIPDB_API_KEY` env variable *\n");
    }

    println!("{}\n", skin.term_text(&markdown));
}