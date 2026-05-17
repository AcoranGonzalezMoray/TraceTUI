use std::path::PathBuf;
use std::time::Duration;
pub const TICK_RATE_MS: u64 = 250;
pub const AUTO_ANALYZE_SLEEP_MS: u64 = 60;
pub const HTTP_TIMEOUT_SECS: u64 = 10;
pub const WHOIS_TIMEOUT_SECS: u64 = 5;
pub const WHOIS_MAX_LINES: usize = 15;
pub const GEOIP_RATE_LIMIT: u32 = 45;
pub const GEOIP_RATE_WINDOW_SECS: u64 = 60;
pub const LRU_CACHE_SIZE: usize = 100;
pub const RISK_CONN_HIGH: usize = 10;
pub const RISK_CONN_MEDIUM: usize = 5;
pub const RISK_CONN_LOW: usize = 1;
pub const RISK_CPU_THRESHOLD: f32 = 50.0;
pub const RISK_MEM_HIGH: u64 = 1024 * 1024 * 1024;
pub const RISK_MEM_MEDIUM: u64 = 512 * 1024 * 1024;
pub const MAX_MEMORY_BYTES: f64 = 16.0 * 1024.0 * 1024.0 * 1024.0;
pub const INV_RISK_BASE: u8 = 5;
pub const INV_RISK_DOMAIN_MISMATCH: u8 = 25;
pub const INV_RISK_NO_REVERSE_DNS: u8 = 15;
pub const INV_RISK_PROXY: u8 = 45;
pub const INV_RISK_HOSTING: u8 = 20;
pub const INV_RISK_MOBILE: u8 = 5;
pub const INV_RISK_HIGH_LATENCY: u8 = 10;
pub const INV_RISK_LATENCY_THRESHOLD_MS: u32 = 300;
pub const RISK_FACTOR_PROXY: &str = "Proxy/VPN detected";
pub const RISK_FACTOR_HOSTING: &str = "Hosting/Data Center IP";
pub const RISK_FACTOR_MOBILE: &str = "Mobile network connection";
pub const SIDEBAR_LEFT_PCT: u16 = 25;
pub const CENTER_PANEL_PCT: u16 = 50;
pub const SIDEBAR_RIGHT_PCT: u16 = 25;
pub const FIREWALL_COL_PCT: u16 = 33;
pub const SEARCH_BAR_PCT: u16 = 40;
pub const SCROLLBAR_WIDTH: u16 = 1;
pub const FIREWALL_CLICK_CONN_BOUNDARY: u16 = 33;
pub const FIREWALL_CLICK_BLOCKED_BOUNDARY: u16 = 66;
pub const CPU_HISTORY_MAX: usize = 60;
pub const CONN_HISTORY_MAX: usize = 120;
pub const REFRESH_COUNTER_THRESHOLD: u64 = 60;
pub const DEFAULT_TERM_WIDTH: u16 = 100;
pub const DEFAULT_TERM_HEIGHT: u16 = 30;
#[cfg(windows)]
pub const ICON_EXTRACTOR_SCRIPT: &str = "scripts/icon_extractor.ps1";
#[cfg(windows)]
pub const ICON_EXTRACTOR_WIDTH: &str = "24";
pub const FIREWALL_RULE_PREFIX: &str = "TraceTUI_Block_";
pub const ACTION_COUNT: usize = 9;
pub const LANGUAGE_VISIBLE_ITEMS: usize = 10;
pub const KNOWN_SAFE_PROCESSES: &[&str] = &[
    "svchost", "explorer", "services", "lsass", "wininit", "smss", "csrss", "widgets", "msedge",
    "chrome", "firefox",
];
pub const SUSPICIOUS_PROCESS_NAMES: &[&str] = &[
    "powershell",
    "cmd",
    "wscript",
    "cscript",
    "regsvr32",
    "rundll32",
    "mshta",
    "wmic",
    "certutil",
    "vssadmin",
    "psh",
    "sc",
];
pub const DOMAIN_ALLOWLIST: &[&str] = &["microsoft", "akamai", "cloudfront", "google"];
pub const DB_FILENAME: &str = "tracetui.db";
pub fn tick_rate() -> Duration {
    Duration::from_millis(TICK_RATE_MS)
}
pub fn auto_analyze_sleep() -> Duration {
    Duration::from_millis(AUTO_ANALYZE_SLEEP_MS)
}
pub fn http_timeout() -> Duration {
    Duration::from_secs(HTTP_TIMEOUT_SECS)
}

pub(crate) fn config_dir() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".config")
            })
    } else {
        std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".config")
            })
    };
    base.join("tracetui")
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub locale: String,
    #[serde(default)]
    pub last_version: String,
}

pub fn save_config(config: &AppConfig) {
    let dir = config_dir();
    let path = dir.join("config.json");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("Failed to create config dir: {}", e);
        return;
    }
    if let Ok(content) = serde_json::to_string_pretty(config) {
        if let Err(e) = std::fs::write(&path, content) {
            eprintln!("Failed to save config: {}", e);
        }
    }
}

pub fn load_config() -> AppConfig {
    let path = config_dir().join("config.json");
    let mut config = AppConfig::default();
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(parsed) = serde_json::from_str::<AppConfig>(&content) {
            config = parsed;
        }
    }
    config
}

pub fn save_language(locale: &str) {
    let mut config = load_config();
    config.locale = locale.to_string();
    save_config(&config);
}

pub fn load_language() -> Option<String> {
    let config = load_config();
    if config.locale.is_empty() {
        None
    } else {
        Some(config.locale)
    }
}
