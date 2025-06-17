mod cli;
mod connections;
mod schemas;
mod table;
mod utils;

use clap::CommandFactory;
use cli::{print_completions, Args, CliCommand, Commands};
use schemas::{Connection, FilterOptions};

fn main() {
    let args = match cli::cli() {
        CliCommand::Subcommand(Commands::GenerateCompletions { shell }) => {
            let mut cmd = Args::command();
            print_completions(shell, &mut cmd);
            return;
        }
        CliCommand::Run(flags) => flags,
    };

    let filter_options: FilterOptions = FilterOptions {
        by_proto: cli::resolve_protocols(&args),
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
        table::print_connections_table(&all_connections, args.compact);
    }

    if args.kill {
        cli::interactive_process_kill(&all_connections);
    }
}
