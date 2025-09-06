#![cfg(target_os = "linux")]

/// This file contains integration tests which run the somo binary and test the resulting stdout.
/// Instead of using the host systems procfs pseudo filesystem (because it's not deterministic),
/// somo reads from a mocked procfs directory, at 'tests/mock/proc' (by setting the PROCFS_ROOT env. variable).
/// It contains some dummy netcat processes on which we can run our tests.
///
/// Currently, the mocked procfs would result in the following somo output:
///
/// > PROCFS_SOMO=tests/mock/proc cargo run -- -c
/// ┌───┬───────┬────────────┬────────────────┬─────────────┬─────────────┬────────┐
/// │ # │ proto │ local port │ remote address │ remote port │ pid program │ state  │
/// ├───┼───────┼────────────┼────────────────┼─────────────┼─────────────┼────────┤
/// │1  │ tcp   │ 5001       │ 0.0.0.0        │ 0           │ 9 nc        │ listen │
/// │2  │ tcp   │ 5002       │ 0.0.0.0        │ 0           │ 11 nc       │ listen │
/// │3  │ tcp   │ 5003       │ [::]           │ 0           │ 13 nc       │ listen │
/// │4  │ tcp   │ 5004       │ [::]           │ 0           │ 15 nc       │ listen │
/// │5  │ udp   │ 5005       │ 0.0.0.0        │ 0           │ 17 nc       │ close  │
/// │6  │ udp   │ 5006       │ 0.0.0.0        │ 0           │ 19 nc       │ close  │
/// │7  │ udp   │ 5007       │ [::]           │ 0           │ 21 nc       │ close  │
/// │8  │ udp   │ 5008       │ [::]           │ 0           │ 22 nc       │ close  │
/// └───┴───────┴────────────┴────────────────┴─────────────┴─────────────┴────────┘
///
/// The tests are split into two groups:
/// * 'test_connection_data': Tests if somo outputs the correct data (eg. when filtering or sorting) by using the `--json` flag
///                           to serialize the connections into Rust structs and checking the fields.
/// * 'test_stdout_format':   Tests if the human-readable stdout (eg. the table or custom format strings) is as expected.
use assert_cmd::Command;
use somo::schemas::Connection;
use std::net::IpAddr;

// Should match the amount of netcat processes inside 'tests/setup/init_processes.sh'.
static NUM_PROCESSES: usize = 8;

fn base_exec() -> Command {
    let exec_path = env!("CARGO_BIN_EXE_somo");
    let mut cmd = Command::cargo_bin(exec_path).unwrap();
    cmd.env("PROCFS_ROOT", "tests/mock/proc/");
    cmd
}

fn base_exec_json() -> Command {
    let mut cmd = base_exec();
    cmd.arg("--json");
    cmd
}

fn base_exec_format(template: &str) -> Command {
    let mut cmd = base_exec();
    cmd.arg("--format").arg(template);
    cmd
}

fn get_stdout(mut cmd: Command) -> String {
    let output = cmd.output().expect("Failed to run somo.");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[cfg(test)]
mod test_connection_data {
    use super::*;

    #[test]
    fn test_basic_usage() {
        let cmd = base_exec_json();
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        assert_eq!(
            connections.len(),
            NUM_PROCESSES,
            "Expected to receive all connections."
        );
    }

    #[test]
    fn test_tcp_filter() {
        let mut cmd = base_exec_json();
        cmd.arg("--tcp");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        assert!(
            !connections.is_empty(),
            "Expected for TCP connections to exist."
        );
        for conn in &connections {
            assert_eq!(conn.proto, "tcp", "Expected only TCP connections.");
        }
    }

    #[test]
    fn test_udp_filter() {
        let mut cmd = base_exec_json();
        cmd.arg("--udp");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        assert!(
            !connections.is_empty(),
            "Expected for UDP connections to exist."
        );
        for conn in &connections {
            assert_eq!(conn.proto, "udp", "Expected only UDP connections.");
        }
    }

    #[test]
    fn test_tcp_and_udp_filter() {
        let mut cmd = base_exec_json();
        cmd.arg("--tcp").arg("--udp");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        // Expecting to get both TCP and UDP, ie. all connections
        assert_eq!(
            connections.len(),
            NUM_PROCESSES,
            "Expected to receive all connections."
        );
    }

    #[test]
    fn test_proto_filter() {
        // Test deprecated `--proto tcp` filter
        let mut cmd = base_exec_json();
        cmd.arg("--proto").arg("tcp");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        assert!(
            !connections.is_empty(),
            "Expected for TCP connections to exist."
        );
        for conn in &connections {
            assert_eq!(conn.proto, "tcp", "Expected only TCP connections.");
        }

        // Test deprecated `--proto udp` filter
        let mut cmd = base_exec_json();
        cmd.arg("--proto").arg("udp");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        assert!(
            !connections.is_empty(),
            "Expected for UDP connections to exist."
        );
        for conn in &connections {
            assert_eq!(conn.proto, "udp", "Expected only UDP connections.");
        }
    }

    #[test]
    fn test_port_filter() {
        let mut cmd = base_exec_json();
        cmd.arg("--port").arg("5001"); // Port should exist in 'tests/setup/init_processes.sh'
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        assert_eq!(connections.len(), 1, "Expected one connection.");
    }

    #[test]
    fn test_nonexistent_port_filter() {
        let mut cmd = base_exec_json();
        cmd.arg("--port").arg("9999"); // Port shouldn't exist in 'tests/setup/init_processes.sh'
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        assert!(connections.is_empty(), "Expected no connections.");
    }

    #[test]
    fn test_remote_port_filter() {
        let mut cmd = base_exec_json();
        cmd.arg("--remote-port").arg("0"); // All remote ports should be 0 because we are serving the netcat processes ourselves
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        assert!(
            !connections.is_empty(),
            "Expected for connections to exist."
        );
        for conn in &connections {
            assert_eq!(conn.remote_port, "0", "Expected only '0' remote ports.");
        }
    }

    #[test]
    fn test_ip_filter_with_ipv4() {
        let mut cmd = base_exec_json();
        cmd.arg("--ip").arg("0.0.0.0");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        assert!(
            !connections.is_empty(),
            "Expected for connections to exist."
        );
        for conn in &connections {
            assert_eq!(
                conn.remote_address, "0.0.0.0",
                "Expected only '0.0.0.0' connections."
            );
        }
    }

    #[test]
    fn test_ip_filter_with_ipv6() {
        let mut cmd = base_exec_json();
        cmd.arg("--ip").arg("[::]");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        assert!(
            !connections.is_empty(),
            "Expected for connections to exist."
        );
        for conn in &connections {
            assert_eq!(
                conn.remote_address, "[::]",
                "Expected only '[::]' connections."
            );
        }
    }

    #[test]
    fn test_program_filter() {
        let mut cmd = base_exec_json();
        cmd.arg("--program").arg("nc"); // All processes are netcat
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        assert_eq!(
            connections.len(),
            NUM_PROCESSES,
            "Expected to receive all connections."
        );
    }

    #[test]
    fn test_open_state_filter() {
        let mut cmd = base_exec_json();
        cmd.arg("--open");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        for conn in &connections {
            assert!(conn.state != "closed", "Expected only open connections.");
        }
    }

    #[test]
    fn test_listen_state_filter() {
        // Test `--listen` filter ('listen' state may not appear in mocked processes)
        let mut cmd = base_exec_json();
        cmd.arg("--listen");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        // 'listen' state may not appear in mocked processes, then the loop will be skipped
        for conn in &connections {
            assert_eq!(conn.state, "listen", "Expected only listening connections.");
        }
    }

    #[test]
    fn test_established_state_filter() {
        let mut cmd = base_exec_json();
        cmd.arg("--established");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        // 'established' state may not appear in mocked processes, then the loop will be skipped
        for conn in &connections {
            assert_eq!(
                conn.state, "established",
                "Expected only established connections."
            );
        }
    }

    #[test]
    fn test_ipv4_filter() {
        let mut cmd: Command = base_exec_json();
        cmd.arg("--ipv4");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        assert!(
            !connections.is_empty(),
            "Expected for IPv4 connections to exist."
        );
        for conn in &connections {
            let ip: IpAddr = conn.remote_address.parse().expect("Invalid IP address.");
            assert!(
                matches!(ip, IpAddr::V4(_)),
                "Expected only IPv4 connections, got {}.",
                conn.remote_address
            );
        }
    }

    #[test]
    fn test_ipv6_filter() {
        let mut cmd = base_exec_json();
        cmd.arg("--ipv6");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        assert!(
            !connections.is_empty(),
            "Expected for IPv6 connections to exist."
        );
        for conn in &connections {
            let ip: IpAddr = conn
                .remote_address
                .trim_matches(|c| c == '[' || c == ']')
                .parse()
                .expect("Invalid IP address.");
            assert!(
                matches!(ip, IpAddr::V6(_)),
                "Expected only IPv6 connections, got {}.",
                conn.remote_address
            );
        }
    }

    #[test]
    fn test_ipv4_and_ipv6_filters() {
        let mut cmd = base_exec_json();
        cmd.arg("--ipv4").arg("--ipv6");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        // Expecting to get both IPv4 and IPv6, ie. all connections
        assert_eq!(
            connections.len(),
            NUM_PROCESSES,
            "Expected to receive all connections."
        );
    }

    #[test]
    fn test_exclude_ipv6_filter() {
        let mut cmd = base_exec_json();
        cmd.arg("--exclude-ipv6");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        assert!(
            !connections.is_empty(),
            "Expected for non IPv6 connections to exist."
        );
        for conn in &connections {
            let ip: IpAddr = conn.remote_address.parse().expect("Invalid IP address.");
            assert!(
                matches!(ip, IpAddr::V4(_)),
                "Expected only IPv4 connections, got {}.",
                conn.remote_address
            );
        }
    }

    #[test]
    fn test_sort_by_pid() {
        let mut cmd = base_exec_json();
        cmd.arg("--sort=pid");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        let pids: Vec<i32> = connections
            .iter()
            .map(|conn| conn.pid.parse::<i32>().unwrap())
            .collect();

        let mut sorted_pids = pids.clone();
        sorted_pids.sort();

        assert_eq!(
            pids, sorted_pids,
            "Expected connections to be numerically sorted by PID ascending."
        );
    }

    #[test]
    fn test_sort_by_pid_reverse() {
        let mut cmd = base_exec_json();
        cmd.arg("--sort=pid").arg("--reverse");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        let pids: Vec<i32> = connections
            .iter()
            .map(|conn| conn.pid.parse::<i32>().unwrap())
            .collect();

        let mut sorted_pids = pids.clone();
        sorted_pids.sort();
        sorted_pids.reverse();

        assert_eq!(
            pids, sorted_pids,
            "Expected connections to be numerically sorted by PID descending."
        );
    }

    #[test]
    fn test_sort_by_protocol() {
        let mut cmd = base_exec_json();
        cmd.arg("--sort=proto");
        let stdout = get_stdout(cmd);

        let connections: Vec<Connection> =
            serde_json::from_str(&stdout).expect("Failed to parse JSON.");

        let protos: Vec<String> = connections.iter().map(|conn| conn.proto.clone()).collect();

        let mut sorted_protos = protos.clone();
        sorted_protos.sort();

        assert_eq!(
            protos, sorted_protos,
            "Expected connections to be lexicographically sorted by protocol (TCP/UDP) ascending."
        );
    }
}

#[cfg(test)]
mod test_stdout_format {
    use super::*;

    #[test]
    fn test_custom_format() {
        let cmd = base_exec_format(
            "PID: {{pid}}, Protocol: {{proto}}, Remote Address: {{remote_address}}",
        );
        let stdout = get_stdout(cmd);

        let line_re = regex::Regex::new(
            r"^PID: \d+, Protocol: (tcp|udp), Remote Address: (0\.0\.0\.0|\[::\])$",
        )
        .unwrap();

        for line in stdout.lines() {
            assert!(
                line_re.is_match(line),
                "Line did not match expected format: {}",
                line
            );
        }
    }

    #[test]
    fn test_custom_format_syntax_error() {
        let mut cmd = base_exec_format("PID: {{pid}");

        cmd.assert()
            .failure()
            .code(2) // Should be exit code 2
            .stdout(predicates::str::contains("Invalid template syntax.")) // Expect error message
            .stdout(predicates::str::contains("└─>")) // Expect syntax line pointer
            .stdout(predicates::str::contains("^")); // Expect syntax char pointer
    }

    #[test]
    fn test_get_config_file_path() {
        let mut cmd = base_exec();
        cmd.arg("--config-file")
            .assert()
            .success()
            .stdout(predicates::str::ends_with("/somo/config\n"));
    }

    #[test]
    fn test_table_layout() {
        let cmd = base_exec();
        let stdout = get_stdout(cmd);

        // Check table corners
        let corners = ['┌', '┐', '└', '┘'];
        for &c in &corners {
            assert!(
                stdout.contains(c),
                "Expected table corner '{}' to be present.",
                c
            );
        }

        // Check table headers (seperated by words because of possible line breaks during testing)
        let headers = [
            "proto", "local", "port", "remote", "address", "remote", "port", "pid", "state",
        ];
        for &header in &headers {
            assert!(
                stdout.contains(header),
                "Expected table header '{}' to be present.",
                header
            );
        }

        // Check row separators count matches number of processes
        let row_separator_count = stdout.matches('├').count();
        assert_eq!(
            row_separator_count, NUM_PROCESSES,
            "Expected {} row separators (one per process), found {}.",
            NUM_PROCESSES, row_separator_count
        );
    }

    #[test]
    fn test_compact_table_layout() {
        let mut cmd = base_exec();
        cmd.arg("--compact");

        let stdout = get_stdout(cmd);

        // Check table corners
        let corners = ['┌', '┐', '└', '┘'];
        for &c in &corners {
            assert!(
                stdout.contains(c),
                "Expected table corner '{}' to be present.",
                c
            );
        }

        // Check table headers (seperated by words because of possible line breaks during testing)
        let headers = [
            "proto", "local", "port", "remote", "address", "remote", "port", "pid", "state",
        ];
        for &header in &headers {
            assert!(
                stdout.contains(header),
                "Expected table header '{}' to be present.",
                header
            );
        }

        // Check that there is only one row seperator (between headers and table body)
        let row_separator_count = stdout.matches('├').count();
        assert_eq!(
            row_separator_count, 1,
            "Expected 1 row separators (one per process), found {}.",
            row_separator_count
        );
    }
}
