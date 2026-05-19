#[cfg(test)]
mod e2e_analysis_lifecycle {
    use crate::app::network::NetworkConnection;
    use crate::app::process::ProcessInfo;
    use crate::app::risk::RiskAnalyzer;
    use crate::app::types::AppConnection;
    use crate::app::App;

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

    fn sample_proc(pid: u32, name: &str, cpu: f32, mem: u64) -> ProcessInfo {
        ProcessInfo {
            pid,
            name: name.to_string(),
            path: Some(format!("C:\\{}.exe", name)),
            command_line: None,
            cpu_usage: cpu,
            memory_usage: mem,
            start_time: None,
            status: "Running".to_string(),
        }
    }

    fn build_app_connection(
        pid: u32,
        name: &str,
        cpu: f32,
        mem: u64,
        conns: Vec<NetworkConnection>,
    ) -> AppConnection {
        let proc = sample_proc(pid, name, cpu, mem);
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
            signature_status: crate::utils::signatures::SignatureStatus::Unknown,
        }
    }

    #[test]
    fn e2e_analysis_pause_resume_cycle() {
        let mut app = App::new();

        app.auto_analysis_complete = true;
        app.is_initial_loading = false;
        app.show_welcome_dialog = false;
        app.show_update_dialog = false;
        assert!(!app.analysis_paused);
        assert_eq!(app.continuous_refresh_counter, 0);

        app.on_tick();
        assert_eq!(app.continuous_refresh_counter, 1);

        app.analysis_paused = true;
        let before = app.continuous_refresh_counter;
        app.on_tick();
        assert_eq!(app.continuous_refresh_counter, before);

        app.analysis_paused = false;
        app.on_tick();
        assert_eq!(app.continuous_refresh_counter, before + 1);

        app.handle_key_event(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('r'),
            crossterm::event::KeyModifiers::empty(),
        ));
        assert!(app.analysis_paused);
        assert_eq!(app.continuous_refresh_counter, 0);
    }

    #[test]
    fn e2e_frame_count_and_history() {
        let mut app = App::new();
        let conns = vec![sample_conn(1, "8.8.8.8")];
        app.app_connections = vec![build_app_connection(1, "chrome", 15.0, 200_000_000, conns)];
        app.selected_app_index = 0;
        app.frame_count = u64::MAX;
        app.auto_analysis_complete = true;

        app.on_tick();
        assert_eq!(app.frame_count, 0);
        assert_eq!(app.cpu_history.len(), 1);
        assert_eq!(app.cpu_history[0], 15.0);

        app.on_tick();
        assert_eq!(app.cpu_history.len(), 2);
    }

    #[test]
    fn e2e_combined_filtering() {
        let mut app = App::new();
        app.app_connections = vec![
            build_app_connection(
                1,
                "chrome.exe",
                10.0,
                100_000_000,
                vec![sample_conn(1, "8.8.8.8")],
            ),
            build_app_connection(
                2,
                "powershell.exe",
                5.0,
                50_000_000,
                vec![sample_conn(2, "10.0.0.1")],
            ),
            build_app_connection(
                3,
                "notepad.exe",
                1.0,
                10_000_000,
                vec![sample_conn(3, "1.1.1.1"), sample_conn(3, "2.2.2.2")],
            ),
        ];
        app.auto_analysis_complete = true;
        app.is_initial_loading = false;

        assert_eq!(app.get_filtered_apps().len(), 3);

        app.search_query = "power".to_string();
        assert_eq!(app.get_filtered_apps().len(), 1);
        assert_eq!(app.get_filtered_apps()[0].process_name, "powershell.exe");

        app.search_query = "8.8.8.8".to_string();
        assert_eq!(app.get_filtered_apps().len(), 1);
        assert_eq!(app.get_filtered_apps()[0].process_name, "chrome.exe");

        app.search_query.clear();
        app.filter_high_risk_only = true;
        let filtered = app.get_filtered_apps();
        for a in &filtered {
            assert!(
                a.risk_level.contains("HIGH") || a.risk_level.contains("CRITICAL"),
                "Expected HIGH/CRITICAL risk, got: {}",
                a.risk_level
            );
        }

        app.filter_high_risk_only = false;
        assert_eq!(app.get_filtered_apps().len(), 3);
    }

    #[test]
    fn e2e_geo_preserves_existing_locations() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let mut app = App::new();
        use crate::app::network::NetworkConnection;
        app.app_connections = vec![AppConnection {
            process_name: "test.exe".to_string(),
            process_path: "C:\\test.exe".to_string(),
            icon: String::new(),
            pid: 1,
            connections: vec![
                NetworkConnection {
                    protocol: "TCP".to_string(),
                    local_address: "0.0.0.0".to_string(),
                    local_port: 1234,
                    foreign_address: "8.8.8.8".to_string(),
                    foreign_port: 443,
                    state: "ESTABLISHED".to_string(),
                    pid: 1,
                    location: Some("US - California".to_string()),
                    isp: Some("Google".to_string()),
                },
                NetworkConnection {
                    protocol: "UDP".to_string(),
                    local_address: "0.0.0.0".to_string(),
                    local_port: 53,
                    foreign_address: "1.1.1.1".to_string(),
                    foreign_port: 53,
                    state: "ESTABLISHED".to_string(),
                    pid: 1,
                    location: None,
                    isp: None,
                },
            ],
            cpu_usage: 5.0,
            memory_usage: 1_000_000,
            risk_level: "LOW".to_string(),
            signature_status: crate::utils::signatures::SignatureStatus::Unknown,
        }];
        app.selected_app_index = 0;
        app.sidebar_focus = crate::app::types::SidebarFocus::Left;

        let before = app.pending_geo_lookups;
        app.trigger_geo_lookup_for_selected_app();

        assert_eq!(app.pending_geo_lookups, before + 1);
    }

    #[test]
    fn e2e_start_batch_analysis() {
        let mut app = App::new();
        app.auto_analysis_complete = true;
        app.is_initial_loading = true;

        app.start_batch_analysis();

        assert!(!app.is_initial_loading);
        assert!(app.auto_analysis_complete);
    }

    #[test]
    fn e2e_icon_cache_persistence() {
        let mut app = App::new();
        app.icon_cache
            .insert_icon("C:\\test.exe", "test_icon".to_string());
        let icon = app.icon_cache.get_icon("C:\\test.exe", "test");
        assert_eq!(icon, "test_icon");

        let icon2 = app.icon_cache.get_icon("C:\\test.exe", "test");
        assert_eq!(icon2, "test_icon");
    }

    #[test]
    fn e2e_keyboard_navigation_with_apps() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        use crate::app::types::SidebarFocus;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut app = App::new();
        app.app_connections = vec![
            build_app_connection(
                1,
                "chrome.exe",
                15.0,
                200_000_000,
                vec![sample_conn(1, "8.8.8.8"), sample_conn(1, "1.1.1.1")],
            ),
            build_app_connection(
                2,
                "firefox.exe",
                10.0,
                150_000_000,
                vec![sample_conn(2, "4.4.4.4")],
            ),
        ];
        app.auto_analysis_complete = true;
        app.is_initial_loading = false;

        fn press(key: KeyCode) -> KeyEvent {
            KeyEvent::new(key, KeyModifiers::empty())
        }

        assert_eq!(app.sidebar_focus, SidebarFocus::Left);
        assert_eq!(app.selected_app_index, 0);

        app.handle_key_event(press(KeyCode::Down));
        assert_eq!(app.selected_app_index, 1);
        assert_eq!(app.get_selected_app().unwrap().process_name, "firefox.exe");

        app.handle_key_event(press(KeyCode::Up));
        assert_eq!(app.selected_app_index, 0);

        app.handle_key_event(press(KeyCode::Enter));
        assert_eq!(app.sidebar_focus, SidebarFocus::Center);
        assert_eq!(app.selected_connection_index, 0);

        app.handle_key_event(press(KeyCode::Down));
        assert_eq!(app.selected_connection_index, 1);

        app.handle_key_event(press(KeyCode::Tab));
        assert_eq!(app.sidebar_focus, SidebarFocus::Right);

        app.handle_key_event(press(KeyCode::Tab));
        assert_eq!(app.sidebar_focus, SidebarFocus::Nav);

        app.handle_key_event(press(KeyCode::Tab));
        assert_eq!(app.sidebar_focus, SidebarFocus::Left);
    }
}
