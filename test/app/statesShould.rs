#[cfg(test)]
mod states_tests {
    use crate::app::states::*;
    use crate::app::types::{AppState, FileSortMode, FirewallPanel, NavView, SidebarFocus};
    use std::path::PathBuf;

    #[test]
    fn test_install_state_new_defaults() {
        let s = InstallState::new();
        assert!(!s.show_dialog);
        assert!(s.message.is_empty());
        assert!(!s.installing);
        assert!(!s.done);
        assert!(!s.success);
        assert!(s.log.is_empty());
        assert!(s.child.is_none());
        assert!(!s.show_password_modal);
        assert!(s.password.is_empty());
        assert!(!s.needs_password);
    }

    #[test]
    fn test_nerd_font_state_new_defaults() {
        let s = NerdFontState::new();
        assert!(!s.show_dialog);
        assert!(!s.dialog_dismissed);
        assert!(!s.installing);
        assert!(!s.install_done);
        assert!(!s.install_success);
        assert!(s.install_message.is_empty());
        assert!(s.install_rx.is_none());
    }

    #[test]
    fn test_ui_state_new_defaults() {
        let translator = crate::i18n::Translator::new("en");
        let s = UiState::new(translator);
        assert!(!s.should_quit);
        assert_eq!(s.current_state, AppState::Dashboard);
        assert_eq!(s.sidebar_focus, SidebarFocus::Left);
        assert_eq!(s.frame_count, 0);
        assert!(!s.needs_clear);
        assert!(s.search_query.is_empty());
        assert!(!s.search_mode);
        assert!(!s.filter_high_risk_only);
        assert!(s.status_message.is_empty());
        assert!(!s.show_confirmation);
        assert!(s.confirmation_message.is_empty());
        assert!(!s.auto_analysis_complete);
        assert!(s.is_initial_loading);
        assert!(!s.analysis_paused);
        assert_eq!(s.continuous_refresh_counter, 0);
        assert_eq!(s.center_tab, 0);
        assert_eq!(s.current_nav_view, NavView::Main);
        assert!(!s.nav_sidebar_expanded);
        assert_eq!(s.selected_action_index, 0);
        assert!(!s.show_map);
        assert!(!s.show_language_modal);
        assert_eq!(s.language_selection_index, 0);
        assert_eq!(s.language_scroll_offset, 0);
        assert!(!s.show_welcome_dialog);
        assert_eq!(s.welcome_index, 0);
        assert!(!s.show_file_search_modal);
        assert_eq!(s.file_search_state.query, "");
        assert!(!s.file_search_state.recursive);
        assert_eq!(s.file_search_state.extension_idx, 0);
        assert_eq!(s.file_search_state.focused_field, 0);
        assert!(!s.translator.get("app.title").is_empty());
        assert!(!s.hunter_mode);
    }

    #[test]
    fn test_network_data_state_new_defaults() {
        let mut s = NetworkDataState::new();
        assert!(s.network_connections.is_empty());
        assert!(s.processes.is_empty());
        assert!(s.app_connections.is_empty());
        assert_eq!(s.selected_app_index, 0);
        assert_eq!(s.selected_connection_index, 0);
        let icon = s.icon_cache.get_icon("nonexistent.exe", "test");
        assert_eq!(icon, "tes");
        assert!(s.data_rx.is_none());
        assert!(s.grouping_rx.is_none());
        assert!(s.icon_extraction_rx.is_none());
        assert!(s.cached_filtered_indices.is_empty());
    }

    #[test]
    fn test_geo_state_new_defaults() {
        let s = GeoState::new();
        assert_eq!(s.pending_geo_lookups, 0);
        assert!(s.user_geo.is_none());
    }

    #[test]
    fn test_investigation_state_new_defaults() {
        let s = InvestigationState::new();
        assert!(s.investigation_report.is_none());
        assert!(!s.is_investigating);
    }

    #[test]
    fn test_firewall_state_new_defaults() {
        let s = FirewallState::new();
        assert!(!s.firewall_mode);
        assert_eq!(s.firewall_focus, FirewallPanel::Connections);
        assert!(s.firewall_connections.is_empty());
        assert!(s.firewall_process_name.is_empty());
        assert!(s.blocked_ips.is_empty());
        assert_eq!(s.firewall_conn_index, 0);
        assert_eq!(s.firewall_blocked_index, 0);
        assert_eq!(s.firewall_action_index, 0);
        assert!(s.firewall_conn_checked.is_empty());
        assert!(s.firewall_blocked_checked.is_empty());
    }

    #[test]
    fn test_update_state_new_defaults() {
        let s = UpdateState::new();
        assert!(!s.show_update_dialog);
        assert!(s.latest_remote_version.is_empty());
        assert!(s.update_rx.is_none());
        assert!(s.update_task_rx.is_none());
        assert!(!s.is_updating);
        assert!(!s.update_done);
        assert!(!s.update_success);
        assert!(s.update_message.is_empty());
        assert_eq!(s.update_progress, 0.0);
    }

    #[test]
    fn test_storage_state_new_defaults() {
        let s = StorageState::new();
        assert!(s.disks.is_empty());
        assert_eq!(s.selected_disk_index, 0);
        assert!(!s.disks_loading);
        assert_eq!(s.current_directory, PathBuf::from("/"));
        assert!(s.file_entries.is_empty());
        assert_eq!(s.file_scroll, 0);
        assert!(!s.show_file_viewer);
        assert!(s.file_viewer_content.is_empty());
        assert_eq!(s.file_viewer_scroll, 0);
        assert!(!s.file_viewer_is_ansi);
        assert_eq!(s.storage_focus, 0);
        assert!(s.file_search_query.is_empty());
        assert!(!s.file_search_mode);
        assert!(!s.file_search_recursive);
        assert_eq!(s.file_search_extension_idx, 0);
        assert_eq!(s.selected_storage_action_index, 0);
        assert_eq!(s.file_sort_mode, FileSortMode::ByName);
        assert!(!s.search_progress_running);
        assert_eq!(s.search_progress_found, 0);
        assert!(s.search_progress_rx.is_none());
        assert!(s.search_progress_count.is_none());
        assert!(s.search_progress_abort.is_none());
    }

    #[test]
    fn test_container_state_new_defaults() {
        let s = ContainerState::new();
        assert!(s.containers.is_empty());
        assert_eq!(s.selected_container_index, 0);
        assert_eq!(s.selected_container_action_index, 0);
        assert_eq!(s.container_detail_scroll, 0);
        assert!(!s.containers_loading);
        assert!(!s.containers_loaded_once);
        assert!(s.containers_error.is_none());
        assert!(s.container_rx.is_none());
        assert!(s.container_logs.is_empty());
        assert!(!s.container_logs_loading);
        assert!(s.container_logs_rx.is_none());
        assert!(!s.show_container_logs_modal);
        assert!(!s.show_container_console_modal);
        assert_eq!(s.container_logs_scroll, 0);
        assert!(s.container_console_input.is_empty());
        assert!(s.container_console_output.is_empty());
        assert!(!s.container_console_loading);
        assert_eq!(s.container_console_scroll, 0);
        assert!(s.container_console_rx.is_none());
        assert!(!s.show_docker_hub_modal);
        assert!(s.docker_hub_search.search_query.is_empty());
        assert!(s.docker_hub_search.results.is_empty());
        assert_eq!(s.docker_hub_search.selected_result_index, 0);
        assert!(s.docker_hub_search.container_name.is_empty());
        assert!(s.docker_hub_search.ports.is_empty());
        assert!(s.docker_hub_search.env_vars.is_empty());
        assert_eq!(s.docker_hub_search.focused_field, 0);
        assert!(s.docker_hub_search_rx.is_none());
        assert!(s.docker_hub_create_rx.is_none());
        assert!(s.pending_container_action.is_none());
        assert!(s.pending_docker_action.is_none());
    }

    #[test]
    fn test_library_state_new_defaults() {
        let s = LibraryState::new();
        assert!(s.libraries.is_empty());
        assert!(!s.libraries_loading);
        assert_eq!(s.selected_library_process_index, 0);
        assert_eq!(s.selected_library_index, 0);
        assert_eq!(s.library_process_scroll, 0);
        assert_eq!(s.library_lib_scroll, 0);
        assert!(!s.libraries_loaded_once);
        assert!(s.library_search_query.is_empty());
        assert!(!s.library_search_active);
        assert!(s.library_risk_filter.is_none());
        assert!(!s.show_hash_info_modal);
        assert!(!s.show_library_binary_viewer);
        assert!(s.library_binary_path.is_empty());
        assert!(s.library_binary_hex_lines.is_empty());
        assert!(s.library_binary_disasm_lines.is_empty());
        assert_eq!(s.library_binary_scroll, 0);
        assert_eq!(s.library_binary_tab, 0);
        assert!(s.libraries_rx.is_none());
    }

    #[test]
    fn test_trend_state_new_defaults() {
        let s = TrendState::new();
        assert!(s.cpu_history.is_empty());
        assert!(s.conn_count_history.is_empty());
    }

    #[test]
    fn test_all_state_constructors() {
        let _ = InstallState::new();
        let _ = NerdFontState::new();
        let _ = UiState::new(crate::i18n::Translator::new("en"));
        let _ = NetworkDataState::new();
        let _ = GeoState::new();
        let _ = InvestigationState::new();
        let _ = FirewallState::new();
        let _ = UpdateState::new();
        let _ = StorageState::new();
        let _ = ContainerState::new();
        let _ = LibraryState::new();
        let _ = TrendState::new();
    }
}
