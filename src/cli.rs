use clap::Parser;
use inquire::Select;
use inquire::InquireError;
use std::{process};
use std::string::String;

use crate::string_utils;
use crate::connections;

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


/// get flags
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


// handle command line flags
pub fn cli() -> FlagValues {
    let args = Args::parse();

    println!("{:?}", args.kill);

    let flag_values = FlagValues {
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
    };

    return flag_values;
}


// 
pub fn interactve_process_kill(connections: &Vec<connections::Connection>) {
    let selection: Result<u32, InquireError> = Select::new("Which process to kill (search or type index)?", (1..=connections.len() as u32).collect()).prompt();

    match selection {
        Ok(choice) => {
            let output = process::Command::new("kill")
                .arg(&connections[choice as usize - 1].pid)
                .output()
                .expect(&format!("Failed to kill process with PID {}", choice));

            if output.status.success() {
                println!("Killed process with PID {}.", choice);
            }
        },
        Err(_) => println!("Couldn't find process."),
    }

}