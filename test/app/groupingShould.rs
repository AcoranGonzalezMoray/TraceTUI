#[cfg(test)]
mod grouping_tests {
    use crate::app::grouping::ConnectionGrouper;
    use crate::app::network::NetworkConnection;
    use crate::app::process::ProcessInfo;

    fn make_proc(pid: u32, name: &str) -> ProcessInfo {
        ProcessInfo {
            pid,
            name: name.to_string(),
            path: Some(format!("C:\\{}.exe", name)),
            command_line: None,
            cpu_usage: 5.0,
            memory_usage: 1_000_000,
            start_time: None,
            status: "Running".to_string(),
        }
    }

    fn make_conn(pid: u32, foreign: &str) -> NetworkConnection {
        NetworkConnection {
            protocol: "TCP".to_string(),
            local_address: "0.0.0.0".to_string(),
            local_port: 12345,
            foreign_address: foreign.to_string(),
            foreign_port: 443,
            state: "ESTABLISHED".to_string(),
            pid,
            location: None,
            isp: None,
        }
    }

    #[test]
    fn test_group_empty_inputs() {
        let result = ConnectionGrouper::group(&[], &[], false, |_, _| String::new());
        assert!(result.is_empty());
    }

    #[test]
    fn test_group_single_process_one_connection() {
        let procs = vec![make_proc(1, "test")];
        let conns = vec![make_conn(1, "8.8.8.8")];
        let result = ConnectionGrouper::group(&procs, &conns, false, |_, _| String::new());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pid, 1);
        assert_eq!(result[0].process_name, "test");
        assert_eq!(result[0].connections.len(), 1);
    }

    #[test]
    fn test_group_process_without_connections_excluded() {
        let procs = vec![make_proc(1, "with_conns"), make_proc(2, "no_conns")];
        let conns = vec![make_conn(1, "8.8.8.8")];
        let result = ConnectionGrouper::group(&procs, &conns, false, |_, _| String::new());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pid, 1);
    }

    #[test]
    fn test_group_multiple_connections_same_process() {
        let procs = vec![make_proc(1, "test")];
        let conns = vec![make_conn(1, "8.8.8.8"), make_conn(1, "1.1.1.1")];
        let result = ConnectionGrouper::group(&procs, &conns, false, |_, _| String::new());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].connections.len(), 2);
    }

    #[test]
    fn test_group_sorted_by_connection_count() {
        let procs = vec![make_proc(1, "few"), make_proc(2, "many")];
        let conns = vec![
            make_conn(1, "8.8.8.8"),
            make_conn(2, "1.1.1.1"),
            make_conn(2, "2.2.2.2"),
            make_conn(2, "3.3.3.3"),
        ];
        let result = ConnectionGrouper::group(&procs, &conns, false, |_, _| String::new());
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].pid, 2);
        assert_eq!(result[1].pid, 1);
    }

    #[test]
    fn test_group_icon_callback_called() {
        use std::cell::RefCell;
        let called = RefCell::new(false);
        let procs = vec![make_proc(1, "test")];
        let conns = vec![make_conn(1, "8.8.8.8")];
        let _result = ConnectionGrouper::group(&procs, &conns, false, |path, name| {
            *called.borrow_mut() = true;
            assert!(!path.is_empty());
            assert_eq!(name, "test");
            String::new()
        });
        assert!(*called.borrow());
    }

    #[test]
    fn test_group_hunter_mode_keeps_unsigned_known_safe() {
        let procs = vec![make_proc(1, "svchost")];
        let conns = vec![make_conn(1, "8.8.8.8")];
        let result = ConnectionGrouper::group(&procs, &conns, true, |_, _| String::new());
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_group_risk_level_calculated() {
        let procs = vec![make_proc(1, "test")];
        let conns = vec![make_conn(1, "8.8.8.8")];
        let result = ConnectionGrouper::group(&procs, &conns, false, |_, _| String::new());
        assert_eq!(result.len(), 1);
        assert!(!result[0].risk_level.is_empty());
    }

    #[test]
    fn test_group_includes_all_fields() {
        let procs = vec![make_proc(42, "chrome")];
        let conns = vec![make_conn(42, "8.8.8.8")];
        let result = ConnectionGrouper::group(&procs, &conns, false, |_, _| "icon".to_string());
        assert_eq!(result[0].pid, 42);
        assert_eq!(result[0].process_name, "chrome");
        assert_eq!(result[0].process_path, "C:\\chrome.exe");
        assert_eq!(result[0].icon, "icon");
        assert_eq!(result[0].cpu_usage, 5.0);
        assert_eq!(result[0].memory_usage, 1_000_000);
    }
}
