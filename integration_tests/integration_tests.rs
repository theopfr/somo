use std::process::Command;
use somo::schemas::Connection;

/// Helper to create a base Command that runs `somo --json`
fn base_somo_command() -> Command {
    let exec_path = env!("CARGO_BIN_EXE_somo");
    let mut cmd = Command::new(exec_path);
    cmd.arg("--json");
    cmd
}

/// Test that the binary runs and returns a non-empty list of connections
#[test]
#[ignore] // ignored by default; run with -- --ignored
fn test_basic_usage() {
    let mut cmd = base_somo_command();
    let output = cmd.output().expect("Failed to run somo");
    let stdout = String::from_utf8_lossy(&output.stdout);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON");
    assert!(!connections.is_empty(), "Expected at least one connection");
}

/// Test that all returned connections have a valid protocol
#[test]
#[ignore]
fn test_proto_field() {
    let mut cmd = base_somo_command();
    let output = cmd.output().expect("Failed to run somo");
    let stdout = String::from_utf8_lossy(&output.stdout);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON");

    for conn in connections {
        assert!(
            conn.proto == "tcp" || conn.proto == "udp",
            "Unexpected protocol: {}",
            conn.proto
        );
    }
}

/// Test filtering by TCP connections
#[test]
#[ignore]
fn test_tcp_filter() {
    let exec_path = env!("CARGO_BIN_EXE_somo");
    let mut cmd = Command::new(exec_path);
    cmd.arg("--json").arg("--tcp");
    let output = cmd.output().expect("Failed to run somo");
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{:?}", stdout);
    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON");

    for conn in &connections {
        assert_eq!(conn.proto, "tcp", "Expected only TCP connections");
    }
}

/// Test filtering by UDP connections
#[test]
#[ignore]
fn test_udp_filter() {
    let exec_path = env!("CARGO_BIN_EXE_somo");
    let mut cmd = Command::new(exec_path);
    cmd.arg("--json").arg("--udp");
    let output = cmd.output().expect("Failed to run somo");
    let stdout = String::from_utf8_lossy(&output.stdout);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON");

    for conn in &connections {
        assert_eq!(conn.proto, "udp", "Expected only UDP connections");
    }
}

/// Test filtering by a non-existent remote IP (should return zero connections)
#[test]
#[ignore]
fn test_filter_by_nonexistent_ip() {
    let exec_path = env!("CARGO_BIN_EXE_somo");
    let mut cmd = Command::new(exec_path);
    cmd.arg("--json").arg("--ip").arg("10.255.255.1"); // unlikely to exist
    let output = cmd.output().expect("Failed to run somo");
    let stdout = String::from_utf8_lossy(&output.stdout);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON");
    assert!(connections.is_empty(), "Expected no connections for non-existent IP");
}

/// Test that local ports are valid numeric strings
#[test]
#[ignore]
fn test_local_port_format() {
    let mut cmd = base_somo_command();
    let output = cmd.output().expect("Failed to run somo");
    let stdout = String::from_utf8_lossy(&output.stdout);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON");

    for conn in &connections {
        let port: Result<u16, _> = conn.local_port.parse();
        assert!(
            port.is_ok(),
            "Local port is not a valid number: {}",
            conn.local_port
        );
    }
}
