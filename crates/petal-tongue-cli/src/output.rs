// SPDX-License-Identifier: AGPL-3.0-only
//! Output formatting helpers (for testing; plain text without ANSI colors).

#[cfg(test)]
use std::fmt::Write;

/// Format show output as plain text (for testing; excludes ANSI colors).
#[cfg(test)]
#[must_use]
pub fn format_show_output(status: &petal_tongue_ipc::InstanceStatus) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "  ID:       {}", status.instance_id.as_str());
    let _ = writeln!(out, "  PID:      {}", status.pid);
    let _ = writeln!(out, "  Uptime:   {}s", status.uptime_seconds);
    let _ = writeln!(out, "  Nodes:    {}", status.node_count);
    let _ = writeln!(out, "  Edges:    {}", status.edge_count);
    let _ = writeln!(
        out,
        "  Window:   {}",
        if status.window_visible {
            "visible"
        } else {
            "hidden"
        }
    );
    if let Some(name) = &status.name {
        let _ = writeln!(out, "  Name:     {name}");
    }
    if let Some(wid) = status.window_id {
        let _ = writeln!(out, "  Window ID: 0x{wid:x}");
    }
    if !status.metadata.is_empty() {
        out.push_str("\n  Metadata:\n");
        for (key, value) in &status.metadata {
            let _ = writeln!(out, "    {key}: {value}");
        }
    }
    out
}

/// Format raise success output (for testing).
#[cfg(test)]
#[must_use]
pub fn format_raise_success(instance_id: &petal_tongue_core::InstanceId) -> String {
    format!("Instance {} raised", instance_id.as_str())
}

/// Format ping success output (for testing).
#[cfg(test)]
#[must_use]
pub fn format_ping_success(instance_id: &petal_tongue_core::InstanceId) -> String {
    format!("Instance {} is responsive", instance_id.as_str())
}

/// Format ping failure output (for testing).
#[cfg(test)]
#[must_use]
pub fn format_ping_failure(instance_id: &petal_tongue_core::InstanceId, error: &str) -> String {
    format!(
        "Instance {} is unresponsive\n   Error: {error}",
        instance_id.as_str()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::InstanceId;
    use petal_tongue_ipc::InstanceStatus;

    #[test]
    fn test_format_show_output() {
        let status = InstanceStatus {
            instance_id: InstanceId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            pid: 12345,
            window_id: Some(0x123),
            name: Some("Test Instance".to_string()),
            uptime_seconds: 3600,
            node_count: 5,
            edge_count: 10,
            window_visible: true,
            metadata: std::iter::once(("key".to_string(), "value".to_string())).collect(),
        };
        let out = format_show_output(&status);
        assert!(out.contains("550e8400-e29b-41d4-a716-446655440000"));
        assert!(out.contains("12345"));
        assert!(out.contains("3600"));
        assert!(out.contains('5'));
        assert!(out.contains("10"));
        assert!(out.contains("visible"));
        assert!(out.contains("Test Instance"));
        assert!(out.contains("key"));
        assert!(out.contains("value"));
    }

    #[test]
    fn test_format_show_output_hidden_window() {
        let status = InstanceStatus {
            instance_id: InstanceId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            pid: 1,
            window_id: None,
            name: None,
            uptime_seconds: 0,
            node_count: 0,
            edge_count: 0,
            window_visible: false,
            metadata: std::collections::HashMap::new(),
        };
        let out = format_show_output(&status);
        assert!(out.contains("hidden"));
        assert!(!out.contains("Metadata:"));
    }

    #[test]
    fn test_format_raise_success() {
        let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let out = format_raise_success(&id);
        assert!(out.contains("550e8400-e29b-41d4-a716-446655440000"));
        assert!(out.contains("raised"));
    }

    #[test]
    fn test_format_ping_success() {
        let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let out = format_ping_success(&id);
        assert!(out.contains("responsive"));
    }

    #[test]
    fn test_format_ping_failure() {
        let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let out = format_ping_failure(&id, "connection refused");
        assert!(out.contains("unresponsive"));
        assert!(out.contains("connection refused"));
    }

    #[test]
    fn test_format_show_output_minimal() {
        let status = InstanceStatus {
            instance_id: InstanceId::parse("550e8400-e29b-41d4-a716-446655440000").expect("id"),
            pid: 1,
            window_id: None,
            name: None,
            uptime_seconds: 0,
            node_count: 0,
            edge_count: 0,
            window_visible: false,
            metadata: std::collections::HashMap::new(),
        };
        let out = format_show_output(&status);
        assert!(out.contains("hidden"));
        assert!(out.contains("PID:"));
        assert!(out.contains('1'));
    }

    #[test]
    fn test_format_show_output_with_window_id() {
        let status = InstanceStatus {
            instance_id: InstanceId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            pid: 1,
            window_id: Some(0x1234),
            name: None,
            uptime_seconds: 0,
            node_count: 0,
            edge_count: 0,
            window_visible: false,
            metadata: std::collections::HashMap::new(),
        };
        let out = format_show_output(&status);
        assert!(out.contains("0x1234"));
    }

    #[test]
    fn test_format_show_output_with_metadata() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), "1.0".to_string());
        let status = InstanceStatus {
            instance_id: InstanceId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            pid: 1,
            window_id: None,
            name: None,
            uptime_seconds: 0,
            node_count: 0,
            edge_count: 0,
            window_visible: false,
            metadata,
        };
        let out = format_show_output(&status);
        assert!(out.contains("Metadata"));
        assert!(out.contains("version"));
        assert!(out.contains("1.0"));
    }
}
