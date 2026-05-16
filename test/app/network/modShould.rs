#[cfg(test)]
mod network_tests {
    use crate::app::network::NetworkAnalyzer;
    use crate::app::network::NetworkConnection;

    #[test]
    fn test_new_analyzer_empty() {
        let analyzer = NetworkAnalyzer::new();
        assert!(analyzer.get_connections().is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn test_parse_netstat_line_windows_valid() {
        let analyzer = NetworkAnalyzer::new();
        let line = "TCP 0.0.0.0:135 0.0.0.0:0 LISTENING 1234";
        let result = analyzer.parse_netstat_line_windows(line);
        assert!(result.is_some());
        if let Some(conn) = result {
            assert_eq!(conn.protocol, "TCP");
            assert_eq!(conn.pid, 1234);
            assert_eq!(conn.state, "LISTENING");
        }
    }

    #[cfg(windows)]
    #[test]
    fn test_parse_netstat_line_windows_too_short() {
        let analyzer = NetworkAnalyzer::new();
        let line = "TCP 0.0.0.0";
        let result = analyzer.parse_netstat_line_windows(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_network_connection_serde() {
        let conn = NetworkConnection {
            protocol: "TCP".to_string(),
            local_address: "192.168.1.1".to_string(),
            local_port: 1234,
            foreign_address: "8.8.8.8".to_string(),
            foreign_port: 443,
            state: "ESTABLISHED".to_string(),
            pid: 5678,
            location: Some("US".to_string()),
            isp: Some("Google".to_string()),
        };
        let json = serde_json::to_string(&conn).unwrap();
        let deserialized: NetworkConnection = serde_json::from_str(&json).unwrap();
        assert_eq!(conn.protocol, deserialized.protocol);
        assert_eq!(conn.local_address, deserialized.local_address);
        assert_eq!(conn.local_port, deserialized.local_port);
        assert_eq!(conn.foreign_address, deserialized.foreign_address);
        assert_eq!(conn.foreign_port, deserialized.foreign_port);
        assert_eq!(conn.state, deserialized.state);
        assert_eq!(conn.pid, deserialized.pid);
        assert_eq!(conn.location, deserialized.location);
        assert_eq!(conn.isp, deserialized.isp);
    }

    #[test]
    fn test_connection_debug() {
        let conn = NetworkConnection {
            protocol: "TCP".to_string(),
            local_address: "0.0.0.0".to_string(),
            local_port: 0,
            foreign_address: "0.0.0.0".to_string(),
            foreign_port: 0,
            state: "NONE".to_string(),
            pid: 0,
            location: None,
            isp: None,
        };
        let debug = format!("{:?}", conn);
        assert!(debug.contains("TCP"));
        assert!(debug.contains("NONE"));
    }
}
