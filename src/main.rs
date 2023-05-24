mod connections;
mod address_checkers;
mod string_utils;
mod table;
mod cli;


fn main() {

    let mut args: cli::FlagValues = cli::cli();

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

    if args.check {
        println!("Checking IPs using AbuseIPDB.com...");
        let abuse_result = address_checkers::get_ip_audit(&("127.0.0.1cc".to_string()), true).unwrap();
        match abuse_result {
            Some(_) => { }
            None => {
                println!("Cancelling IP abuse check.");
                args.check = false;
            }
        } 
    }

    // get running processes
    let all_connections: Vec<connections::Connection> = connections::get_all_connections(&filter_options, args.check);
    
    table::get_connections_table(&all_connections, args.check);

    if args.kill {
        cli::interactve_process_kill(&all_connections);
    }

}
