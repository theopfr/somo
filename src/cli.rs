use clap::Parser;
use inquire::InquireError;
use inquire::Select;
use nix::sys::signal;
use nix::unistd::Pid;
use std::string::String;

use crate::schemas::Connection;
use crate::utils;

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
        exclude_ipv6: args.exclude_ipv6,
    };
}

/// Kills a process by its PID.
///
/// # Argument
/// * `pid`: The PID value as a string.
///
/// # Returns
/// None
pub fn kill_process(pid_num: i32) {
    let pid = Pid::from_raw(pid_num);

    match signal::kill(pid, signal::Signal::SIGTERM) {
        Ok(_) => utils::pretty_print_info(&format!("Killed process with PID {}.", pid)),
        Err(_) => utils::pretty_print_error(&format!("Failed to kill process with PID {}", pid)),
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
            let pid_str = &connections[choice as usize - 1].pid;
            let pid_num = match pid_str.parse::<i32>() {
                Ok(pid) => pid,
                Err(_) => {
                    utils::pretty_print_error("Couldn't find PID.");
                    return;
                }
            };
            kill_process(pid_num)
        }
        Err(_) => {
            utils::pretty_print_error("Process selection cancelled.");
            return;
        }
    };
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
            "--proto",
            "udp",
            "--ip",
            "192.168.0.1",
            "--remote-port",
            "53",
            "-p",
            "8080",
            "--program",
            "nginx",
            "--pid",
            "1234",
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
        let long = Args::parse_from(&["test-bin", "--kill", "--port", "80", "--open", "--listen"]);

        assert_eq!(short.kill, long.kill);
        assert_eq!(short.port, long.port);
        assert_eq!(short.open, long.open);
        assert_eq!(short.listen, long.listen);
        assert_eq!(short.exclude_ipv6, long.exclude_ipv6);
    }
}
