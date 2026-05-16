#[cfg(test)]
mod process_tests {
    use crate::app::process::ProcessInfo;
    use crate::app::process::ProcessManager;

    #[test]
    fn test_new_process_manager() {
        let manager = ProcessManager::new();
        assert!(manager.get_all_processes().is_empty());
    }

    #[test]
    fn test_process_info_structure() {
        let info = ProcessInfo {
            pid: 1234,
            name: "test.exe".to_string(),
            path: Some("C:\\test.exe".to_string()),
            command_line: Some("test.exe --flag".to_string()),
            cpu_usage: 12.5,
            memory_usage: 4096,
            start_time: None,
            status: "Running".to_string(),
        };
        assert_eq!(info.pid, 1234);
        assert_eq!(info.name, "test.exe");
        assert_eq!(info.cpu_usage, 12.5);
        assert_eq!(info.memory_usage, 4096);
    }

    #[test]
    fn test_process_info_serde() {
        let info = ProcessInfo {
            pid: 999,
            name: "proc".to_string(),
            path: Some("/usr/bin/proc".to_string()),
            command_line: None,
            cpu_usage: 0.0,
            memory_usage: 0,
            start_time: None,
            status: "Idle".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: ProcessInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.pid, deserialized.pid);
        assert_eq!(info.name, deserialized.name);
    }
}
