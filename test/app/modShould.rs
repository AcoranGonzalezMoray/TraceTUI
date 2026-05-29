#[cfg(test)]
mod app_mod_tests {
    #[test]
    fn test_app_new() {
        let app = crate::app::App::new();
        assert!(!app.ui.should_quit);
        assert_eq!(app.network.selected_app_index, 0);
        assert_eq!(app.network.selected_connection_index, 0);
        assert_eq!(app.ui.selected_action_index, 0);
        assert!(!app.ui.auto_analysis_complete);
        assert!(app.ui.is_initial_loading);
        assert!(!app.ui.show_confirmation);
        assert!(!app.ui.search_mode);
        assert!(!app.ui.filter_high_risk_only);
        assert!(!app.ui.hunter_mode);
        assert!(!app.firewall.firewall_mode);
        assert!(!app.ui.show_map);
        assert!(!app.ui.analysis_paused);
        assert_eq!(app.ui.continuous_refresh_counter, 0);
        assert_eq!(app.geo.pending_geo_lookups, 0);
        assert!(!app.investigation.is_investigating);
        assert!(app.network.network_connections.is_empty());
        assert!(app.network.processes.is_empty());
        assert!(app.network.app_connections.is_empty());
        assert!(app.trend.cpu_history.is_empty());
        assert!(app.trend.conn_count_history.is_empty());
        assert!(!app.update.show_update_dialog);
        assert!(app.update.latest_remote_version.is_empty());
        assert!(app.update.update_rx.is_none());
        assert_eq!(app.ui.current_nav_view, crate::app::types::NavView::Main);
        assert!(!app.ui.nav_sidebar_expanded);
    }

    #[test]
    fn test_default_sidebar_focus() {
        let app = crate::app::App::new();
        assert_eq!(app.ui.sidebar_focus, crate::app::types::SidebarFocus::Left);
    }

    #[test]
    fn test_default_current_state() {
        let app = crate::app::App::new();
        assert_eq!(app.ui.current_state, crate::app::types::AppState::Dashboard);
    }

    #[test]
    fn test_initial_filtered_apps_empty() {
        let app = crate::app::App::new();
        let filtered = app.get_filtered_apps();
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_get_selected_app_none_when_empty() {
        let app = crate::app::App::new();
        assert!(app.get_selected_app().is_none());
    }

    #[cfg(windows)]
    #[test]
    fn test_icon_cache_initialized() {
        let mut app = crate::app::App::new();
        let icon = app.network.icon_cache.get_icon("nonexistent.exe", "test");
        assert_eq!(icon, "tes");
    }

    #[test]
    fn test_install_password_empty() {
        let app = crate::app::App::new();
        assert!(app.install.password.is_empty());
    }

    #[test]
    fn test_install_log_empty() {
        let app = crate::app::App::new();
        assert!(app.install.log.is_empty());
    }

    #[test]
    fn test_status_message_empty() {
        let app = crate::app::App::new();
        assert!(app.ui.status_message.is_empty());
    }

    #[test]
    fn test_translator_initialized() {
        let app = crate::app::App::new();
        assert!(!app.ui.translator.get("app.title").is_empty());
    }

    #[test]
    fn test_database_initialized() {
        let app = crate::app::App::new();
        let blocked = app.database.get_blocked_ips();
        assert!(blocked.is_ok());
    }
}
