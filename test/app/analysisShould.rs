#[cfg(test)]
mod analysis_tests {
    use crate::app::App;

    #[test]
    fn test_on_tick_increments_frame_count() {
        let mut app = App::new();
        let before = app.frame_count;
        app.on_tick();
        assert_eq!(app.frame_count, before.wrapping_add(1));
    }

    #[test]
    fn test_on_tick_multiple() {
        let mut app = App::new();
        let before = app.frame_count;
        for _ in 0..10 {
            app.on_tick();
        }
        assert_eq!(app.frame_count, before.wrapping_add(10));
    }

    #[test]
    fn test_background_refresh_skips_when_rx_active() {
        let mut app = App::new();
        let (tx, rx) = std::sync::mpsc::channel();
        app.data_rx = Some(rx);
        app.trigger_background_refresh();
        assert!(app.data_rx.is_some());
        drop(tx);
    }

    #[test]
    fn test_trigger_geo_lookup_no_selection() {
        let mut app = App::new();
        let before = app.pending_geo_lookups;
        app.trigger_geo_lookup_for_selected_app();
        assert_eq!(app.pending_geo_lookups, before);
    }

    #[test]
    fn test_start_batch_analysis_sets_initial_loading_false() {
        let mut app = App::new();
        app.is_initial_loading = true;
        app.start_batch_analysis();
        assert!(!app.is_initial_loading);
    }

    #[test]
    fn test_start_batch_analysis_preserves_auto_analysis_complete() {
        let mut app = App::new();
        app.auto_analysis_complete = true;
        app.start_batch_analysis();
        assert!(app.auto_analysis_complete);
    }

    #[test]
    fn test_continuous_refresh_counter_wraps() {
        let mut app = App::new();
        app.continuous_refresh_counter = u64::MAX;
        app.continuous_refresh_counter = app.continuous_refresh_counter.wrapping_add(1);
        assert_eq!(app.continuous_refresh_counter, 0);
    }

    #[test]
    fn test_analysis_paused_blocks_refresh() {
        let mut app = App::new();
        app.auto_analysis_complete = true;
        app.analysis_paused = true;
        let before = app.continuous_refresh_counter;
        app.on_tick();
        assert_eq!(app.continuous_refresh_counter, before);
    }

    #[test]
    fn test_analysis_not_complete_blocks_refresh() {
        let mut app = App::new();
        app.auto_analysis_complete = false;
        app.analysis_paused = false;
        let before = app.continuous_refresh_counter;
        app.on_tick();
        assert_eq!(app.continuous_refresh_counter, before);
    }

    #[test]
    fn test_start_investigation_pauses_analysis() {
        use crate::app::network::NetworkConnection;
        use crate::app::types::AppConnection;
        use crate::utils::signatures::SignatureStatus;

        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
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
                location: None,
                isp: None,
            }],
            cpu_usage: 0.0,
            memory_usage: 0,
            risk_level: "LOW".to_string(),
            signature_status: SignatureStatus::Unknown,
        }];
        app.selected_app_index = 0;
        app.selected_connection_index = 0;
        assert!(!app.analysis_paused);

        app.start_investigation();
        assert!(app.analysis_paused);
    }

    #[test]
    fn test_frame_count_wrapping() {
        let mut app = App::new();
        app.frame_count = u64::MAX;
        app.on_tick();
        assert_eq!(app.frame_count, 0);
    }
}
