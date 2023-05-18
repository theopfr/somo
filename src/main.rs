use termimad::crossterm::style::{Color::*};
use termimad::*;


mod connections;
mod processes;
mod address_checkers;
mod string_utils;
mod interface;
mod second_interface;


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
    
    interface::cli_interface(&all_connections);

    //second_interface::cli_interface(&all_connections);

}
