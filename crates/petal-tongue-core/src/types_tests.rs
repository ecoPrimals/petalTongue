//! Tests for types module

#[cfg(test)]
mod tests {
    use super::super::types::*;

    #[test]
    fn test_primal_health_status_variants() {
        let healthy = PrimalHealthStatus::Healthy;
        let warning = PrimalHealthStatus::Warning;
        let critical = PrimalHealthStatus::Critical;
        let unknown = PrimalHealthStatus::Unknown;

        assert!(matches!(healthy, PrimalHealthStatus::Healthy));
        assert!(matches!(warning, PrimalHealthStatus::Warning));
        assert!(matches!(critical, PrimalHealthStatus::Critical));
        assert!(matches!(unknown, PrimalHealthStatus::Unknown));
    }

    #[test]
    fn test_primal_health_status_as_str() {
        assert_eq!(PrimalHealthStatus::Healthy.as_str(), "Healthy");
        assert_eq!(PrimalHealthStatus::Warning.as_str(), "Warning");
        assert_eq!(PrimalHealthStatus::Critical.as_str(), "Critical");
        assert_eq!(PrimalHealthStatus::Unknown.as_str(), "Unknown");
    }

    #[test]
    fn test_primal_health_status_parse() {
        assert_eq!(
            PrimalHealthStatus::parse_health_status("Healthy"),
            PrimalHealthStatus::Healthy
        );
        assert_eq!(
            PrimalHealthStatus::parse_health_status("Warning"),
            PrimalHealthStatus::Warning
        );
        assert_eq!(
            PrimalHealthStatus::parse_health_status("Critical"),
            PrimalHealthStatus::Critical
        );
        assert_eq!(
            PrimalHealthStatus::parse_health_status("invalid"),
            PrimalHealthStatus::Unknown
        );
    }

    #[test]
    fn test_primal_info_creation() {
        let info = PrimalInfo {
            id: "test-1".to_string(),
            name: "Test Primal".to_string(),
            primal_type: "compute".to_string(),
            endpoint: "http://test:8080".to_string(),
            health: PrimalHealthStatus::Healthy,
            trust_level: None,
            family_id: None,
            capabilities: vec!["cap1".to_string(), "cap2".to_string()],
            last_seen: 1_234_567_890,
            properties: Default::default(),
        };

        assert_eq!(info.id, "test-1");
        assert_eq!(info.name, "Test Primal");
        assert_eq!(info.primal_type, "compute");
        assert_eq!(info.endpoint, "http://test:8080");
        assert_eq!(info.capabilities.len(), 2);
        assert!(matches!(info.health, PrimalHealthStatus::Healthy));
        assert_eq!(info.last_seen, 1_234_567_890);
    }

    #[test]
    fn test_primal_info_clone() {
        let info = PrimalInfo {
            id: "test-1".to_string(),
            name: "Test Primal".to_string(),
            primal_type: "compute".to_string(),
            endpoint: "http://test:8080".to_string(),
            health: PrimalHealthStatus::Healthy,
            trust_level: None,
            family_id: None,
            capabilities: vec!["cap1".to_string()],
            last_seen: 1_234_567_890,
            properties: Default::default(),
        };

        let cloned = info.clone();
        assert_eq!(info.id, cloned.id);
        assert_eq!(info.name, cloned.name);
        assert_eq!(info.capabilities, cloned.capabilities);
        assert_eq!(info.last_seen, cloned.last_seen);
    }

    #[test]
    fn test_topology_edge_creation() {
        let edge = TopologyEdge {
            from: "primal-a".to_string(),
            to: "primal-b".to_string(),
            edge_type: "capability".to_string(),
            label: Some("test-label".to_string()),
        };

        assert_eq!(edge.from, "primal-a");
        assert_eq!(edge.to, "primal-b");
        assert_eq!(edge.edge_type, "capability");
        assert_eq!(edge.label, Some("test-label".to_string()));
    }

    #[test]
    fn test_topology_edge_without_label() {
        let edge = TopologyEdge {
            from: "primal-a".to_string(),
            to: "primal-b".to_string(),
            edge_type: "capability".to_string(),
            label: None,
        };

        assert_eq!(edge.from, "primal-a");
        assert_eq!(edge.to, "primal-b");
        assert!(edge.label.is_none());
    }

    #[test]
    fn test_topology_graph() {
        let primal1 = PrimalInfo {
            id: "primal-1".to_string(),
            name: "Primal 1".to_string(),
            primal_type: "compute".to_string(),
            endpoint: "http://p1:8080".to_string(),
            health: PrimalHealthStatus::Healthy,
            trust_level: None,
            family_id: None,
            capabilities: vec![],
            last_seen: 1_234_567_890,
            properties: Default::default(),
        };

        let primal2 = PrimalInfo {
            id: "primal-2".to_string(),
            name: "Primal 2".to_string(),
            primal_type: "storage".to_string(),
            endpoint: "http://p2:8080".to_string(),
            health: PrimalHealthStatus::Healthy,
            trust_level: None,
            family_id: None,
            capabilities: vec![],
            last_seen: 1_234_567_890,
            properties: Default::default(),
        };

        let edge = TopologyEdge {
            from: "primal-1".to_string(),
            to: "primal-2".to_string(),
            edge_type: "api_call".to_string(),
            label: None,
        };

        let graph = TopologyGraph {
            nodes: vec![primal1, primal2],
            edges: vec![edge],
            timestamp: 1_234_567_890,
        };

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.timestamp, 1_234_567_890);
    }

    #[test]
    fn test_connection_status_variants() {
        let connected = ConnectionStatus::Connected;
        let connecting = ConnectionStatus::Connecting;
        let disconnected = ConnectionStatus::Disconnected;
        let error = ConnectionStatus::Error("test error".to_string());

        assert!(matches!(connected, ConnectionStatus::Connected));
        assert!(matches!(connecting, ConnectionStatus::Connecting));
        assert!(matches!(disconnected, ConnectionStatus::Disconnected));
        assert!(matches!(error, ConnectionStatus::Error(_)));
    }

    #[test]
    fn test_primal_connection() {
        let connection = PrimalConnection {
            name: "Test Primal".to_string(),
            primal_type: "compute".to_string(),
            status: ConnectionStatus::Connected,
            endpoint: "http://test:8080".to_string(),
            last_heartbeat: Some(1_234_567_890),
        };

        assert_eq!(connection.name, "Test Primal");
        assert_eq!(connection.primal_type, "compute");
        assert!(matches!(connection.status, ConnectionStatus::Connected));
        assert_eq!(connection.last_heartbeat, Some(1_234_567_890));
    }

    #[test]
    fn test_flow_event() {
        let event = FlowEvent {
            id: "event-1".to_string(),
            from: "primal-a".to_string(),
            to: "primal-b".to_string(),
            message_type: "api_call".to_string(),
            timestamp: 1_234_567_890,
            metadata: Some(serde_json::json!({"key": "value"})),
        };

        assert_eq!(event.id, "event-1");
        assert_eq!(event.from, "primal-a");
        assert_eq!(event.to, "primal-b");
        assert!(event.metadata.is_some());
    }

    #[test]
    fn test_traffic_stats() {
        let stats = TrafficStats {
            from: "primal-a".to_string(),
            to: "primal-b".to_string(),
            message_count: 100,
            bytes_transferred: 10240,
            avg_latency_ms: 12.5,
            period_start: 1_234_567_890,
            period_end: 1_234_567_900,
        };

        assert_eq!(stats.message_count, 100);
        assert_eq!(stats.bytes_transferred, 10240);
        assert!((stats.avg_latency_ms - 12.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_primal_info_serialization() {
        let info = PrimalInfo {
            id: "test-1".to_string(),
            name: "Test Primal".to_string(),
            primal_type: "compute".to_string(),
            endpoint: "http://test:8080".to_string(),
            health: PrimalHealthStatus::Healthy,
            trust_level: None,
            family_id: None,
            capabilities: vec!["cap1".to_string()],
            last_seen: 1_234_567_890,
            properties: Default::default(),
        };

        let json = serde_json::to_string(&info).expect("Failed to serialize");
        assert!(json.contains("test-1"));
        assert!(json.contains("Test Primal"));

        let deserialized: PrimalInfo = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.id, info.id);
        assert_eq!(deserialized.name, info.name);
    }

    #[test]
    fn test_topology_edge_serialization() {
        let edge = TopologyEdge {
            from: "primal-a".to_string(),
            to: "primal-b".to_string(),
            edge_type: "capability".to_string(),
            label: Some("test".to_string()),
        };

        let json = serde_json::to_string(&edge).expect("Failed to serialize");
        assert!(json.contains("primal-a"));
        assert!(json.contains("primal-b"));
        assert!(json.contains("capability"));

        let deserialized: TopologyEdge =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.from, edge.from);
        assert_eq!(deserialized.to, edge.to);
    }
}
