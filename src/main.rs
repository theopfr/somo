use termimad::crossterm::style::{Color::*};
use termimad::*;
use reqwest;

use std::error::Error;

mod connections;
mod processes;
mod address_checkers;
mod string_utils;
mod table;
mod interactive;
mod cli;


fn main() {

    

    let args: cli::FlagValues = cli::cli();

    // example filter option: Some("tcp".to_string())
    let filter_options: connections::FilterOptions = connections::FilterOptions { 
        by_proto: args.proto,
        by_remote_address: args.ip,
        by_remote_port: args.port, 
        by_local_port: args.local_port,
        by_program: args.program,
        by_pid: args.pid,
        by_open: args.open,
        exclude_ipv6: args.exclude_ipv6
    };

    // get running processes
    let all_connections: Vec<connections::Connection> = connections::get_all_connections(&filter_options);
    
    table::get_connections_table(&all_connections, args.check);

    if args.kill {
        cli::interactve_process_kill(&all_connections);
    }

}
