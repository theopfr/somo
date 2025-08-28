use clap::Parser;
use inquire::InquireError;
use inquire::Select;
use std::process;
use std::string::String;

use crate::utils;
use crate::schemas::Connection;

/// Used for parsing all the flags values provided by the user in the CLI.
#[derive(Debug)]
pub struct Flags {
    pub kill: bool,
    pub proto: Option<String>,
    pub ip: Option<String>,
    pub remote_port: Option<String>,
    pub port: Option<String>,
    pub program: Option<String>,
    pub pid: Option<String>,
    pub open: bool,
    pub listen: bool,
    pub ipv4_only: bool,
    pub ipv6_only: bool,
    pub exclude_ipv6: bool,
}

/// Represents all possible flags which can be provided by the user in the CLI.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'k', long, default_value = None)]
    kill: bool,

    #[arg(long, default_value = None)]
    proto: Option<String>,

    #[arg(long, default_value = None)]
    ip: Option<String>,

    #[arg(long, default_value = None)]
    remote_port: Option<String>,

    #[arg(short = 'p', long, default_value = None)]
    port: Option<String>,

    #[arg(long, default_value = None)]
    program: Option<String>,

    #[arg(long, default_value = None)]
    pid: Option<String>,

    #[arg(short = 'o', long, default_value_t = false)]
    open: bool,

    #[arg(short = 'l', long, default_value_t = false)]
    listen: bool,

    #[arg(short = '4', long, default_value_t = false)]
    ipv4_only: bool,

    #[arg(short = '6', long, default_value_t = false)]
    ipv6_only: bool,

    #[arg(long, default_value_t = false)]
    exclude_ipv6: bool,
}

/// Gets all flag values provided by the user in the CLI using the "clap" crate.
///
/// # Arguments
/// None
///
/// # Returns
/// A struct containing all the flag values.
pub fn cli() -> Flags {
    let args = Args::parse();

    return Flags {
        kill: args.kill,
        proto: args.proto,
        ip: args.ip,
        program: args.program,
        remote_port: args.remote_port,
        port: args.port,
        pid: args.pid,
        open: args.open,
        listen: args.listen,
        ipv4_only: args.ipv4_only,
        ipv6_only: args.ipv6_only,
        exclude_ipv6: args.exclude_ipv6,
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
        utils::pretty_print_info(&format!("Killed process with PID {}.", pid));
    } else {
        println!("Failed to kill process, try running");
        utils::pretty_print_error(
            "Couldn't kill process! Try again using sudo.",
        );
    }
}

/// Starts an interactive selection process in the console for choosing a process to kill using the "inquire" crate.
///
/// # Argument
/// * `connections`: A vector containing all connections which themselves contain a PID value.
///
/// # Returns
/// None
pub fn interactve_process_kill(connections: &Vec<Connection>) {
    let selection: Result<u32, InquireError> = Select::new(
        "Which process to kill (search or type index)?",
        (1..=connections.len() as u32).collect(),
    )
    .prompt();

    match selection {
        Ok(choice) => {
            let pid: &String = &connections[choice as usize - 1].pid;
            kill_process(pid);
        }
        Err(_) => println!("Couldn't find process."),
    }
}



#[cfg(test)]
mod tests {
    use super::Args;
    use clap::Parser;

    #[test]
    fn test_all_flags_parsing() {
        let args = Args::parse_from(&[
            "test-bin",
            "-k",
            "--proto", "udp",
            "--ip", "192.168.0.1",
            "--remote-port", "53",
            "-p", "8080",
            "--program", "nginx",
            "--pid", "1234",
            "-o",
            "-l",
            "--exclude-ipv6",
        ]);

        assert!(args.kill);
        assert_eq!(args.proto.as_deref(), Some("udp"));
        assert_eq!(args.ip.as_deref(), Some("192.168.0.1"));
        assert_eq!(args.remote_port.as_deref(), Some("53"));
        assert_eq!(args.port.as_deref(), Some("8080"));
        assert_eq!(args.program.as_deref(), Some("nginx"));
        assert_eq!(args.pid.as_deref(), Some("1234"));
        assert!(args.open);
        assert!(args.listen);
        assert!(args.exclude_ipv6);
    }

    #[test]
    fn test_default_values() {
        let args = Args::parse_from(&["test-bin"]);

        assert!(!args.kill);
        assert!(args.proto.is_none());
        assert!(args.ip.is_none());
        assert!(args.remote_port.is_none());
        assert!(args.port.is_none());
        assert!(args.program.is_none());
        assert!(args.pid.is_none());
        assert!(!args.open);
        assert!(!args.listen);
        assert!(!args.exclude_ipv6);
    }

    #[test]
    fn test_flag_short_and_long_equivalence() {
        let short = Args::parse_from(&["test-bin", "-k", "-p", "80", "-o", "-l"]);
        let long = Args::parse_from(&[
            "test-bin",
            "--kill",
            "--port", "80",
            "--open",
            "--listen",
        ]);

        assert_eq!(short.kill, long.kill);
        assert_eq!(short.port, long.port);
        assert_eq!(short.open, long.open);
        assert_eq!(short.listen, long.listen);
        assert_eq!(short.exclude_ipv6, long.exclude_ipv6);
    }
}
