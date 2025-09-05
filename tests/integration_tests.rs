use somo::schemas::Connection;
use std::net::IpAddr;
use std::process::Command;

// Should match the amount of netcat processes inside 'tests/setup/mock_processes.sh'.
static NUM_PROCESSES: usize = 8;

fn base_exec() -> Command {
    let exec_path = env!("CARGO_BIN_EXE_somo");
    let mut cmd = Command::new(exec_path);
    cmd.env("PROCFS_ROOT", "tests/mock/proc/");
    cmd
}

fn base_exec_json() -> Command {
    let mut cmd = base_exec();
    cmd.arg("--json");
    cmd
}

fn _base_exec_format(template: &str) -> Command {
    let mut cmd = base_exec();
    cmd.arg("--format").arg(template);
    cmd
}

fn exec_somo(mut cmd: Command) -> String {
    let output = cmd.output().expect("Failed to run somo.");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn test_basic_usage() {
    let cmd = base_exec_json();
    let stdout = exec_somo(cmd);

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
    let stdout = exec_somo(cmd);

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
    let stdout = exec_somo(cmd);

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
    let stdout = exec_somo(cmd);

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
    let stdout = exec_somo(cmd);

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
    let stdout = exec_somo(cmd);

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
    cmd.arg("--port").arg("5001"); // Port should exist in 'tests/setup/mock_processes.sh'
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    assert_eq!(connections.len(), 1, "Expected one connection.");
}

#[test]
fn test_nonexistent_port_filter() {
    let mut cmd = base_exec_json();
    cmd.arg("--port").arg("9999"); // Port shouldn't exist in 'tests/setup/mock_processes.sh'
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    assert!(connections.is_empty(), "Expected no connections.");
}

#[test]
fn test_remote_port_filter() {
    let mut cmd = base_exec_json();
    cmd.arg("--remote-port").arg("0"); // All remote ports should be 0 because we are serving the netcat processes ourselves
    let stdout = exec_somo(cmd);

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
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    assert!(
        !connections.is_empty(),
        "Expected for IPv4 connections to exist."
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
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    assert!(
        !connections.is_empty(),
        "Expected for IPv6 connections to exist."
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
    let stdout = exec_somo(cmd);

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
    let stdout = exec_somo(cmd);

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
    let stdout = exec_somo(cmd);

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
    let stdout = exec_somo(cmd);

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
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

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
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

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
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    // Expecting to get both IPv4 and IPv6, ie. all connections
    assert_eq!(
        connections.len(),
        NUM_PROCESSES,
        "Expected to receive all connections."
    );
}
