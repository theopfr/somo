use procfs;
use procfs::process::FDTarget;
use procfs::process::Stat;
use std::collections::HashMap;
use termimad::crossterm::style::{Attribute::*, Color::*};
use termimad::*;

mod string_utils;

fn get_processes() -> HashMap<u64, Stat> {
    /* gets all running processes on the system */

    let all_procs = procfs::process::all_processes().unwrap();

    let mut map: HashMap<u64, Stat> = HashMap::new();
    for p in all_procs {
        let process = p.unwrap();
        if let (Ok(stat), Ok(fds)) = (process.stat(), process.fd()) {
            for fd in fds {
                if let FDTarget::Socket(inode) = fd.unwrap().target {
                    map.insert(inode, stat.clone());
                }
            }
        }
    }
    return map;
}


fn check_known_ip(remote_ip: String) -> String {
    /* check if an IP corresponds to a DNS server */

    if remote_ip == "0.0.0.0" || remote_ip == "[::]" {
        return format!("{} *unspecified*", remote_ip);
    }
    else if remote_ip == "127.0.0.1" || remote_ip == "[::1]" {
        return format!("{} *localhost*", remote_ip);
    }
    return remote_ip;
}


fn check_malicious(remote_ip: &String) -> (bool, i16) {
    /* check if an IP corresponds to a DNS server */
    let mut malicious: bool = false;
    let mut checked_ip_status: i16 = 0;

    if remote_ip == "185.230.162.220" {
        malicious = true;
        checked_ip_status = 1;
    }
    
    return (malicious, checked_ip_status);
}


fn main() {
    // get running processes
    let process_map = get_processes();

    // initialize and customize termimad environment to create a nice looking table
    let mut skin = MadSkin::default();
    skin.bold.set_fg(Green);
    skin.italic.set_fg(gray(11));
    skin.strikeout.set_fg(Red);
    skin.paragraph.align = Alignment::Left;
    skin.table.align = Alignment::Left;
    let (width, _) = terminal_size();

    static CENTER_MARKDOWN_ROW: &str = "| :-: | :-: | :-: | :-: | :-: |\n";

    // add table headers
    let mut markdown = format!("\nConnections: **{}**\n", width);
    markdown.push_str(CENTER_MARKDOWN_ROW);
    markdown.push_str("| **local port** | **remote address** | **remote port** | **program***/pid* | **state** |\n");

    // get all TCP connections
    let tcp = procfs::net::tcp().unwrap();
    let tcp6 = procfs::net::tcp6().unwrap();

    // get all UDP connections
    let udp = procfs::net::udp().unwrap();
    let udp6 = procfs::net::udp6().unwrap();

    let mut checked_ip_status: i16 = 0;

    // iterate over all connections to build the table
    for entry in udp.into_iter().chain(udp6) {
        markdown.push_str(CENTER_MARKDOWN_ROW);

        // split <ip>:<port>
        let (_, local_port) = string_utils::get_address_parts(&format!("{}", entry.local_address));
        let (mut remote_ip, remote_port) = string_utils::get_address_parts(&format!("{}", entry.remote_address));
        let state = format!("{:?}", entry.state).to_ascii_lowercase();
        
        // check if the remote IP is a DNS server
        remote_ip = check_known_ip(remote_ip);
        
        // check if the remote IP is malicious using the AbuseIpDb API
        let (is_malicious, checked) = check_malicious(&remote_ip);
        if is_malicious {
            remote_ip = format!("{} ~~A~~ malicious", remote_ip)
        }
        checked_ip_status = checked;

        // add row with connection information
        if let Some(stat) = process_map.get(&entry.inode) {
            markdown.push_str(&format!(
                "| {} | {} | {} | {}*/{}* | {} |\n",
                local_port, remote_ip, remote_port, stat.comm, stat.pid, state
            ));
        } else {
            markdown.push_str(&format!(
                "| {} | {} | {} | *-/-* | {} |\n",
                local_port, remote_ip, remote_port, state
            ));
        }
    }

    // add a row that forces the table to fill the terminal width
    let empty_character: String = string_utils::str_from_bytes(&[0xE2, 0xA0, 0x80]);
    let spacing = empty_character.repeat(usize::from(width - 6) / 5);
    markdown.push_str(CENTER_MARKDOWN_ROW);
    markdown.push_str(&format!(
        "| {} | {} | {} | {} | {} |\n",
        spacing, spacing, spacing, spacing, spacing
    ));

    // print information about checking malicious IPs
    if checked_ip_status == 1 {
        markdown.push_str("\n*Successfully checked remote IPs with the AbuseIpDB API.*\n");
    }
    else if checked_ip_status == 0 {
        markdown.push_str("\n~~A~~ *If you want SRC to automatically check for malicious IP addresses in your connections, go make an account at `www.abuseipdb.com`.*\n");
        markdown.push_str("*Then add your API key as an env variable: `ABUSEIPDB_API_KEY={your-api-key}.`*\n");
    }
    else if checked_ip_status == -1 {
        markdown.push_str("\n~~A~~ *Couldn't reach the AbuseIpDB API to check for malicious IP address in your connections.*\n");
        markdown.push_str("*Possible problems:*\n");
        markdown.push_str("*1. API down or new non-backward compatible changes -> check if there is a new version of SRC avaialble *\n");
        markdown.push_str("*2. wrong or expired API key stored in the `ABUSEIPDB_API_KEY` env variable *\n");
    }

    println!("{}\n", skin.term_text(&markdown));
}
