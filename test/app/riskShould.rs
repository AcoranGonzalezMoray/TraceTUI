#[cfg(test)]
mod risk_tests {
    use crate::app::network::NetworkConnection;
    use crate::app::process::ProcessInfo;
    use crate::app::risk::RiskAnalyzer;

    fn make_proc(name: &str, cpu: f32, mem: u64, path: Option<&str>) -> ProcessInfo {
        ProcessInfo {
            pid: 1,
            name: name.to_string(),
            path: path.map(|s| s.to_string()),
            command_line: None,
            cpu_usage: cpu,
            memory_usage: mem,
            start_time: None,
            status: "Running".to_string(),
        }
    }

    fn make_conn(pid: u32) -> NetworkConnection {
        NetworkConnection {
            protocol: "TCP".to_string(),
            local_address: "0.0.0.0".to_string(),
            local_port: 12345,
            foreign_address: "8.8.8.8".to_string(),
            foreign_port: 443,
            state: "ESTABLISHED".to_string(),
            pid,
            location: None,
            isp: None,
        }
    }

    #[test]
    fn test_low_risk_few_connections() {
        let proc = make_proc("notepad.exe", 5.0, 1_000_000, Some("C:\\notepad.exe"));
        let conns = vec![make_conn(1)];
        assert_eq!(RiskAnalyzer::calculate(&proc, &conns), "LOW");
    }

    #[test]
    fn test_medium_risk_moderate_connections() {
        let proc = make_proc("notepad.exe", 5.0, 1_000_000, Some("C:\\notepad.exe"));
        let conns = vec![make_conn(1); 6];
        assert_eq!(RiskAnalyzer::calculate(&proc, &conns), "MEDIUM");
    }

    #[test]
    fn test_high_risk_many_connections_with_cpu() {
        let proc = make_proc("notepad.exe", 60.0, 1_000_000, Some("C:\\notepad.exe"));
        let conns = vec![make_conn(1); 11];
        assert_eq!(RiskAnalyzer::calculate(&proc, &conns), "HIGH");
    }

    #[test]
    fn test_high_cpu_increases_risk() {
        let proc = make_proc("notepad.exe", 60.0, 1_000_000, Some("C:\\notepad.exe"));
        let conns = vec![make_conn(1); 6];
        let risk = RiskAnalyzer::calculate(&proc, &conns);
        assert_eq!(risk, "MEDIUM");
    }

    #[test]
    fn test_high_memory_increases_risk() {
        let high_mem: u64 = 600 * 1024 * 1024;
        let proc = make_proc("notepad.exe", 5.0, high_mem, Some("C:\\notepad.exe"));
        let conns = vec![make_conn(1); 1];
        let risk = RiskAnalyzer::calculate(&proc, &conns);
        assert_eq!(risk, "LOW");
    }

    #[test]
    fn test_suspicious_process_name_increases_risk() {
        let proc = make_proc("powershell.exe", 5.0, 1_000_000, Some("C:\\powershell.exe"));
        let conns = vec![make_conn(1); 1];
        assert_eq!(RiskAnalyzer::calculate(&proc, &conns), "MEDIUM");
    }

    #[test]
    fn test_no_path_increases_risk() {
        let proc = make_proc("unknown.exe", 5.0, 1_000_000, None);
        let conns = vec![make_conn(1)];
        assert_eq!(RiskAnalyzer::calculate(&proc, &conns), "LOW");
    }

    #[test]
    fn test_critical_risk_all_factors() {
        let proc = make_proc(
            "powershell.exe",
            60.0,
            2_000_000_000,
            Some("C:\\powershell.exe"),
        );
        let conns = vec![make_conn(1); 15];
        assert_eq!(RiskAnalyzer::calculate(&proc, &conns), "CRITICAL");
    }

    #[test]
    fn test_is_high_or_critical() {
        assert!(RiskAnalyzer::is_high_or_critical("HIGH"));
        assert!(RiskAnalyzer::is_high_or_critical("CRITICAL"));
        assert!(RiskAnalyzer::is_high_or_critical("HIGH RISK"));
        assert!(!RiskAnalyzer::is_high_or_critical("LOW"));
        assert!(!RiskAnalyzer::is_high_or_critical("MEDIUM"));
    }
}
