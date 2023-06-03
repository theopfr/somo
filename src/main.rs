mod connections;
mod address_checkers;
mod string_utils;
mod table;
mod cli;


#[tokio::main]
async fn main() {

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

    // sanity-check if the AbuseIPDB is usable, if not: don't check remote addresses and print an error
    if args.check {
        string_utils::pretty_print_info("Checking IPs using AbuseIPDB.com...");
        let abuse_result = address_checkers::check_address_for_abuse(&("127.0.0.1".to_string()), true).await.unwrap();
        match abuse_result {
            Some(_) => { }
            None => {
                string_utils::pretty_print_error("Cancelling check for malicious IPs.");
                args.check = false;
            }
        } 
    }

    // get running processes
    let all_connections: Vec<connections::Connection> = connections::get_all_connections(&filter_options, args.check).await;
    
    table::get_connections_table(&all_connections);

    if args.kill {
        cli::interactve_process_kill(&all_connections);
    }

}