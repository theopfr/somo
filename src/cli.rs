use clap::Parser;
use inquire::Select;
use inquire::InquireError;
use std::{process};
use std::string::String;
use crate::connections;
use crate::string_utils;

/// Used for parsing all the flags values provided by the user in the CLI.
#[derive(Debug)]
pub struct FlagValues {
    pub check: bool,
    pub kill: bool,
    pub proto: Option<String>,
    pub ip: Option<String>,
    pub port: Option<String>,
    pub local_port: Option<String>,
    pub program: Option<String>,
    pub pid: Option<String>,
    pub open: bool,
    pub exclude_ipv6: bool
}


/// Represents all possible flags which can be provided by the user in the CLI.
#[derive(Parser, Debug)] 
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'c', long, default_value_t = false)]
    check: bool,

    #[arg(short = 'k', long, default_value = None)]
    kill: bool,

    #[arg(long, default_value = None)]
    proto: Option<String>,

    #[arg(long, default_value = None)]
    ip: Option<String>,

    #[arg(short = 'p', long, default_value = None)]
    port: Option<String>,

    #[arg(long, default_value = None)]
    local_port: Option<String>,

    #[arg(long, default_value = None)]
    program: Option<String>,

    #[arg(long, default_value = None)]
    pid: Option<String>,

    #[arg(short = 'o', long, default_value_t = false)]
    open: bool,

    #[arg(short = 'e', long, default_value_t = false)]
    exclude_ipv6: bool,
}


/// Gets all flag values provided by the user in the CLI using the "clap" crate.
/// 
/// # Arguments
/// None
/// 
/// # Returns
/// A struct containing all the flag values.
pub fn cli() -> FlagValues {
    let args = Args::parse();

    FlagValues {
        check: args.check,
        kill: args.kill,
        proto: args.proto,
        ip: args.ip,
        program: args.program,
        port: args.port,
        local_port: args.local_port,
        pid: args.pid,
        open: args.open,
        exclude_ipv6: args.exclude_ipv6
    }
}


/// Kills a process by its PID.
/// 
/// # Argument
/// * `pid`: The PID value as a string.
/// 
/// # Returns
/// None
pub fn kill_process(pid: &String) {
    let output = process::Command::new("kill")
        .arg(pid)
        .output()
        .unwrap_or_else(|_| panic!("Failed to kill process with PID {}", pid));

    if output.status.success() {
        //println!("Killed process with PID {}.", pid);
        string_utils::pretty_print_info(&format!("Killed process with PID {}.", pid));
    }
    else {
        println!("Failed to kill process, try running");
        string_utils::pretty_print_error("Couldn't kill process! Try again using sudo: 'sudo $(where somo)'.");
    }
}


/// Starts an interactive selection process in the console for choosing a process to kill using the "inquire" crate.
/// 
/// # Argument
/// * `connections`: A vector containing all connections which themselves contain a PID value.
/// 
/// # Returns
/// None
pub fn interactve_process_kill(connections: &Vec<connections::Connection>) {
    let selection: Result<u32, InquireError> = Select::new("Which process to kill (search or type index)?", (1..=connections.len() as u32).collect()).prompt();

    match selection {
        Ok(choice) => {
            let pid: &String = &connections[choice as usize - 1].pid;
            kill_process(pid);
        },
        Err(_) => println!("Couldn't find process."),
    }
}