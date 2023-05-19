use termimad::crossterm::style::{Color::*};
use termimad::*;
use reqwest;

use std::error::Error;

mod connections;
mod processes;
mod address_checkers;
mod string_utils;
mod interface;


fn main() {
    // example filter option: Some("tcp".to_string())
    let filter_options: connections::FilterOptions = connections::FilterOptions { 
        by_conn_type: None,
        by_program: None,
        by_pid: None,
        by_remote_address: None, 
        by_remote_port: None, 
        exclude_ipv6: false
    };

    // get running processes
    let all_connections: Vec<connections::Connection> = connections::get_all_connections(&filter_options);
    
    // let _ = address_checkers::get_ip_audit();

    interface::cli(&all_connections);
}
