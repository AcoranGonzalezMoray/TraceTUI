#[cfg(test)]
mod update_tests {
    use crate::app::states::UpdateState;

    #[test]
    fn test_update_state_new() {
        let state = UpdateState::new();
        assert!(!state.show_update_dialog);
        assert_eq!(state.latest_remote_version, "");
        assert!(state.update_rx.is_none());
        assert!(state.update_task_rx.is_none());
        assert!(!state.is_updating);
        assert!(!state.update_done);
        assert!(!state.update_success);
        assert_eq!(state.update_message, "");
        assert_eq!(state.update_progress, 0.0);
    }

    #[test]
    fn test_update_state_dialog() {
        let mut state = UpdateState::new();
        assert!(!state.show_update_dialog);
        state.show_update_dialog = true;
        assert!(state.show_update_dialog);
        state.show_update_dialog = false;
        assert!(!state.show_update_dialog);
    }

    #[test]
    fn test_update_state_version() {
        let mut state = UpdateState::new();
        state.latest_remote_version = "1.2.3".to_string();
        assert_eq!(state.latest_remote_version, "1.2.3");
    }

    #[test]
    fn test_update_state_progress() {
        let mut state = UpdateState::new();
        assert_eq!(state.update_progress, 0.0);
        state.update_progress = 0.5;
        assert_eq!(state.update_progress, 0.5);
        state.update_progress = 1.0;
        assert_eq!(state.update_progress, 1.0);
    }

    #[test]
    fn test_update_state_message() {
        let mut state = UpdateState::new();
        state.update_message = "Downloading...".to_string();
        assert_eq!(state.update_message, "Downloading...");
    }

    #[test]
    fn test_update_state_status() {
        let mut state = UpdateState::new();
        assert!(!state.is_updating);
        assert!(!state.update_done);
        assert!(!state.update_success);

        state.is_updating = true;
        assert!(state.is_updating);

        state.update_done = true;
        assert!(state.update_done);

        state.update_success = true;
        assert!(state.update_success);
    }
}
