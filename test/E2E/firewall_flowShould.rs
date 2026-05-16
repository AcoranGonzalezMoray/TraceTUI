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

    /// E2E: Enter firewall mode from a selected app, navigate panels, toggle checkboxes
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

        // Enter firewall mode via action index 7
        app.selected_action_index = 7;
        app.execute_action();
        assert!(app.firewall_mode);
        assert_eq!(app.firewall_focus, FirewallPanel::Connections);
        assert_eq!(app.firewall_connections.len(), 3);
        assert_eq!(app.firewall_conn_checked.len(), 3);

        // Navigate connections with Down key
        app.handle_key_event(press(KeyCode::Down));
        assert_eq!(app.firewall_conn_index, 1);
        app.handle_key_event(press(KeyCode::Down));
        assert_eq!(app.firewall_conn_index, 2);

        // Up goes back
        app.handle_key_event(press(KeyCode::Up));
        assert_eq!(app.firewall_conn_index, 1);

        // Tab to BlockedList panel
        app.handle_key_event(press(KeyCode::Tab));
        assert_eq!(app.firewall_focus, FirewallPanel::BlockedList);

        // Tab to Actions panel
        app.handle_key_event(press(KeyCode::Tab));
        assert_eq!(app.firewall_focus, FirewallPanel::Actions);

        // BackTab reverses
        app.handle_key_event(press(KeyCode::BackTab));
        assert_eq!(app.firewall_focus, FirewallPanel::BlockedList);

        // Exit firewall mode via Q
        app.handle_key_event(press(KeyCode::Char('q')));
        assert!(!app.firewall_mode);
        assert!(app.firewall_connections.is_empty());
        assert!(app.firewall_process_name.is_empty());
    }

    /// E2E: Toggle connection checkboxes and verify checked state
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

        // Initially none checked
        assert!(!app.any_conn_checked());

        // Space toggles checkbox at current index
        app.handle_key_event(press(KeyCode::Char(' ')));
        assert!(app.firewall_conn_checked[0]);

        // Down + Space toggles second checkbox
        app.handle_key_event(press(KeyCode::Down));
        app.handle_key_event(press(KeyCode::Char(' ')));
        assert!(app.firewall_conn_checked[1]);
        assert!(app.any_conn_checked());

        // Space again untoggles
        app.handle_key_event(press(KeyCode::Char(' ')));
        assert!(!app.firewall_conn_checked[1]);
    }

    /// E2E: Firewall mode with no connections selected from an app
    #[test]
    fn e2e_firewall_no_selection() {
        let mut app = App::new();
        app.auto_analysis_complete = true;
        app.is_initial_loading = false;

        // No apps selected
        app.selected_action_index = 7;
        app.execute_action();
        // Should not enter firewall mode without a selected app
        assert!(!app.firewall_mode);
    }
}
