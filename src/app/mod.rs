pub mod analysis;
pub mod containers;
pub mod firewall_service;
pub mod grouping;
pub mod input;
pub mod installation;
pub mod investigation_service;
pub mod io;
pub mod libraries;
pub mod nerdfont;
pub mod network;
pub mod process;
pub mod risk;
pub mod storage;
pub mod types;
pub mod ui;
use crate::config;
use crate::i18n::{self, Translator};
use crate::services::geoip_service::{GeoInfo, GeoIpService};
use crate::tr;
use crate::utils::db::Database;
use crate::utils::icon_extractor::IconCache;
pub use investigation_service::InvestigationReport;
pub use io::{restore_terminal, setup_terminal};
use tokio::sync::mpsc;
pub use types::{AppConnection, AppState, FirewallPanel, NavView, SidebarFocus};
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
    pub current_nav_view: NavView,
    pub nav_sidebar_expanded: bool,
    pub containers: Vec<crate::app::containers::ContainerInfo>,
    pub selected_container_index: usize,
    pub selected_container_action_index: usize,
    pub container_detail_scroll: usize,
    pub containers_loading: bool,
    pub containers_loaded_once: bool,
    pub containers_error: Option<String>,
    pub container_rx: Option<
        std::sync::mpsc::Receiver<Result<Vec<crate::app::containers::ContainerInfo>, String>>,
    >,
    pub container_logs: Vec<String>,
    pub container_logs_loading: bool,
    pub container_logs_rx: Option<std::sync::mpsc::Receiver<Result<Vec<String>, String>>>,
    pub show_container_logs_modal: bool,
    pub show_container_console_modal: bool,
    pub container_logs_scroll: usize,
    pub container_console_input: String,
    pub container_console_output: Vec<String>,
    pub container_console_loading: bool,
    pub container_console_scroll: usize,
    pub container_console_rx: Option<std::sync::mpsc::Receiver<Result<Vec<String>, String>>>,
    pub show_docker_hub_modal: bool,
    pub docker_hub_search: crate::app::containers::DockerHubSearchState,
    pub docker_hub_search_rx: Option<
        std::sync::mpsc::Receiver<Result<Vec<crate::app::containers::DockerHubImage>, String>>,
    >,
    pub docker_hub_create_rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    pub pending_container_action: Option<crate::app::containers::ContainerAction>,
    pub pending_docker_action: Option<crate::app::containers::DockerAction>,
    pub disks: Vec<crate::app::storage::DiskInfo>,
    pub selected_disk_index: usize,
    pub disks_loading: bool,
    pub current_directory: std::path::PathBuf,
    pub file_entries: Vec<crate::app::storage::FileEntry>,
    pub file_scroll: usize,
    pub show_file_viewer: bool,
    pub file_viewer_content: Vec<String>,
    pub file_viewer_scroll: usize,
    pub file_viewer_is_ansi: bool,
    pub storage_focus: usize,
    pub file_search_query: String,
    pub file_search_mode: bool,
    pub file_search_recursive: bool,
    pub file_search_extension_idx: usize,
    pub selected_storage_action_index: usize,
    pub cached_filtered_indices: Vec<usize>,
    pub show_file_search_modal: bool,
    pub file_search_state: crate::app::types::FileSearchState,
    pub file_sort_mode: crate::app::types::FileSortMode,
    pub last_storage_refresh: std::time::Instant,
    pub needs_clear: bool,
    pub search_progress_running: bool,
    pub search_progress_found: usize,
    search_progress_rx: Option<std::sync::mpsc::Receiver<Vec<crate::app::storage::FileEntry>>>,
    search_progress_count: Option<std::sync::Arc<std::sync::atomic::AtomicUsize>>,
    search_progress_abort: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    pub libraries: Vec<crate::app::libraries::LibraryInfo>,
    pub libraries_loading: bool,
    pub selected_library_process_index: usize,
    pub selected_library_index: usize,
    pub library_process_scroll: usize,
    pub library_lib_scroll: usize,
    pub libraries_loaded_once: bool,
    pub library_search_query: String,
    pub library_search_active: bool,
    pub library_risk_filter: Option<String>,
    pub show_hash_info_modal: bool,
    pub show_library_binary_viewer: bool,
    pub library_binary_path: String,
    pub library_binary_hex_lines: Vec<String>,
    pub library_binary_disasm_lines: Vec<String>,
    pub library_binary_scroll: usize,
    pub library_binary_tab: usize,
    libraries_rx: Option<std::sync::mpsc::Receiver<Vec<crate::app::libraries::LibraryInfo>>>,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let (itx, irx) = mpsc::unbounded_channel();
        let (utx, urx) = mpsc::unbounded_channel();
        let detected_locale = config::load_language().unwrap_or_else(i18n::detect_system_locale);

        #[allow(unused_mut)]
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
            translator: Translator::new(&detected_locale),
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
            current_nav_view: NavView::Main,
            nav_sidebar_expanded: false,
            containers: Vec::new(),
            selected_container_index: 0,
            selected_container_action_index: 0,
            container_detail_scroll: 0,
            containers_loading: false,
            containers_loaded_once: false,
            containers_error: None,
            container_rx: None,
            container_logs: Vec::new(),
            container_logs_loading: false,
            container_logs_rx: None,
            show_container_logs_modal: false,
            show_container_console_modal: false,
            container_logs_scroll: 0,
            container_console_input: String::new(),
            container_console_output: Vec::new(),
            container_console_loading: false,
            container_console_scroll: 0,
            container_console_rx: None,
            show_docker_hub_modal: false,
            docker_hub_search: crate::app::containers::DockerHubSearchState::default(),
            docker_hub_search_rx: None,
            docker_hub_create_rx: None,
            pending_container_action: None,
            pending_docker_action: None,
            disks: Vec::new(),
            selected_disk_index: 0,
            disks_loading: false,
            current_directory: std::path::PathBuf::from("/"),
            file_entries: Vec::new(),
            file_scroll: 0,
            show_file_viewer: false,
            file_viewer_content: Vec::new(),
            file_viewer_scroll: 0,
            file_viewer_is_ansi: false,
            storage_focus: 0,
            file_search_query: String::new(),
            file_search_mode: false,
            file_search_recursive: false,
            file_search_extension_idx: 0,
            selected_storage_action_index: 0,
            cached_filtered_indices: Vec::new(),
            file_sort_mode: crate::app::types::FileSortMode::ByName,
            last_storage_refresh: std::time::Instant::now(),
            needs_clear: false,
            show_file_search_modal: false,
            file_search_state: crate::app::types::FileSearchState::default(),
            search_progress_running: false,
            search_progress_found: 0,
            search_progress_rx: None,
            search_progress_count: None,
            search_progress_abort: None,
            libraries: Vec::new(),
            libraries_loading: false,
            selected_library_process_index: 0,
            selected_library_index: 0,
            library_process_scroll: 0,
            library_lib_scroll: 0,
            libraries_loaded_once: false,
            libraries_rx: None,
            library_search_query: String::new(),
            library_search_active: false,
            library_risk_filter: None,
            show_hash_info_modal: false,
            show_library_binary_viewer: false,
            library_binary_path: String::new(),
            library_binary_hex_lines: Vec::new(),
            library_binary_disasm_lines: Vec::new(),
            library_binary_scroll: 0,
            library_binary_tab: 0,
        };

        #[cfg(not(test))]
        {
            let config_path = crate::config::config_dir().join("config.json");
            let config_exists = config_path.exists();
            let mut config = crate::config::load_config();
            let current_version = env!("CARGO_PKG_VERSION").to_string();

            if config_exists
                && (config.last_version.is_empty() || config.last_version != current_version)
            {
                app.show_welcome_dialog = true;
            }

            if config.locale.is_empty() {
                config.locale = detected_locale;
            }
            config.last_version = current_version;
            crate::config::save_config(&config);
        }

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
    pub fn get_selected_container(&self) -> Option<&crate::app::containers::ContainerInfo> {
        self.containers.get(self.selected_container_index)
    }
    pub fn get_selected_disk(&self) -> Option<&crate::app::storage::DiskInfo> {
        self.disks.get(self.selected_disk_index)
    }
    pub fn compute_filtered_indices(&mut self) {
        let ext_idx = self.file_search_extension_idx.min(
            crate::app::storage::FILE_EXTENSION_FILTERS
                .len()
                .saturating_sub(1),
        );
        let query = self.file_search_query.to_lowercase();
        let (_, exts) = crate::app::storage::FILE_EXTENSION_FILTERS[ext_idx];
        self.cached_filtered_indices = if self.file_search_mode {
            self.file_entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    let matches_query = query.is_empty() || e.name.to_lowercase().contains(&query);
                    let matches_ext =
                        exts.is_empty() || exts.contains(&e.extension.to_lowercase().as_str());
                    matches_query && matches_ext
                })
                .map(|(i, _)| i)
                .collect()
        } else {
            (0..self.file_entries.len()).collect()
        };
    }
    pub fn abort_search(&mut self) {
        if let Some(ref abort) = self.search_progress_abort {
            abort.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        if self.search_progress_running {
            self.search_progress_running = false;
            self.search_progress_found = 0;
        }
        self.search_progress_rx = None;
        self.search_progress_count = None;
        self.search_progress_abort = None;
    }
    pub fn refresh_libraries(&mut self) {
        if self.libraries_loading {
            return;
        }

        if self.processes.is_empty() && self.app_connections.is_empty() {
            self.libraries_loading = true;
            self.status_message = tr!(self.translator, "libraries.status.refreshing").to_string();
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let processes = self.processes.clone();
        let app_conns = self.app_connections.clone();
        self.libraries.clear();
        std::thread::spawn(move || {
            crate::app::libraries::inspect_libraries_batched(&processes, &app_conns, tx);
        });
        self.libraries_rx = Some(rx);
        self.libraries_loading = true;
        self.status_message = tr!(self.translator, "libraries.status.refreshing").to_string();
    }

    pub fn tick_libraries(&mut self) {
        self.process_libraries_results();
    }
}
