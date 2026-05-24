#![allow(dead_code)]

use crate::app::containers::{ContainerInfo, ContainerAction, DockerAction, DockerHubSearchState};
use crate::app::libraries::LibraryInfo;
use crate::app::network::NetworkConnection;
use crate::app::process::ProcessInfo;
use crate::app::storage::{DiskInfo, FileEntry};
use crate::app::types::{AppConnection, AppState, FileSearchState, FileSortMode, FirewallPanel, NavView, SidebarFocus, UpdateEvent};
use crate::app::InvestigationReport;
use crate::services::geoip_service::{GeoInfo, GeoIpService};
use crate::utils::icon_extractor::IconCache;
use crate::i18n::Translator;
use tokio::sync::mpsc;

pub struct InstallState {
    pub show_dialog: bool,
    pub message: String,
    pub installing: bool,
    pub done: bool,
    pub success: bool,
    pub log: String,
    pub child: Option<tokio::sync::oneshot::Receiver<std::process::Output>>,
    pub show_password_modal: bool,
    pub password: String,
    pub needs_password: bool,
}

impl InstallState {
    pub fn new() -> Self {
        Self {
            show_dialog: false,
            message: String::new(),
            installing: false,
            done: false,
            success: false,
            log: String::new(),
            child: None,
            show_password_modal: false,
            password: String::new(),
            needs_password: false,
        }
    }
}

pub struct NerdFontState {
    pub show_dialog: bool,
    pub dialog_dismissed: bool,
    pub installing: bool,
    pub install_done: bool,
    pub install_success: bool,
    pub install_message: String,
    pub install_rx: Option<tokio::sync::oneshot::Receiver<String>>,
}

impl NerdFontState {
    pub fn new() -> Self {
        Self {
            show_dialog: false,
            dialog_dismissed: false,
            installing: false,
            install_done: false,
            install_success: false,
            install_message: String::new(),
            install_rx: None,
        }
    }
}

pub struct UiState {
    pub should_quit: bool,
    pub current_state: AppState,
    pub sidebar_focus: SidebarFocus,
    pub frame_count: u64,
    pub needs_clear: bool,
    pub search_query: String,
    pub search_mode: bool,
    pub filter_high_risk_only: bool,
    pub status_message: String,
    pub show_confirmation: bool,
    pub confirmation_message: String,
    pub auto_analysis_complete: bool,
    pub is_initial_loading: bool,
    pub analysis_paused: bool,
    pub continuous_refresh_counter: u64,
    pub center_tab: usize,
    pub current_nav_view: NavView,
    pub nav_sidebar_expanded: bool,
    pub selected_action_index: usize,
    pub show_map: bool,
    pub show_language_modal: bool,
    pub language_selection_index: usize,
    pub language_scroll_offset: usize,
    pub show_welcome_dialog: bool,
    pub welcome_index: usize,
    pub show_update_dialog: bool,
    pub show_file_search_modal: bool,
    pub file_search_state: FileSearchState,
    pub translator: Translator,
    pub hunter_mode: bool,
}

pub struct NetworkDataState {
    pub network_connections: Vec<NetworkConnection>,
    pub processes: Vec<ProcessInfo>,
    pub app_connections: Vec<AppConnection>,
    pub selected_app_index: usize,
    pub selected_connection_index: usize,
    pub icon_cache: IconCache,
    pub data_rx: Option<std::sync::mpsc::Receiver<(Vec<NetworkConnection>, Vec<ProcessInfo>)>>,
    pub grouping_rx: Option<std::sync::mpsc::Receiver<Vec<AppConnection>>>,
    pub icon_extraction_rx: Option<std::sync::mpsc::Receiver<(String, String)>>,
    pub cached_filtered_indices: Vec<usize>,
}

pub struct GeoState {
    pub geoip: GeoIpService,
    pub geo_tx: mpsc::UnboundedSender<(u32, String, GeoInfo)>,
    pub geo_rx: mpsc::UnboundedReceiver<(u32, String, GeoInfo)>,
    pub pending_geo_lookups: usize,
    pub user_geo: Option<GeoInfo>,
    pub user_info_rx: mpsc::UnboundedReceiver<GeoInfo>,
    pub user_info_tx: mpsc::UnboundedSender<GeoInfo>,
}

pub struct InvestigationState {
    pub investigation_report: Option<InvestigationReport>,
    pub is_investigating: bool,
    pub inv_tx: mpsc::UnboundedSender<InvestigationReport>,
    pub inv_rx: mpsc::UnboundedReceiver<InvestigationReport>,
}

pub struct FirewallState {
    pub firewall_mode: bool,
    pub firewall_focus: FirewallPanel,
    pub firewall_connections: Vec<NetworkConnection>,
    pub firewall_process_name: String,
    pub blocked_ips: Vec<(String, String, String)>,
    pub firewall_conn_index: usize,
    pub firewall_blocked_index: usize,
    pub firewall_action_index: usize,
    pub firewall_conn_checked: Vec<bool>,
    pub firewall_blocked_checked: Vec<bool>,
}

pub struct UpdateState {
    pub show_update_dialog: bool,
    pub latest_remote_version: String,
    pub update_rx: Option<std::sync::mpsc::Receiver<String>>,
    pub update_task_rx: Option<mpsc::UnboundedReceiver<UpdateEvent>>,
    pub is_updating: bool,
    pub update_done: bool,
    pub update_success: bool,
    pub update_message: String,
    pub update_progress: f64,
}

pub struct StorageState {
    pub disks: Vec<DiskInfo>,
    pub selected_disk_index: usize,
    pub disks_loading: bool,
    pub current_directory: std::path::PathBuf,
    pub file_entries: Vec<FileEntry>,
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
    pub file_sort_mode: FileSortMode,
    pub last_storage_refresh: std::time::Instant,
    pub search_progress_running: bool,
    pub search_progress_found: usize,
    pub search_progress_rx: Option<std::sync::mpsc::Receiver<Vec<FileEntry>>>,
    pub search_progress_count: Option<std::sync::Arc<std::sync::atomic::AtomicUsize>>,
    pub search_progress_abort: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}

pub struct ContainerState {
    pub containers: Vec<ContainerInfo>,
    pub selected_container_index: usize,
    pub selected_container_action_index: usize,
    pub container_detail_scroll: usize,
    pub containers_loading: bool,
    pub containers_loaded_once: bool,
    pub containers_error: Option<String>,
    pub container_rx: Option<std::sync::mpsc::Receiver<Result<Vec<ContainerInfo>, String>>>,
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
    pub docker_hub_search: DockerHubSearchState,
    pub docker_hub_search_rx: Option<std::sync::mpsc::Receiver<Result<Vec<crate::app::containers::DockerHubImage>, String>>>,
    pub docker_hub_create_rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    pub pending_container_action: Option<ContainerAction>,
    pub pending_docker_action: Option<DockerAction>,
}

pub struct LibraryState {
    pub libraries: Vec<LibraryInfo>,
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
    pub libraries_rx: Option<std::sync::mpsc::Receiver<Vec<LibraryInfo>>>,
}

pub struct TrendState {
    pub cpu_history: Vec<f64>,
    pub conn_count_history: Vec<u64>,
}
