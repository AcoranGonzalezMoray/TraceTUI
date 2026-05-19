#[cfg(test)]
mod input_tests {
    use crate::app::App;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use std::sync::Mutex;

    static EXPORT_MUTEX: Mutex<()> = Mutex::new(());

    fn press(key: KeyCode) -> KeyEvent {
        KeyEvent::new(key, KeyModifiers::empty())
    }

    #[test]
    fn test_handle_key_quit_via_q() {
        let mut app = App::new();
        app.handle_key_event(press(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn test_handle_key_quit_via_escape() {
        let mut app = App::new();
        app.handle_key_event(press(KeyCode::Esc));
        assert!(app.should_quit);
    }

    #[test]
    fn test_handle_key_ctrl_c_quits() {
        let mut app = App::new();
        app.handle_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.should_quit);
    }

    #[test]
    fn test_handle_key_release_ignored() {
        let mut app = App::new();
        let released = KeyEvent {
            kind: KeyEventKind::Release,
            ..press(KeyCode::Char('q'))
        };
        app.handle_key_event(released);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_toggle_analysis_paused_via_r() {
        let mut app = App::new();
        assert!(!app.analysis_paused);
        app.handle_key_event(press(KeyCode::Char('r')));
        assert!(app.analysis_paused);
        app.handle_key_event(press(KeyCode::Char('r')));
        assert!(!app.analysis_paused);
    }

    #[test]
    fn test_ctrl_r_triggers_batch_analysis() {
        let mut app = App::new();
        app.auto_analysis_complete = true;
        app.handle_key_event(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert!(!app.is_initial_loading);
    }

    fn dismiss_welcome_dialog(app: &mut App) {
        if app.show_welcome_dialog {
            app.handle_key_event(press(KeyCode::Esc));
        }
    }

    #[test]
    fn test_tab_cycles_focus() {
        let mut app = App::new();
        dismiss_welcome_dialog(&mut app);
        use crate::app::types::SidebarFocus;

        assert_eq!(app.sidebar_focus, SidebarFocus::Left);
        app.handle_key_event(press(KeyCode::Tab));
        assert_eq!(app.sidebar_focus, SidebarFocus::Center);
        app.handle_key_event(press(KeyCode::Tab));
        assert_eq!(app.sidebar_focus, SidebarFocus::Right);
        app.handle_key_event(press(KeyCode::Tab));
        assert_eq!(app.sidebar_focus, SidebarFocus::Nav);
        app.handle_key_event(press(KeyCode::Tab));
        assert_eq!(app.sidebar_focus, SidebarFocus::Left);
    }

    #[test]
    fn test_backtab_cycles_focus_reverse() {
        let mut app = App::new();
        dismiss_welcome_dialog(&mut app);
        use crate::app::types::SidebarFocus;

        app.handle_key_event(press(KeyCode::BackTab));
        assert_eq!(app.sidebar_focus, SidebarFocus::Nav);
        app.handle_key_event(press(KeyCode::BackTab));
        assert_eq!(app.sidebar_focus, SidebarFocus::Right);
        app.handle_key_event(press(KeyCode::BackTab));
        assert_eq!(app.sidebar_focus, SidebarFocus::Center);
        app.handle_key_event(press(KeyCode::BackTab));
        assert_eq!(app.sidebar_focus, SidebarFocus::Left);
    }

    #[test]
    fn test_up_down_noop_with_empty_apps() {
        let mut app = App::new();
        app.handle_key_event(press(KeyCode::Up));
        assert_eq!(app.selected_app_index, 0);
        app.handle_key_event(press(KeyCode::Down));
        assert_eq!(app.selected_app_index, 0);
    }

    #[test]
    fn test_search_mode_toggle_via_slash() {
        let mut app = App::new();
        assert!(!app.search_mode);
        app.handle_key_event(press(KeyCode::Char('/')));
        assert!(app.search_mode);
    }

    #[test]
    fn test_hunter_mode_toggle_via_h() {
        let mut app = App::new();
        assert!(!app.hunter_mode);
        app.handle_key_event(press(KeyCode::Char('h')));
        assert!(app.hunter_mode);
        app.handle_key_event(press(KeyCode::Char('h')));
        assert!(!app.hunter_mode);
    }

    #[test]
    fn test_nav_sidebar_toggle_via_m() {
        let mut app = App::new();
        assert!(!app.nav_sidebar_expanded);
        app.handle_key_event(press(KeyCode::Char('m')));
        assert!(app.nav_sidebar_expanded);
        app.handle_key_event(press(KeyCode::Char('m')));
        assert!(!app.nav_sidebar_expanded);
    }

    #[test]
    fn test_nav_view_switching_via_arrows() {
        let mut app = App::new();
        use crate::app::types::{NavView, SidebarFocus};
        app.sidebar_focus = SidebarFocus::Nav;
        
        assert_eq!(app.current_nav_view, NavView::Main);
        app.handle_key_event(press(KeyCode::Down));
        assert_eq!(app.current_nav_view, NavView::TrendGraphs);
        app.handle_key_event(press(KeyCode::Down));
        assert_eq!(app.current_nav_view, NavView::DgaDetector);
        app.handle_key_event(press(KeyCode::Up));
        assert_eq!(app.current_nav_view, NavView::TrendGraphs);
    }

    #[test]
    fn test_filter_toggle_via_f() {
        let mut app = App::new();
        assert!(!app.filter_high_risk_only);
        app.handle_key_event(press(KeyCode::Char('f')));
        assert!(app.filter_high_risk_only);
        app.handle_key_event(press(KeyCode::Char('f')));
        assert!(!app.filter_high_risk_only);
    }

    #[test]
    fn test_center_tab_switch_via_number_keys() {
        let mut app = App::new();
        assert_eq!(app.center_tab, 0);
        app.handle_key_event(press(KeyCode::Char('3')));
        assert_eq!(app.center_tab, 2);
    }

    #[test]
    fn test_firewall_mode_enter_via_b() {
        let mut app = App::new();
        app.handle_key_event(press(KeyCode::Char('b')));
        assert!(!app.firewall_mode);
    }

    #[test]
    fn test_password_modal_preempts_dashboard() {
        let mut app = App::new();
        app.show_password_modal = true;
        app.handle_key_event(press(KeyCode::Char('q')));
        assert!(!app.should_quit);
    }

    #[test]
    fn test_nerdfont_dialog_preempts_dashboard() {
        let mut app = App::new();
        app.show_nerdfont_dialog = true;
        app.handle_key_event(press(KeyCode::Char('q')));
        assert!(!app.should_quit);
    }

    #[test]
    fn test_language_modal_preempts_dashboard() {
        let mut app = App::new();
        app.show_language_modal = true;
        app.handle_key_event(press(KeyCode::Char('q')));
        assert!(!app.should_quit);
    }

    #[test]
    fn test_confirm_dialog_preempts_dashboard() {
        let mut app = App::new();
        app.show_confirmation = true;
        app.confirmation_message = "test".to_string();
        app.handle_key_event(press(KeyCode::Esc));
        assert!(!app.should_quit);
        assert!(!app.show_confirmation);
    }

    #[test]
    fn test_search_mode_escape_clears() {
        let mut app = App::new();
        app.search_mode = true;
        app.search_query = "test".to_string();
        app.handle_key_event(press(KeyCode::Esc));
        assert!(!app.search_mode);
        assert!(app.search_query.is_empty());
    }

    #[test]
    fn test_search_mode_enter_exits_search() {
        let mut app = App::new();
        app.search_mode = true;
        app.handle_key_event(press(KeyCode::Enter));
        assert!(!app.search_mode);
    }

    #[test]
    fn test_search_mode_backspace() {
        let mut app = App::new();
        app.search_mode = true;
        app.search_query = "ab".to_string();
        app.handle_key_event(press(KeyCode::Backspace));
        assert_eq!(app.search_query, "a");
    }

    #[test]
    fn test_search_mode_char_input() {
        let mut app = App::new();
        app.search_mode = true;
        app.handle_key_event(press(KeyCode::Char('x')));
        assert_eq!(app.search_query, "x");
    }

    #[test]
    fn test_mouse_click_outside_modal_handled() {
        let mut app = App::new();
        use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
        app.handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::empty(),
        });
        assert_eq!(app.sidebar_focus as u8, 1);
    }

    #[test]
    fn test_mouse_scroll_down_in_left_panel() {
        let mut app = App::new();
        use crossterm::event::{MouseEvent, MouseEventKind};
        app.handle_mouse_event(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 10,
            row: 5,
            modifiers: KeyModifiers::empty(),
        });
        assert_eq!(app.selected_app_index, 0);
    }

    #[test]
    fn test_execute_action_no_selection() {
        let mut app = App::new();
        app.selected_action_index = 1;
        app.execute_action();
        assert!(!app.show_confirmation);
    }

    #[test]
    fn test_execute_action_kill_process_no_selection() {
        let mut app = App::new();
        app.selected_action_index = 1;
        app.execute_action();
        assert!(!app.show_confirmation);
    }

    #[test]
    fn test_execute_action_kill_all_no_selection() {
        let mut app = App::new();
        app.selected_action_index = 2;
        app.execute_action();
        assert!(!app.show_confirmation);
    }

    #[test]
    fn test_execute_action_search_online_no_selection() {
        let mut app = App::new();
        app.selected_action_index = 3;
        app.execute_action();
    }

    #[test]
    fn test_execute_action_copy_path_no_selection() {
        let mut app = App::new();
        app.selected_action_index = 4;
        app.execute_action();
    }

    #[test]
    fn test_export_to_json_empty() {
        let _lock = EXPORT_MUTEX.lock().unwrap();
        for entry in std::fs::read_dir(".").unwrap().flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("network_analysis_") && name.ends_with(".json") {
                let _ = std::fs::remove_file(&name);
            }
        }
        let mut app = App::new();
        app.export_to_json();
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
        assert!(json.is_array(), "Export should be a JSON array");
        assert_eq!(
            json.as_array().unwrap().len(),
            0,
            "Empty app should export empty array"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_export_to_json_with_data() {
        use crate::app::network::NetworkConnection;
        use crate::app::types::AppConnection;
        use crate::utils::signatures::SignatureStatus;

        let _lock = EXPORT_MUTEX.lock().unwrap();
        for entry in std::fs::read_dir(".").unwrap().flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("network_analysis_") && name.ends_with(".json") {
                let _ = std::fs::remove_file(&name);
            }
        }
        let mut app = App::new();
        app.app_connections = vec![AppConnection {
            process_name: "test.exe".to_string(),
            process_path: "C:\\test.exe".to_string(),
            icon: String::new(),
            pid: 1234,
            connections: vec![NetworkConnection {
                protocol: "TCP".to_string(),
                local_address: "0.0.0.0".to_string(),
                local_port: 12345,
                foreign_address: "8.8.8.8".to_string(),
                foreign_port: 443,
                state: "ESTABLISHED".to_string(),
                pid: 1234,
                location: Some("US".to_string()),
                isp: Some("Google".to_string()),
            }],
            cpu_usage: 12.5,
            memory_usage: 4096,
            risk_level: "LOW".to_string(),
            signature_status: SignatureStatus::Unknown,
        }];
        app.export_to_json();
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
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["pid"], 1234);
        assert_eq!(arr[0]["process_name"], "test.exe");
        assert_eq!(arr[0]["connections"][0]["foreign_address"], "8.8.8.8");
        assert_eq!(arr[0]["connections"][0]["location"], "US");
        assert_eq!(arr[0]["connections"][0]["isp"], "Google");
        assert_eq!(arr[0]["cpu_usage"], 12.5);
        assert_eq!(arr[0]["risk_level"], "LOW");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_execute_action_filter_toggle() {
        let mut app = App::new();
        app.selected_action_index = 6;
        app.execute_action();
        assert!(app.filter_high_risk_only);
    }

    #[test]
    fn test_execute_action_firewall_no_selection() {
        let mut app = App::new();
        app.selected_action_index = 7;
        app.execute_action();
        assert!(!app.firewall_mode);
    }

    #[test]
    fn test_execute_action_language_modal() {
        let mut app = App::new();
        app.selected_action_index = 8;
        app.execute_action();
        assert!(app.show_language_modal);
    }

    #[test]
    fn test_execute_action_pause_resume_toggle() {
        let mut app = App::new();
        app.selected_action_index = 0;
        assert!(!app.analysis_paused);
        app.execute_action();
        assert!(app.analysis_paused);
        app.execute_action();
        assert!(!app.analysis_paused);
    }

    #[test]
    fn test_handle_confirmation_keys_n_cancels() {
        let mut app = App::new();
        app.show_confirmation = true;
        app.confirmation_message = "kill process test".to_string();
        app.handle_key_event(press(KeyCode::Char('n')));
        assert!(!app.show_confirmation);
    }

    #[test]
    fn test_handle_confirmation_keys_esc_cancels() {
        let mut app = App::new();
        app.show_confirmation = true;
        app.confirmation_message = "test".to_string();
        app.handle_key_event(press(KeyCode::Esc));
        assert!(!app.show_confirmation);
    }

    #[test]
    fn test_handle_password_keys_enter_empty_password() {
        let mut app = App::new();
        app.show_password_modal = true;
        app.handle_key_event(press(KeyCode::Enter));
        assert!(app.show_password_modal);
    }

    #[test]
    fn test_handle_password_keys_esc_cancels() {
        let mut app = App::new();
        app.show_password_modal = true;
        app.handle_key_event(press(KeyCode::Esc));
        assert!(!app.show_password_modal);
    }

    #[test]
    fn test_handle_install_dialog_keys_esc_cancels() {
        let mut app = App::new();
        app.show_install_dialog = true;
        app.handle_key_event(press(KeyCode::Esc));
        assert!(!app.show_install_dialog);
    }

    #[test]
    fn test_toggle_selected_conn_checkbox() {
        let mut app = App::new();
        app.firewall_conn_checked = vec![false, true, false];
        app.firewall_conn_index = 0;
        app.toggle_selected_conn_checkbox();
        assert!(app.firewall_conn_checked[0]);
        app.firewall_conn_index = 2;
        app.toggle_selected_conn_checkbox();
        assert!(app.firewall_conn_checked[2]);
    }

    #[test]
    fn test_any_conn_checked_none() {
        let app = App::new();
        assert!(!app.any_conn_checked());
    }

    #[test]
    fn test_any_conn_checked_some() {
        let mut app = App::new();
        app.firewall_conn_checked = vec![false, true];
        assert!(app.any_conn_checked());
    }

    #[test]
    fn test_any_blocked_checked_none() {
        let app = App::new();
        assert!(!app.any_blocked_checked());
    }

    #[test]
    fn test_any_blocked_checked_some() {
        let mut app = App::new();
        app.firewall_blocked_checked = vec![true];
        assert!(app.any_blocked_checked());
    }

    #[test]
    fn test_mouse_event_blocked_by_modals() {
        let mut app = App::new();
        app.show_language_modal = true;
        use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
        app.handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 50,
            row: 10,
            modifiers: KeyModifiers::empty(),
        });
        assert_eq!(app.sidebar_focus as u8, 1);
    }

    #[test]
    fn test_update_dialog_blocks_dashboard() {
        let mut app = App::new();
        app.show_update_dialog = true;
        app.handle_key_event(press(KeyCode::Char('q')));
        assert!(!app.should_quit);
    }

    #[test]
    fn test_update_dialog_esc_dismisses() {
        let mut app = App::new();
        app.show_update_dialog = true;
        app.handle_key_event(press(KeyCode::Esc));
        assert!(!app.show_update_dialog);
    }

    #[test]
    fn test_update_dialog_q_dismisses() {
        let mut app = App::new();
        app.show_update_dialog = true;
        app.handle_key_event(press(KeyCode::Char('q')));
        assert!(!app.show_update_dialog);
    }

    #[tokio::test]
    async fn test_update_dialog_enter_starts_update() {
        let mut app = App::new();
        app.show_welcome_dialog = false;
        app.show_update_dialog = true;
        app.latest_remote_version = "1.1.0".to_string();
        app.handle_key_event(press(KeyCode::Enter));
        assert!(app.is_updating);
        assert!(app.show_update_dialog);
    }

    #[test]
    fn test_update_dialog_blocked_by_modals() {
        let mut app = App::new();
        app.show_update_dialog = true;
        use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
        app.handle_mouse_event(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 50,
            row: 10,
            modifiers: KeyModifiers::empty(),
        });
        assert_eq!(app.sidebar_focus as u8, 1);
    }

    fn sample_app_conn(pid: u32, name: &str, num_conns: u8) -> crate::app::types::AppConnection {
        use crate::app::network::NetworkConnection;
        use crate::utils::signatures::SignatureStatus;
        crate::app::types::AppConnection {
            process_name: name.to_string(),
            process_path: format!("C:\\{}.exe", name),
            icon: String::new(),
            pid,
            connections: (0..num_conns)
                .map(|i| NetworkConnection {
                    protocol: "TCP".to_string(),
                    local_address: "0.0.0.0".to_string(),
                    local_port: pid as u16 * 100 + i as u16,
                    foreign_address: format!("{}.0.0.{}", i + 1, i + 1),
                    foreign_port: 80 + i as u16,
                    state: "ESTABLISHED".to_string(),
                    pid,
                    location: None,
                    isp: None,
                })
                .collect(),
            cpu_usage: 10.0,
            memory_usage: 1000,
            risk_level: "LOW".to_string(),
            signature_status: SignatureStatus::Unknown,
        }
    }

    #[test]
    fn test_investigation_blocks_center_keyboard_scroll() {
        use crate::app::types::SidebarFocus;
        let mut app = App::new();
        app.app_connections = vec![sample_app_conn(1, "app1", 3)];
        app.selected_app_index = 0;
        app.selected_connection_index = 2;
        app.is_investigating = true;
        app.sidebar_focus = SidebarFocus::Center;

        app.handle_key_event(press(KeyCode::Up));
        assert_eq!(app.selected_connection_index, 2);

        app.handle_key_event(press(KeyCode::Down));
        assert_eq!(app.selected_connection_index, 2);
    }

    #[test]
    fn test_investigation_blocks_center_mouse_scroll() {
        use crate::app::types::SidebarFocus;
        use crossterm::event::{MouseEvent, MouseEventKind};
        let mut app = App::new();
        app.app_connections = vec![sample_app_conn(1, "app1", 3)];
        app.selected_app_index = 0;
        app.selected_connection_index = 2;
        app.is_investigating = true;
        app.sidebar_focus = SidebarFocus::Center;

        app.handle_mouse_event(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 10,
            row: 5,
            modifiers: KeyModifiers::empty(),
        });
        assert_eq!(app.selected_connection_index, 2);

        app.handle_mouse_event(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 10,
            row: 5,
            modifiers: KeyModifiers::empty(),
        });
        assert_eq!(app.selected_connection_index, 2);
    }

    #[test]
    fn test_investigation_blocks_left_mouse_scroll() {
        use crate::app::types::SidebarFocus;
        use crossterm::event::{MouseEvent, MouseEventKind};
        let mut app = App::new();
        app.app_connections = vec![sample_app_conn(1, "app1", 1), sample_app_conn(2, "app2", 1)];
        app.selected_app_index = 0;
        app.is_investigating = true;
        app.sidebar_focus = SidebarFocus::Left;

        app.handle_mouse_event(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 5,
            row: 5,
            modifiers: KeyModifiers::empty(),
        });
        assert_eq!(app.selected_app_index, 0);
    }

    #[test]
    fn test_investigation_esc_dismiss_resumes_analysis() {
        let mut app = App::new();
        app.is_investigating = true;
        app.analysis_paused = true;

        app.handle_key_event(press(KeyCode::Esc));
        assert!(!app.is_investigating);
        assert!(app.investigation_report.is_none());
        assert!(!app.analysis_paused);
    }

    #[test]
    fn test_investigation_q_dismiss_resumes_analysis() {
        let mut app = App::new();
        app.investigation_report = Some(
            crate::app::investigation_service::InvestigationReport::new("8.8.8.8".to_string(), 443),
        );
        app.analysis_paused = true;

        app.handle_key_event(press(KeyCode::Char('q')));
        assert!(app.investigation_report.is_none());
        assert!(!app.analysis_paused);
    }

    #[test]
    fn test_mouse_scroll_in_firewall_mode() {
        let mut app = App::new();
        app.firewall_mode = true;
        app.firewall_connections = vec![crate::app::network::NetworkConnection {
            protocol: "TCP".to_string(),
            local_address: "0.0.0.0".to_string(),
            local_port: 0,
            foreign_address: "1.2.3.4".to_string(),
            foreign_port: 80,
            state: "ESTABLISHED".to_string(),
            pid: 0,
            location: None,
            isp: None,
        }];
        app.firewall_conn_checked = vec![false];
        use crossterm::event::{MouseEvent, MouseEventKind};
        app.handle_mouse_event(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 10,
            row: 5,
            modifiers: KeyModifiers::empty(),
        });
        assert_eq!(app.firewall_conn_index, 0);
    }
}
