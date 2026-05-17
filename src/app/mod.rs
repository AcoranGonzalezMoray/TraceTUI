pub mod analysis;
pub mod firewall_service;
pub mod grouping;
pub mod input;
pub mod installation;
pub mod investigation_service;
pub mod io;
pub mod nerdfont;
pub mod network;
pub mod process;
pub mod risk;
pub mod types;
pub mod ui;
use crate::config;
use crate::i18n::{self, Translator};
use crate::services::geoip_service::{GeoInfo, GeoIpService};
use crate::utils::db::Database;
use crate::utils::icon_extractor::IconCache;
pub use investigation_service::InvestigationReport;
pub use io::{restore_terminal, setup_terminal};
use tokio::sync::mpsc;
pub use types::{AppConnection, AppState, FirewallPanel, SidebarFocus};
pub struct App {
    pub should_quit: bool,
    pub current_state: AppState,
    pub sidebar_focus: SidebarFocus,
    pub network_connections: Vec<crate::app::network::NetworkConnection>,
    pub processes: Vec<crate::app::process::ProcessInfo>,
    pub app_connections: Vec<AppConnection>,
    pub selected_app_index: usize,
    pub selected_connection_index: usize,
    pub selected_action_index: usize,
    pub show_confirmation: bool,
    pub confirmation_message: String,
    pub status_message: String,
    pub auto_analysis_complete: bool,
    pub is_initial_loading: bool,
    pub icon_cache: IconCache,
    pub search_query: String,
    pub search_mode: bool,
    pub filter_high_risk_only: bool,
    pub frame_count: u64,
    pub geoip: GeoIpService,
    pub geo_tx: mpsc::UnboundedSender<(u32, String, GeoInfo)>,
    pub geo_rx: mpsc::UnboundedReceiver<(u32, String, GeoInfo)>,
    pub pending_geo_lookups: usize,
    pub investigation_report: Option<InvestigationReport>,
    pub is_investigating: bool,
    pub inv_tx: mpsc::UnboundedSender<InvestigationReport>,
    pub inv_rx: mpsc::UnboundedReceiver<InvestigationReport>,
    pub user_geo: Option<GeoInfo>,
    pub user_info_rx: mpsc::UnboundedReceiver<GeoInfo>,
    pub user_info_tx: mpsc::UnboundedSender<GeoInfo>,
    pub hunter_mode: bool,
    pub database: Database,
    pub data_rx: Option<
        std::sync::mpsc::Receiver<(
            Vec<crate::app::network::NetworkConnection>,
            Vec<crate::app::process::ProcessInfo>,
        )>,
    >,
    pub grouping_rx: Option<std::sync::mpsc::Receiver<Vec<AppConnection>>>,
    pub icon_extraction_rx: Option<std::sync::mpsc::Receiver<(String, String)>>,
    pub show_nerdfont_dialog: bool,
    pub nerdfont_dialog_dismissed: bool,
    pub nerdfont_installing: bool,
    pub nerdfont_install_done: bool,
    pub nerdfont_install_success: bool,
    pub nerdfont_install_message: String,
    pub nerdfont_install_rx: Option<tokio::sync::oneshot::Receiver<String>>,
    pub show_install_dialog: bool,
    pub install_message: String,
    pub is_installing: bool,
    pub install_done: bool,
    pub install_success: bool,
    pub install_log: String,
    pub install_child: Option<tokio::sync::oneshot::Receiver<std::process::Output>>,
    pub show_password_modal: bool,
    pub install_password: String,
    pub install_needs_password: bool,
    pub firewall_mode: bool,
    pub firewall_focus: FirewallPanel,
    pub firewall_connections: Vec<crate::app::network::NetworkConnection>,
    pub firewall_process_name: String,
    pub blocked_ips: Vec<(String, String, String)>,
    pub firewall_conn_index: usize,
    pub firewall_blocked_index: usize,
    pub firewall_action_index: usize,
    pub firewall_conn_checked: Vec<bool>,
    pub firewall_blocked_checked: Vec<bool>,
    pub translator: Translator,
    pub show_language_modal: bool,
    pub language_selection_index: usize,
    pub language_scroll_offset: usize,
    pub show_map: bool,
    pub center_tab: usize,
    pub cpu_history: Vec<f64>,
    pub conn_count_history: Vec<u64>,
    pub analysis_paused: bool,
    pub continuous_refresh_counter: u64,
    pub show_update_dialog: bool,
    pub latest_remote_version: String,
    pub update_rx: Option<std::sync::mpsc::Receiver<String>>,
    pub update_task_rx:
        Option<tokio::sync::mpsc::UnboundedReceiver<crate::app::types::UpdateEvent>>,
    pub is_updating: bool,
    pub update_done: bool,
    pub update_success: bool,
    pub update_message: String,
    pub update_progress: f64,
    pub show_welcome_dialog: bool,
    pub welcome_index: usize,
}
impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let (itx, irx) = mpsc::unbounded_channel();
        let (utx, urx) = mpsc::unbounded_channel();
        let mut app = Self {
            should_quit: false,
            current_state: AppState::Dashboard,
            sidebar_focus: SidebarFocus::Left,
            network_connections: Vec::new(),
            processes: Vec::new(),
            app_connections: Vec::new(),
            selected_app_index: 0,
            selected_connection_index: 0,
            selected_action_index: 0,
            show_confirmation: false,
            confirmation_message: String::new(),
            status_message: String::new(),
            auto_analysis_complete: false,
            is_initial_loading: true,
            data_rx: None,
            grouping_rx: None,
            icon_extraction_rx: None,
            icon_cache: IconCache::new(),
            search_query: String::new(),
            search_mode: false,
            filter_high_risk_only: false,
            frame_count: 0,
            geoip: GeoIpService::new().expect("Failed to initialize GeoIpService"),
            geo_tx: tx,
            geo_rx: rx,
            pending_geo_lookups: 0,
            investigation_report: None,
            is_investigating: false,
            inv_tx: itx,
            inv_rx: irx,
            user_geo: None,
            user_info_rx: urx,
            user_info_tx: utx,
            hunter_mode: false,
            database: Database::new().expect("Failed to init database"),
            show_nerdfont_dialog: false,
            nerdfont_dialog_dismissed: false,
            nerdfont_installing: false,
            nerdfont_install_done: false,
            nerdfont_install_success: false,
            nerdfont_install_message: String::new(),
            nerdfont_install_rx: None,
            show_install_dialog: false,
            install_message: String::new(),
            is_installing: false,
            install_done: false,
            install_success: false,
            install_log: String::new(),
            install_child: None,
            show_password_modal: false,
            install_password: String::new(),
            install_needs_password: false,
            firewall_mode: false,
            firewall_focus: FirewallPanel::Connections,
            firewall_connections: Vec::new(),
            firewall_process_name: String::new(),
            blocked_ips: Vec::new(),
            firewall_conn_index: 0,
            firewall_blocked_index: 0,
            firewall_action_index: 0,
            firewall_conn_checked: Vec::new(),
            firewall_blocked_checked: Vec::new(),
            translator: Translator::new(
                &config::load_language().unwrap_or_else(i18n::detect_system_locale),
            ),
            show_language_modal: false,
            language_selection_index: 0,
            language_scroll_offset: 0,
            show_map: false,
            center_tab: 0,
            cpu_history: Vec::new(),
            conn_count_history: Vec::new(),
            analysis_paused: false,
            continuous_refresh_counter: 0,
            show_update_dialog: false,
            latest_remote_version: String::new(),
            update_rx: None,
            update_task_rx: None,
            is_updating: false,
            update_done: false,
            update_success: false,
            update_message: String::new(),
            update_progress: 0.0,
            show_welcome_dialog: false,
            welcome_index: 0,
        };

        let config_path = crate::config::config_dir().join("config.json");
        let config_exists = config_path.exists();
        let mut config = crate::config::load_config();
        let current_version = env!("CARGO_PKG_VERSION").to_string();

        if config_exists
            && (config.last_version.is_empty() || config.last_version != current_version)
        {
            app.show_welcome_dialog = true;
        }

        config.last_version = current_version;
        crate::config::save_config(&config);

        app
    }
    pub fn get_filtered_apps(&self) -> Vec<&AppConnection> {
        self.app_connections
            .iter()
            .filter(|app| {
                let matches_search = if self.search_query.is_empty() {
                    true
                } else {
                    let q = self.search_query.to_lowercase();
                    let name_match = app.process_name.to_lowercase().contains(&q);
                    let ip_match = app.connections.iter().any(|conn| {
                        conn.foreign_address.to_lowercase().contains(&q)
                            || conn.local_address.to_lowercase().contains(&q)
                    });
                    name_match || ip_match
                };
                let matches_risk = if self.filter_high_risk_only {
                    app.risk_level.contains("HIGH") || app.risk_level.contains("CRITICAL")
                } else {
                    true
                };
                matches_search && matches_risk
            })
            .collect()
    }
    pub fn get_selected_app(&self) -> Option<&AppConnection> {
        let filtered = self.get_filtered_apps();
        filtered.get(self.selected_app_index).copied()
    }
}
