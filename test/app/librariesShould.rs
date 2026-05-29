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
}
