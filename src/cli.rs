use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use inquire::InquireError;
use inquire::Select;
use std::io;
use std::process;
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
    #[command(subcommand)]
    command: Option<Commands>,

    /// Display an interactive selection option after inspecting connections
    #[arg(short = 'k', long, default_value = None)]
    kill: bool,

    /// Filter connections by protocol, e.g., "tcp", "udp"
    #[arg(long, default_value = None)]
    proto: Option<String>,

    /// Filter connections by remote IP address
    #[arg(long, default_value = None)]
    ip: Option<String>,

    /// Filter connections by remote port
    #[arg(long, default_value = None)]
    remote_port: Option<String>,

    /// Filter connections by local port
    #[arg(short = 'p', long, default_value = None)]
    port: Option<String>,

    /// Filter connections by program name
    #[arg(long, default_value = None)]
    program: Option<String>,

    /// Filter connections by PID
    #[arg(long, default_value = None)]
    pid: Option<String>,

    /// Filter by open connections
    #[arg(short = 'o', long, default_value_t = false)]
    open: bool,

    /// Filter by listening connections
    #[arg(short = 'l', long, default_value_t = false)]
    listen: bool,

    /// Exclude IPv6 connections
    #[arg(long, default_value_t = false)]
    exclude_ipv6: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate shell completions
    GenerateCompletions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Gets all flag values provided by the user in the CLI using the "clap" crate.
///
/// # Arguments
/// None
///
/// # Returns
/// A struct containing all the flag values, or None if a subcommand was executed.
pub fn cli() -> Option<Flags> {
    let args = Args::parse();

    // Handle subcommands
    if let Some(command) = args.command {
        match command {
            Commands::GenerateCompletions { shell } => {
                let mut cmd = Args::command();
                print_completions(shell, &mut cmd);
                return None;
            }
        }
    }

    return Some(Flags {
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
    });
}

/// Generates and prints shell completions to stdout.
///
/// # Arguments
/// * `gen` - The shell to generate completions for
/// * `cmd` - The clap command to generate completions for
///
/// # Returns
/// None
fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
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
        utils::pretty_print_error("Couldn't kill process! Try again using sudo.");
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
    use super::{Args, Commands};
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

    #[test]
    fn test_generate_completions_subcommand() {
        let args = Args::parse_from(&["test-bin", "generate-completions", "bash"]);

        match args.command {
            Some(Commands::GenerateCompletions { shell }) => {
                assert_eq!(shell.to_string(), "bash");
            }
            _ => panic!("Expected GenerateCompletions command"),
        }
    }

    #[test]
    fn test_generate_completions_all_shells() {
        let shells = ["bash", "zsh", "fish", "elvish"];

        for shell in &shells {
            let args = Args::parse_from(&["test-bin", "generate-completions", shell]);

            match args.command {
                Some(Commands::GenerateCompletions {
                    shell: parsed_shell,
                }) => {
                    assert_eq!(parsed_shell.to_string(), *shell);
                }
                _ => panic!("Expected GenerateCompletions command for {}", shell),
            }
        }
    }

    #[test]
    fn test_cli_returns_none_for_subcommands() {
        // Mock the Args parsing by directly testing the logic
        // This test ensures that when a subcommand is present, cli() returns None

        // We can't easily test the full cli() function without actually running the completion
        // generation, so we test the Args parsing logic instead
        let args = Args::parse_from(&["test-bin", "generate-completions", "bash"]);

        // Verify that a subcommand is present
        assert!(args.command.is_some());

        // Verify that flags are still parsed correctly even with subcommands
        assert!(!args.kill);
        assert!(args.proto.is_none());
    }
}
