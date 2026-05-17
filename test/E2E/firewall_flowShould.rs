#[cfg(test)]
mod e2e_firewall_flow {
    use crate::app::network::NetworkConnection;
    use crate::app::process::ProcessInfo;
    use crate::app::risk::RiskAnalyzer;
    use crate::app::types::{AppConnection, FirewallPanel};
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

    fn build_app(pid: u32, name: &str, conns: Vec<NetworkConnection>) -> AppConnection {
        let proc = ProcessInfo {
            pid,
            name: name.to_string(),
            path: Some(format!("C:\\{}.exe", name)),
            command_line: None,
            cpu_usage: 5.0,
            memory_usage: 1_000_000,
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
            cpu_usage: 5.0,
            memory_usage: 1_000_000,
            risk_level: risk,
            signature_status: crate::utils::signatures::SignatureStatus::Unknown,
        }
    }

    #[test]
    fn e2e_firewall_enter_navigate_exit() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let mut app = App::new();
        let conns = vec![
            sample_conn(1, "10.0.0.1"),
            sample_conn(1, "10.0.0.2"),
            sample_conn(1, "10.0.0.3"),
        ];
        app.app_connections = vec![build_app(1, "test_app", conns)];
        app.auto_analysis_complete = true;
        app.is_initial_loading = false;

        fn press(key: KeyCode) -> KeyEvent {
            KeyEvent::new(key, KeyModifiers::empty())
        }

        app.selected_action_index = 7;
        app.execute_action();
        assert!(app.firewall_mode);
        assert_eq!(app.firewall_focus, FirewallPanel::Connections);
        assert_eq!(app.firewall_connections.len(), 3);
        assert_eq!(app.firewall_conn_checked.len(), 3);

        app.handle_key_event(press(KeyCode::Down));
        assert_eq!(app.firewall_conn_index, 1);
        app.handle_key_event(press(KeyCode::Down));
        assert_eq!(app.firewall_conn_index, 2);

        app.handle_key_event(press(KeyCode::Up));
        assert_eq!(app.firewall_conn_index, 1);

        app.handle_key_event(press(KeyCode::Tab));
        assert_eq!(app.firewall_focus, FirewallPanel::BlockedList);

        app.handle_key_event(press(KeyCode::Tab));
        assert_eq!(app.firewall_focus, FirewallPanel::Actions);

        app.handle_key_event(press(KeyCode::BackTab));
        assert_eq!(app.firewall_focus, FirewallPanel::BlockedList);

        app.handle_key_event(press(KeyCode::Char('q')));
        assert!(!app.firewall_mode);
        assert!(app.firewall_connections.is_empty());
        assert!(app.firewall_process_name.is_empty());
    }

    #[test]
    fn e2e_firewall_toggle_checkboxes() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        let mut app = App::new();
        let conns = vec![sample_conn(1, "10.0.0.1"), sample_conn(1, "10.0.0.2")];
        app.app_connections = vec![build_app(1, "test_app", conns)];
        app.auto_analysis_complete = true;
        app.is_initial_loading = false;

        fn press(key: KeyCode) -> KeyEvent {
            KeyEvent::new(key, KeyModifiers::empty())
        }

        app.selected_action_index = 7;
        app.execute_action();
        assert!(app.firewall_mode);

        assert!(!app.any_conn_checked());

        app.handle_key_event(press(KeyCode::Char(' ')));
        assert!(app.firewall_conn_checked[0]);

        app.handle_key_event(press(KeyCode::Down));
        app.handle_key_event(press(KeyCode::Char(' ')));
        assert!(app.firewall_conn_checked[1]);
        assert!(app.any_conn_checked());

        app.handle_key_event(press(KeyCode::Char(' ')));
        assert!(!app.firewall_conn_checked[1]);
    }

    #[test]
    fn e2e_firewall_no_selection() {
        let mut app = App::new();
        app.auto_analysis_complete = true;
        app.is_initial_loading = false;

        app.selected_action_index = 7;
        app.execute_action();

        assert!(!app.firewall_mode);
    }
}
