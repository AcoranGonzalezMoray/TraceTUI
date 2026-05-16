#[cfg(test)]
mod config_tests {
    use crate::config;

    #[test]
    fn test_constants_exist() {
        assert_eq!(config::TICK_RATE_MS, 250);
        assert_eq!(config::AUTO_ANALYZE_SLEEP_MS, 60);
        assert_eq!(config::HTTP_TIMEOUT_SECS, 10);
        assert_eq!(config::WHOIS_TIMEOUT_SECS, 5);
        assert_eq!(config::WHOIS_MAX_LINES, 15);
        assert_eq!(config::GEOIP_RATE_LIMIT, 45);
        assert_eq!(config::GEOIP_RATE_WINDOW_SECS, 60);
        assert_eq!(config::LRU_CACHE_SIZE, 100);
    }

    #[test]
    fn test_risk_constants() {
        assert_eq!(config::RISK_CONN_HIGH, 10);
        assert_eq!(config::RISK_CONN_MEDIUM, 5);
        assert_eq!(config::RISK_CONN_LOW, 1);
        assert_eq!(config::RISK_CPU_THRESHOLD, 50.0);
        assert_eq!(config::RISK_MEM_HIGH, 1024 * 1024 * 1024);
        assert_eq!(config::RISK_MEM_MEDIUM, 512 * 1024 * 1024);
    }

    #[test]
    fn test_investigation_constants() {
        assert_eq!(config::INV_RISK_BASE, 5);
        assert_eq!(config::INV_RISK_DOMAIN_MISMATCH, 25);
        assert_eq!(config::INV_RISK_NO_REVERSE_DNS, 15);
        assert_eq!(config::INV_RISK_PROXY, 45);
        assert_eq!(config::INV_RISK_HOSTING, 20);
        assert_eq!(config::INV_RISK_MOBILE, 5);
        assert_eq!(config::INV_RISK_HIGH_LATENCY, 10);
        assert_eq!(config::INV_RISK_LATENCY_THRESHOLD_MS, 300);
    }

    #[test]
    fn test_layout_constants() {
        assert_eq!(config::SIDEBAR_LEFT_PCT, 25);
        assert_eq!(config::CENTER_PANEL_PCT, 50);
        assert_eq!(config::SIDEBAR_RIGHT_PCT, 25);
        assert_eq!(config::FIREWALL_COL_PCT, 33);
        assert_eq!(config::SEARCH_BAR_PCT, 40);
        assert_eq!(config::SCROLLBAR_WIDTH, 1);
    }

    #[test]
    fn test_click_boundaries() {
        assert_eq!(config::FIREWALL_CLICK_CONN_BOUNDARY, 33);
        assert_eq!(config::FIREWALL_CLICK_BLOCKED_BOUNDARY, 66);
    }

    #[test]
    fn test_history_constants() {
        assert_eq!(config::CPU_HISTORY_MAX, 60);
        assert_eq!(config::CONN_HISTORY_MAX, 120);
        assert_eq!(config::REFRESH_COUNTER_THRESHOLD, 60);
    }

    #[test]
    fn test_default_terminal_size() {
        assert_eq!(config::DEFAULT_TERM_WIDTH, 100);
        assert_eq!(config::DEFAULT_TERM_HEIGHT, 30);
    }

    #[cfg(windows)]
    #[test]
    fn test_icon_settings() {
        assert_eq!(config::ICON_EXTRACTOR_SCRIPT, "scripts/icon_extractor.ps1");
        assert_eq!(config::ICON_EXTRACTOR_WIDTH, "24");
    }

    #[test]
    fn test_firewall_prefix() {
        assert_eq!(config::FIREWALL_RULE_PREFIX, "TraceTUI_Block_");
    }

    #[test]
    fn test_action_count() {
        assert_eq!(config::ACTION_COUNT, 9);
    }

    #[test]
    fn test_language_visible_items() {
        assert_eq!(config::LANGUAGE_VISIBLE_ITEMS, 10);
    }

    #[test]
    fn test_known_safe_processes() {
        assert!(config::KNOWN_SAFE_PROCESSES.contains(&"svchost"));
        assert!(config::KNOWN_SAFE_PROCESSES.contains(&"explorer"));
        assert!(config::KNOWN_SAFE_PROCESSES.contains(&"chrome"));
    }

    #[test]
    fn test_suspicious_process_names() {
        assert!(config::SUSPICIOUS_PROCESS_NAMES.contains(&"powershell"));
        assert!(config::SUSPICIOUS_PROCESS_NAMES.contains(&"cmd"));
    }

    #[test]
    fn test_domain_allowlist() {
        assert!(config::DOMAIN_ALLOWLIST.contains(&"microsoft"));
        assert!(config::DOMAIN_ALLOWLIST.contains(&"google"));
    }

    #[test]
    fn test_db_filename() {
        assert_eq!(config::DB_FILENAME, "tracetui.db");
    }

    #[test]
    fn test_tick_rate_duration() {
        let d = config::tick_rate();
        assert_eq!(d.as_millis(), 250);
    }

    #[test]
    fn test_auto_analyze_sleep_duration() {
        let d = config::auto_analyze_sleep();
        assert_eq!(d.as_millis(), 60);
    }

    #[test]
    fn test_http_timeout_duration() {
        let d = config::http_timeout();
        assert_eq!(d.as_secs(), 10);
    }

    #[test]
    fn test_save_and_load_language() {
        let dir = config::config_dir();
        let path = dir.join("config.json");
        let _ = std::fs::remove_file(&path);
        config::save_language("en");
        assert!(path.exists(), "config file should exist after save");
        let loaded = config::load_language();
        assert!(loaded.is_some(), "should load saved language");
    }
}
