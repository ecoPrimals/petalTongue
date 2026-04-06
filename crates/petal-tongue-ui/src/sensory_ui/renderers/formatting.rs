// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure formatting functions for sensory UI (testable, no egui).

/// Format topology summary as single line
#[must_use]
pub fn format_topology_summary(node_count: usize, edge_count: usize) -> String {
    format!("Topology: {node_count} nodes, {edge_count} edges")
}

/// Format CPU metric
#[must_use]
pub fn format_cpu_metrics(cpu_percent: f64) -> String {
    format!("CPU: {cpu_percent:.1}%")
}

/// Format memory metric
#[must_use]
pub fn format_memory_metrics(memory_percent: f64) -> String {
    format!("Memory: {memory_percent:.1}%")
}

/// Format proprioception as label-value pairs for display
#[must_use]
pub fn format_proprioception_summary(
    health_percentage: f32,
    status: &str,
    confidence: f32,
) -> Vec<(String, String)> {
    vec![
        ("Health".to_string(), format!("{health_percentage:.0}%")),
        ("Status".to_string(), status.to_string()),
        ("Confidence".to_string(), format!("{confidence:.1}")),
    ]
}

/// Format nodes count
#[must_use]
pub fn format_topology_nodes(node_count: usize) -> String {
    format!("Nodes: {node_count}")
}

/// Format edges count
#[must_use]
pub fn format_topology_edges(edge_count: usize) -> String {
    format!("Edges: {edge_count}")
}

/// Format average degree
#[must_use]
pub fn format_avg_degree(avg_degree: f32) -> String {
    format!("Avg Degree: {avg_degree:.1}")
}

/// Format capabilities count
#[must_use]
pub fn format_capabilities_count(count: usize) -> String {
    format!("{count} caps")
}

/// Format combined CPU and memory for compact display
#[must_use]
pub fn format_cpu_memory_combined(cpu_percent: f64, memory_percent: f64) -> String {
    format!("CPU: {cpu_percent:.1}% | Memory: {memory_percent:.1}%")
}

/// Format health and confidence for compact display
#[must_use]
pub fn format_health_confidence(health_percentage: f32, confidence: f32) -> String {
    format!("Health: {health_percentage:.0}% | Confidence: {confidence:.0}%")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_topology_summary() {
        assert_eq!(
            format_topology_summary(5, 12),
            "Topology: 5 nodes, 12 edges"
        );
        assert_eq!(format_topology_summary(0, 0), "Topology: 0 nodes, 0 edges");
    }

    #[test]
    fn test_format_cpu_metrics() {
        assert_eq!(format_cpu_metrics(45.2), "CPU: 45.2%");
        assert_eq!(format_cpu_metrics(0.0), "CPU: 0.0%");
        assert_eq!(format_cpu_metrics(100.0), "CPU: 100.0%");
    }

    #[test]
    fn test_format_memory_metrics() {
        assert_eq!(format_memory_metrics(62.5), "Memory: 62.5%");
        assert_eq!(format_memory_metrics(0.0), "Memory: 0.0%");
    }

    #[test]
    fn test_format_proprioception_summary() {
        let pairs = format_proprioception_summary(85.0, "Healthy", 92.5);
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0], ("Health".to_string(), "85%".to_string()));
        assert_eq!(pairs[1], ("Status".to_string(), "Healthy".to_string()));
        assert_eq!(pairs[2], ("Confidence".to_string(), "92.5".to_string()));
    }

    #[test]
    fn test_format_topology_nodes_edges() {
        assert_eq!(format_topology_nodes(10), "Nodes: 10");
        assert_eq!(format_topology_edges(25), "Edges: 25");
    }

    #[test]
    fn test_format_avg_degree() {
        assert_eq!(format_avg_degree(2.5), "Avg Degree: 2.5");
        assert_eq!(format_avg_degree(0.0), "Avg Degree: 0.0");
    }

    #[test]
    fn test_format_capabilities_count() {
        assert_eq!(format_capabilities_count(3), "3 caps");
        assert_eq!(format_capabilities_count(0), "0 caps");
    }

    #[test]
    fn test_format_cpu_memory_combined() {
        assert_eq!(
            format_cpu_memory_combined(45.0, 62.5),
            "CPU: 45.0% | Memory: 62.5%"
        );
    }

    #[test]
    fn test_format_health_confidence() {
        assert_eq!(
            format_health_confidence(85.0, 92.0),
            "Health: 85% | Confidence: 92%"
        );
    }

    #[test]
    fn test_format_proprioception_summary_edge_cases() {
        let pairs = format_proprioception_summary(0.0, "Offline", 0.0);
        assert_eq!(pairs[0].1, "0%");
        assert_eq!(pairs[2].1, "0.0");
        let pairs = format_proprioception_summary(100.0, "Healthy", 100.0);
        assert_eq!(pairs[0].1, "100%");
        assert_eq!(pairs[2].1, "100.0");
    }

    #[test]
    fn test_format_cpu_metrics_edge_cases() {
        assert_eq!(format_cpu_metrics(99.9), "CPU: 99.9%");
        assert_eq!(format_cpu_metrics(0.1), "CPU: 0.1%");
    }

    #[test]
    fn test_format_memory_metrics_edge_cases() {
        assert_eq!(format_memory_metrics(100.0), "Memory: 100.0%");
    }

    #[test]
    fn test_format_capabilities_count_single() {
        assert_eq!(format_capabilities_count(1), "1 caps");
    }

    #[test]
    fn test_format_cpu_memory_combined_edge_cases() {
        assert_eq!(
            format_cpu_memory_combined(0.0, 0.0),
            "CPU: 0.0% | Memory: 0.0%"
        );
        assert_eq!(
            format_cpu_memory_combined(100.0, 100.0),
            "CPU: 100.0% | Memory: 100.0%"
        );
    }
}
