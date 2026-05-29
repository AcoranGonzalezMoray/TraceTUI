#[cfg(test)]
mod libraries_tests {
    use crate::app::states::LibraryState;
    use crate::app::App;

    #[test]
    fn test_library_state_new() {
        let state = LibraryState::new();
        assert!(state.libraries.is_empty());
        assert!(!state.libraries_loading);
        assert_eq!(state.selected_library_index, 0);
        assert_eq!(state.selected_library_process_index, 0);
        assert_eq!(state.library_search_query, "");
        assert!(!state.show_hash_info_modal);
        assert!(!state.show_library_binary_viewer);
        assert_eq!(state.library_binary_scroll, 0);
        assert_eq!(state.library_binary_tab, 0);
        assert!(state.libraries_rx.is_none());
    }

    #[test]
    fn test_library_state_loading() {
        let mut state = LibraryState::new();
        state.libraries_loading = true;
        assert!(state.libraries_loading);
    }

    #[test]
    fn test_library_state_search() {
        let mut state = LibraryState::new();
        state.library_search_query = "kernel32".to_string();
        assert_eq!(state.library_search_query, "kernel32");
    }

    #[test]
    fn test_library_state_binary_viewer() {
        let mut state = LibraryState::new();
        state.show_library_binary_viewer = true;
        state.library_binary_path = "C:\\Windows\\System32\\kernel32.dll".to_string();
        state.library_binary_hex_lines = vec!["00000000  4D 5A 90 00".to_string()];
        state.library_binary_disasm_lines = vec!["push rbp".to_string()];
        assert!(state.show_library_binary_viewer);
        assert_eq!(
            state.library_binary_path,
            "C:\\Windows\\System32\\kernel32.dll"
        );
        assert_eq!(state.library_binary_hex_lines.len(), 1);
        assert_eq!(state.library_binary_disasm_lines.len(), 1);
    }

    #[test]
    fn test_app_refresh_libraries_skips_when_loading() {
        let mut app = App::new();
        app.libraries.libraries_loading = true;
        app.refresh_libraries();
        assert!(app.libraries.libraries_rx.is_none());
    }

    #[test]
    fn test_app_refresh_libraries_skips_when_no_data() {
        let mut app = App::new();
        app.refresh_libraries();
        assert!(app.libraries.libraries_loading);
        assert!(!app.ui.status_message.is_empty());
        assert!(app.libraries.libraries_rx.is_none());
    }

    fn sample_lib(
        pid: u32,
        name: &str,
        path: &str,
        risk: &str,
    ) -> crate::app::libraries::LibraryInfo {
        use crate::app::libraries::{LibraryOrigin, SignatureStatus};
        crate::app::libraries::LibraryInfo {
            pid,
            process_name: "test.exe".to_string(),
            name: name.to_string(),
            path: path.to_string(),
            size: 1024,
            signature: SignatureStatus::Unknown,
            origin: LibraryOrigin::System,
            is_signed: None,
            risk: risk.to_string(),
            sha256: String::new(),
        }
    }

    #[test]
    fn test_export_libraries_json_empty() {
        let result = crate::app::libraries::export_libraries_json(&[]);
        assert!(result.is_ok());
        let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_export_libraries_json_with_data() {
        let libs = vec![
            sample_lib(
                1,
                "kernel32.dll",
                "C:\\Windows\\System32\\kernel32.dll",
                "Safe",
            ),
            sample_lib(
                2,
                "suspicious.dll",
                "C:\\Users\\test\\suspicious.dll",
                "Suspicious",
            ),
        ];
        let result = crate::app::libraries::export_libraries_json(&libs);
        assert!(result.is_ok());
        let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "kernel32.dll");
        assert_eq!(arr[0]["pid"], 1);
        assert_eq!(arr[0]["risk"], "Safe");
        assert_eq!(arr[1]["name"], "suspicious.dll");
        assert_eq!(arr[1]["risk"], "Suspicious");
    }

    #[test]
    fn test_export_libraries_json_preserves_all_fields() {
        let libs = vec![sample_lib(
            42,
            "ntdll.dll",
            "C:\\Windows\\System32\\ntdll.dll",
            "Safe",
        )];
        let result = crate::app::libraries::export_libraries_json(&libs).unwrap();
        let json: serde_json::Value = serde_json::from_str(&result).unwrap();
        let obj = &json[0];
        assert!(obj.get("pid").is_some());
        assert!(obj.get("process_name").is_some());
        assert!(obj.get("name").is_some());
        assert!(obj.get("path").is_some());
        assert!(obj.get("size").is_some());
        assert!(obj.get("risk").is_some());
        assert!(obj.get("sha256").is_some());
    }

    #[test]
    fn test_export_libraries_csv_empty() {
        let csv = crate::app::libraries::export_libraries_csv(&[]);
        assert!(csv.starts_with("pid,process_name,name,path,size,risk,origin,signature,sha256\n"));
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 1, "Empty export should only have header");
    }

    #[test]
    fn test_export_libraries_csv_with_data() {
        let libs = vec![sample_lib(
            1,
            "kernel32.dll",
            "C:\\Windows\\System32\\kernel32.dll",
            "Safe",
        )];
        let csv = crate::app::libraries::export_libraries_csv(&libs);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 2, "Should have header + 1 data row");
        assert!(lines[1].contains("kernel32.dll"));
        assert!(lines[1].starts_with("1,"));
    }

    #[test]
    fn test_export_libraries_csv_escapes_commas_in_path() {
        let libs = vec![sample_lib(
            1,
            "lib.dll",
            "C:\\Program Files (x86),special\\lib.dll",
            "Safe",
        )];
        let csv = crate::app::libraries::export_libraries_csv(&libs);
        assert!(csv.contains("\"C:\\Program Files (x86),special\\lib.dll\""));
    }

    #[test]
    fn test_export_libraries_csv_header_columns() {
        let csv = crate::app::libraries::export_libraries_csv(&[]);
        let header = csv.lines().next().unwrap();
        let cols: Vec<&str> = header.split(',').collect();
        assert_eq!(cols.len(), 9);
        assert_eq!(cols[0], "pid");
        assert_eq!(cols[1], "process_name");
        assert_eq!(cols[2], "name");
        assert_eq!(cols[3], "path");
        assert_eq!(cols[4], "size");
        assert_eq!(cols[5], "risk");
        assert_eq!(cols[6], "origin");
        assert_eq!(cols[7], "signature");
        assert_eq!(cols[8], "sha256");
    }

    #[test]
    fn test_classify_origin_system_windows() {
        use crate::app::libraries::{classify_origin, LibraryOrigin};
        let origin = classify_origin("C:\\Windows\\System32\\ntdll.dll");
        assert!(matches!(origin, LibraryOrigin::System));
    }

    #[test]
    fn test_classify_origin_system_linux() {
        use crate::app::libraries::{classify_origin, LibraryOrigin};
        let origin = classify_origin("/usr/lib/x86_64-linux-gnu/libc.so.6");
        assert!(matches!(origin, LibraryOrigin::System));
    }

    #[test]
    fn test_classify_origin_program_files() {
        use crate::app::libraries::{classify_origin, LibraryOrigin};
        let origin = classify_origin("C:\\Program Files\\App\\lib.dll");
        assert!(matches!(origin, LibraryOrigin::ProgramFiles));
    }

    #[test]
    fn test_classify_origin_program_files_linux() {
        use crate::app::libraries::{classify_origin, LibraryOrigin};
        let origin = classify_origin("/opt/myapp/lib.so");
        assert!(matches!(origin, LibraryOrigin::ProgramFiles));
    }

    #[test]
    fn test_classify_origin_temp() {
        use crate::app::libraries::{classify_origin, LibraryOrigin};
        let origin = classify_origin("C:\\Users\\test\\AppData\\Local\\Temp\\payload.dll");
        assert!(matches!(origin, LibraryOrigin::Temp));
    }

    #[test]
    fn test_classify_origin_temp_linux() {
        use crate::app::libraries::{classify_origin, LibraryOrigin};
        let origin = classify_origin("/tmp/suspicious.so");
        assert!(matches!(origin, LibraryOrigin::Temp));
    }

    #[test]
    fn test_classify_origin_userspace() {
        use crate::app::libraries::{classify_origin, LibraryOrigin};
        let origin = classify_origin("C:\\Users\\admin\\Downloads\\lib.dll");
        assert!(matches!(origin, LibraryOrigin::UserSpace));
    }

    #[test]
    fn test_classify_origin_userspace_linux() {
        use crate::app::libraries::{classify_origin, LibraryOrigin};
        let origin = classify_origin("/home/user/projects/lib.so");
        assert!(matches!(origin, LibraryOrigin::UserSpace));
    }

    #[test]
    fn test_classify_origin_unknown() {
        use crate::app::libraries::{classify_origin, LibraryOrigin};
        let origin = classify_origin("/some/random/path/lib.so");
        assert!(matches!(origin, LibraryOrigin::Unknown));
    }

    #[test]
    fn test_classify_risk_safe_system() {
        use crate::app::libraries::{classify_risk, LibraryOrigin};
        let risk = classify_risk("C:\\Windows\\System32\\ntdll.dll", &LibraryOrigin::System);
        assert_eq!(risk, "Safe");
    }

    #[test]
    fn test_classify_risk_critical_temp() {
        use crate::app::libraries::{classify_risk, LibraryOrigin};
        let risk = classify_risk("/tmp/normal.so", &LibraryOrigin::Temp);
        assert_eq!(risk, "Critical");
    }

    #[test]
    fn test_classify_risk_critical_keyword_inject() {
        use crate::app::libraries::{classify_risk, LibraryOrigin};
        let risk = classify_risk(
            "C:\\Windows\\System32\\inject_hook.dll",
            &LibraryOrigin::System,
        );
        assert_eq!(risk, "Critical");
    }

    #[test]
    fn test_classify_risk_critical_keyword_payload() {
        use crate::app::libraries::{classify_risk, LibraryOrigin};
        let risk = classify_risk("/usr/lib/payload.so", &LibraryOrigin::System);
        assert_eq!(risk, "Critical");
    }

    #[test]
    fn test_classify_risk_suspicious_downloads() {
        use crate::app::libraries::{classify_risk, LibraryOrigin};
        let risk = classify_risk(
            "C:\\Users\\admin\\Downloads\\unknown.dll",
            &LibraryOrigin::UserSpace,
        );
        assert_eq!(risk, "Suspicious");
    }

    #[test]
    fn test_classify_risk_suspicious_home() {
        use crate::app::libraries::{classify_risk, LibraryOrigin};
        let risk = classify_risk("/home/user/lib.so", &LibraryOrigin::UserSpace);
        assert_eq!(risk, "Suspicious");
    }

    #[test]
    fn test_risk_sort_key_ordering() {
        use crate::app::libraries::risk_sort_key;
        assert!(risk_sort_key("Critical") > risk_sort_key("Suspicious"));
        assert!(risk_sort_key("Suspicious") > risk_sort_key("Unknown"));
        assert!(risk_sort_key("Unknown") > risk_sort_key("Safe"));
        assert_eq!(risk_sort_key("Safe"), 0);
        assert_eq!(risk_sort_key("anything_else"), 0);
    }

    #[test]
    fn test_pick_save_path_returns_default_in_test_mode() {
        let app = App::new();
        let result =
            crate::app::services::input_service::pick_save_path_for_test(&app, "test_file.json");
        assert!(result.is_some());
        assert_eq!(result.unwrap().to_str().unwrap(), "test_file.json");
    }
}
