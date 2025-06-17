mod cli;
mod connections;
mod schemas;
mod table;
mod utils;

use schemas::{Connection, FilterOptions, Protocols};
use utils::{TCP, UDP};

fn main() {
    let args: cli::Flags = cli::cli();

    let proto = {
        let mut proto = Protocols::default();
        if args.tcp || args.udp {
            proto.tcp = args.tcp;
            proto.udp = args.udp;
        } else if let Some(p) = args.proto {
            // support the deprecated "--proto" argument
            match p.to_lowercase().as_str() {
                TCP => {
                    proto.tcp = true;
                }
                UDP => {
                    proto.udp = true;
                }
                _ => {}
            }
        } else {
            // if neither is set, use both
            proto.tcp = true;
            proto.udp = true;
        }
        proto
    };
    let filter_options: FilterOptions = FilterOptions {
        by_proto: proto,
        by_remote_address: args.ip,
        by_remote_port: args.remote_port,
        by_local_port: args.port,
        by_program: args.program,
        by_pid: args.pid,
        by_open: args.open,
        by_listen: args.listen,
        exclude_ipv6: args.exclude_ipv6,
    };

    let all_connections: Vec<Connection> = connections::get_all_connections(&filter_options);

    if args.json {
        let result = table::get_connections_json(&all_connections);
        println!("{}", result);
    } else if args.format.is_some() {
        let result = table::get_connections_formatted(&all_connections, &args.format.unwrap());
        println!("{}", result);
    } else {
        table::print_connections_table(&all_connections);
    }

    if args.kill {
        cli::interactve_process_kill(&all_connections);
    }
}
