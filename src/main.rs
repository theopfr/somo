mod cli;
mod connections;
mod macros;
mod markdown;
mod schemas;
mod utils;
mod view;

use clap::CommandFactory;
use cli::{print_completions, Args, CliCommand, Commands};
use schemas::{Connection, FilterOptions};

use crate::cli::sort_connections;

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
        by_established: args.established,
        exclude_ipv6: args.exclude_ipv6,
    };

    let mut all_connections: Vec<Connection> = connections::get_all_connections(&filter_options);

    if let Some(sort) = args.sort {
        sort_connections(&mut all_connections, sort);
    }

    if args.reverse {
        all_connections.reverse();
    }

    if args.json {
        let result = view::get_connections_json(&all_connections);
        soutln!("{}", result);
    } else if args.format.is_some() {
        let result = view::get_connections_formatted(&all_connections, &args.format.unwrap());
        soutln!("{}", result);
    } else {
        let result = view::get_connections_table(&all_connections, args.compact);
        sout!("{}", result);
        utils::pretty_print_info(&format!("{} Connections", all_connections.len()));
    }

    if args.kill {
        cli::interactive_process_kill(&all_connections);
    }
}
