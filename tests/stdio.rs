use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn startup_logs_never_enter_stdio_protocol() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_gnome-mcp-server"))
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            "unix:path=/nonexistent-mcp-fixture-bus",
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for (id, method) in [(1, "initialize"), (2, "tools/list"), (3, "resources/list")] {
        writeln!(
            input,
            "{}",
            serde_json::json!({"jsonrpc": "2.0", "id": id, "method": method, "params": {}})
        )
        .unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let responses: Vec<serde_json::Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).expect("non-protocol stdout"))
        .collect();
    assert_eq!(responses.len(), 3);
    for (index, response) in responses.iter().enumerate() {
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], index + 1);
    }
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("Starting GNOME MCP server (stdio)"));
    assert!(responses[1]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|tool| tool["name"] == "get_monitors"));
}
