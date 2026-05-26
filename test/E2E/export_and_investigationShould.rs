#[cfg(test)]
mod e2e_export_and_investigation {
    use crate::app::network::NetworkConnection;
    use crate::app::process::ProcessInfo;
    use crate::app::risk::RiskAnalyzer;
    use crate::app::types::AppConnection;
    use crate::app::App;
    use crate::utils::signatures::SignatureStatus;

    fn sample_conn(pid: u32, foreign: &str) -> NetworkConnection {
        NetworkConnection {
            protocol: "TCP".to_string(),
            local_address: "0.0.0.0".to_string(),
            local_port: pid as u16 + 1000,
            foreign_address: foreign.to_string(),
            foreign_port: 443,
            state: "ESTABLISHED".to_string(),
            pid,
            location: None,
            isp: None,
        }
    }

    fn build_app(
        pid: u32,
        name: &str,
        cpu: f32,
        mem: u64,
        conns: Vec<NetworkConnection>,
    ) -> AppConnection {
        let proc = ProcessInfo {
            pid,
            name: name.to_string(),
            path: Some(format!("C:\\{}.exe", name)),
            command_line: None,
            cpu_usage: cpu,
            memory_usage: mem,
            start_time: None,
            status: "Running".to_string(),
        };
        let risk = RiskAnalyzer::calculate(&proc, &conns);
        AppConnection {
            process_name: name.to_string(),
            process_path: format!("C:\\{}.exe", name),
            icon: String::new(),
            pid,
            connections: conns,
            cpu_usage: cpu,
            memory_usage: mem,
            risk_level: risk,
            signature_status: SignatureStatus::Unknown,
        }
    }

    #[test]
    fn e2e_export_multi_app_with_cleanup() {
        let mut app = App::new();
        app.network.app_connections = vec![
            build_app(
                1,
                "chrome.exe",
                25.0,
                300_000_000,
                vec![
                    NetworkConnection {
                        protocol: "TCP".to_string(),
                        local_address: "192.168.1.100".to_string(),
                        local_port: 54321,
                        foreign_address: "8.8.8.8".to_string(),
                        foreign_port: 443,
                        state: "ESTABLISHED".to_string(),
                        pid: 1,
                        location: Some("🌎 United States".to_string()),
                        isp: Some("Google LLC".to_string()),
                    },
                    NetworkConnection {
                        protocol: "TCP".to_string(),
                        local_address: "192.168.1.100".to_string(),
                        local_port: 54322,
                        foreign_address: "1.1.1.1".to_string(),
                        foreign_port: 853,
                        state: "ESTABLISHED".to_string(),
                        pid: 1,
                        location: None,
                        isp: None,
                    },
                ],
            ),
            build_app(
                2,
                "firefox.exe",
                10.0,
                150_000_000,
                vec![NetworkConnection {
                    protocol: "UDP".to_string(),
                    local_address: "192.168.1.100".to_string(),
                    local_port: 5353,
                    foreign_address: "224.0.0.251".to_string(),
                    foreign_port: 5353,
                    state: "NONE".to_string(),
                    pid: 2,
                    location: None,
                    isp: None,
                }],
            ),
        ];
        app.ui.auto_analysis_complete = true;
        app.ui.is_initial_loading = false;

        crate::app::services::input_service::export_to_json(&mut app);

        let json_files: Vec<_> = std::fs::read_dir(".")
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let fname = e.file_name();
                let name = fname.to_string_lossy();
                name.starts_with("network_analysis_") && name.ends_with(".json")
            })
            .collect();
        assert!(!json_files.is_empty(), "Expected at least one export file");

        let path = json_files[0].path();
        let content = std::fs::read_to_string(&path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(json.is_array());
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        assert_eq!(arr[0]["process_name"], "chrome.exe");
        assert_eq!(arr[0]["pid"], 1);
        assert_eq!(arr[0]["connections"].as_array().unwrap().len(), 2);
        assert_eq!(arr[0]["connections"][0]["foreign_address"], "8.8.8.8");
        assert_eq!(arr[0]["connections"][0]["location"], "🌎 United States");
        assert_eq!(arr[0]["connections"][0]["isp"], "Google LLC");
        assert_eq!(
            arr[0]["connections"][1]["location"],
            serde_json::Value::Null
        );

        assert_eq!(arr[1]["process_name"], "firefox.exe");
        assert_eq!(arr[1]["pid"], 2);
        assert_eq!(arr[1]["connections"][0]["protocol"], "UDP");
        assert_eq!(arr[1]["connections"][0]["foreign_address"], "224.0.0.251");

        let _ = std::fs::remove_file(&path);
        assert!(
            !path.exists(),
            "Export file should be cleaned up after test"
        );
    }

    #[test]
    fn e2e_investigation_report_structure() {
        use crate::app::investigation_service::InvestigationReport;

        let report = InvestigationReport::new("8.8.8.8".to_string(), 443);
        let json = serde_json::to_string_pretty(&report).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["ip"], "8.8.8.8");
        assert_eq!(parsed["port"], 443);
        assert_eq!(parsed["risk_score"], 0);
        assert!(parsed["risk_factors"].as_array().unwrap().is_empty());
        assert!(parsed["hops"].as_array().unwrap().is_empty());
        assert!(parsed["domain"].is_null());
        assert!(parsed["whois_data"].is_null());
        assert_eq!(parsed["lat"], 0.0);
        assert_eq!(parsed["lon"], 0.0);
    }

    #[test]
    fn e2e_risk_scoring_integration() {
        use crate::app::process::ProcessInfo;
        use crate::app::risk::RiskAnalyzer;

        let proc = ProcessInfo {
            pid: 1,
            name: "powershell.exe".to_string(),
            path: Some("C:\\powershell.exe".to_string()),
            command_line: None,
            cpu_usage: 60.0,
            memory_usage: 2_000_000_000,
            start_time: None,
            status: "Running".to_string(),
        };

        let conns: Vec<NetworkConnection> = (0..15)
            .map(|i| NetworkConnection {
                protocol: "TCP".to_string(),
                local_address: "0.0.0.0".to_string(),
                local_port: 1000 + i as u16,
                foreign_address: format!("10.0.0.{}", i + 1),
                foreign_port: 443,
                state: "ESTABLISHED".to_string(),
                pid: 1,
                location: None,
                isp: None,
            })
            .collect();

        let risk = RiskAnalyzer::calculate(&proc, &conns);
        assert_eq!(risk, "CRITICAL");
    }

    #[test]
    fn e2e_grouping_integration() {
        use crate::app::grouping::ConnectionGrouper;
        use crate::app::process::ProcessInfo;

        let procs = vec![
            ProcessInfo {
                pid: 1,
                name: "app1.exe".to_string(),
                path: Some("C:\\app1.exe".to_string()),
                command_line: None,
                cpu_usage: 5.0,
                memory_usage: 100_000,
                start_time: None,
                status: "Running".to_string(),
            },
            ProcessInfo {
                pid: 2,
                name: "app2.exe".to_string(),
                path: Some("C:\\app2.exe".to_string()),
                command_line: None,
                cpu_usage: 10.0,
                memory_usage: 200_000,
                start_time: None,
                status: "Running".to_string(),
            },
        ];

        let conns = vec![
            sample_conn(1, "8.8.8.8"),
            sample_conn(1, "1.1.1.1"),
            sample_conn(1, "4.4.4.4"),
            sample_conn(2, "8.8.8.8"),
        ];

        let result = ConnectionGrouper::group(&procs, &conns, false, |_, _| String::new());
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].pid, 1);
        assert_eq!(result[0].connections.len(), 3);
        assert_eq!(result[1].pid, 2);
        assert_eq!(result[1].connections.len(), 1);

        assert!(!result[0].risk_level.is_empty());
        assert!(!result[1].risk_level.is_empty());

        assert!(
            result[0].signature_status == crate::utils::signatures::SignatureStatus::Unsigned
                || result[0].signature_status == crate::utils::signatures::SignatureStatus::Unknown,
            "Expected Unsigned or Unknown, got {:?}",
            result[0].signature_status
        );
    }
}
