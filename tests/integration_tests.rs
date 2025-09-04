use somo::schemas::Connection;
use std::process::Command;
use std::net::IpAddr;


// Should match the amount of netcat processes inside 'integration_tests/setup/mock_processes.sh'.
static NUM_PROCESSES: usize = 6;


fn base_exec() -> Command {
    let exec_path = env!("CARGO_BIN_EXE_somo");
    let cmd = Command::new(exec_path);
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
#[ignore]
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
#[ignore]
fn test_tcp_udp_filters() {
    // Test `--tcp` filter
    let mut cmd = base_exec_json();
    cmd.arg("--tcp");
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    for conn in &connections {
        assert_eq!(conn.proto, "tcp", "Expected only TCP connections.");
    }

    // Test `--udp` filter
    let mut cmd = base_exec_json();
    cmd.arg("--udp");
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    for conn in &connections {
        assert_eq!(conn.proto, "udp", "Expected only UDP connections.");
    }

    // Test setting both `--udp` and `--tcp` filters (expecting to receive results for both)
    let mut cmd = base_exec_json();
    cmd.arg("--tcp").arg("--udp");
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
#[ignore]
fn test_proto_filter() {
    // Test deprecated `--proto tcp` filter
    let mut cmd = base_exec_json();
    cmd.arg("--proto").arg("tcp");
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    for conn in &connections {
        assert_eq!(conn.proto, "tcp", "Expected only TCP connections.");
    }

    // Test deprecated `--proto udp` filter
    let mut cmd = base_exec_json();
    cmd.arg("--proto").arg("udp");
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    for conn in &connections {
        assert_eq!(conn.proto, "udp", "Expected only UDP connections.");
    }
}

#[test]
#[ignore]
fn test_port_filter() {
    let mut cmd = base_exec_json();
    cmd.arg("--port").arg("5001"); // Port should exist in 'integration_tests/setup/mock_processes.sh'
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    assert_eq!(connections.len(), 1, "Expected one connection.");
}

#[test]
#[ignore]
fn test_nonexistent_port_filter() {
    let mut cmd = base_exec_json();
    cmd.arg("--port").arg("9999"); // Port shouldn't exist in 'integration_tests/setup/mock_processes.sh'
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    assert!(connections.is_empty(), "Expected no connections.");
}

#[test]
#[ignore]
fn test_remote_port_filter() {
    let mut cmd = base_exec_json();
    cmd.arg("--remote-port").arg("0");
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    for conn in &connections {
        assert_eq!(conn.remote_port, "0", "Expected only '0' remote ports.");
    }
}

#[test]
#[ignore]
fn test_ip_filter() {
    let mut cmd = base_exec_json();
    cmd.arg("--ip").arg("0.0.0.0");
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> = serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    for conn in &connections {
        assert_eq!(conn.remote_address, "0.0.0.0", "Expected only '0.0.0.0' connections.");
    }

    let mut cmd = base_exec_json();
    cmd.arg("--ip").arg("[::]");
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> = serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    for conn in &connections {
        assert_eq!(conn.remote_address, "[::]", "Expected only '[::]' connections.");
    }
}


#[test]
#[ignore]
fn test_program_filter() {
    let mut cmd = base_exec_json();
    cmd.arg("--program").arg("nc"); // All processes are netcat
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> = serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    assert_eq!(
        connections.len(),
        NUM_PROCESSES,
        "Expected to receive all connections."
    );
}


#[test]
#[ignore]
fn test_state_filters() {
    // Test `--open` filter
    let mut cmd = base_exec_json();
    cmd.arg("--open");
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> = serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    for conn in &connections {
        assert!(conn.state != "closed", "Expected only open connections.");
    }

    // Test `--listen` filter ('listen' state not guaranteed happen in mocked processes)
    let mut cmd = base_exec_json();
    cmd.arg("--listen");
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> = serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    for conn in &connections {
        assert_eq!(conn.state, "listen", "Expected only listening connections.");
    }

    // Test `--established` filter ('established' state not guaranteed to happen in mocked processes)
    let mut cmd = base_exec_json();
    cmd.arg("--established");
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> = serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    for conn in &connections {
        assert_eq!(conn.state, "established", "Expected only established connections.");
    }
}

#[test]
#[ignore]
fn test_ipv4_ipv6_filters() {
    // Test `--ipv4` filter
    let mut cmd = base_exec_json();
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

    // Test `--ipv6` filter
    let mut cmd = base_exec_json();
    cmd.arg("--ipv6");
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    for conn in &connections {
        let ip: IpAddr = conn.remote_address.parse().expect("Invalid IP address.");
        assert!(
            matches!(ip, IpAddr::V6(_)),
            "Expected only IPv6 connections, got {}.",
            conn.remote_address
        );
    }

    // Test setting both `--ipv4` and `--ivp6` filters (expecting to receive results for both)
    let mut cmd = base_exec_json();
    cmd.arg("--ipv4").arg("--ipv6");
    let stdout = exec_somo(cmd);

    let connections: Vec<Connection> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON.");

    assert_eq!(
        connections.len(),
        NUM_PROCESSES,
        "Expected to receive all connections."
    );
}