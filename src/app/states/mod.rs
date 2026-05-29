#![allow(dead_code)]

use crate::app::containers::{ContainerAction, ContainerInfo, DockerAction, DockerHubSearchState};
use crate::app::libraries::LibraryInfo;
use crate::app::network::NetworkConnection;
use crate::app::process::ProcessInfo;
use crate::app::storage::{DiskInfo, FileEntry};
use crate::app::types::{
    AppConnection, AppState, ConfirmationAction, FileSearchState, FileSortMode, FirewallPanel,
    NavView, SidebarFocus, UpdateEvent,
};
use crate::app::InvestigationReport;
use crate::i18n::Translator;
use crate::services::geoip_service::{GeoInfo, GeoIpService};
use crate::utils::icon_extractor::IconCache;
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
    pub show_file_search_modal: bool,
    pub file_search_state: FileSearchState,
    pub translator: Translator,
    pub hunter_mode: bool,
    pub action_in_progress: Option<String>,
    pub status_message_tx: Option<std::sync::mpsc::Sender<String>>,
    pub status_message_rx: Option<std::sync::mpsc::Receiver<String>>,
    pub pending_confirmation_action: Option<ConfirmationAction>,
}

impl UiState {
    pub fn new(translator: Translator) -> Self {
        Self {
            should_quit: false,
            current_state: AppState::Dashboard,
            sidebar_focus: SidebarFocus::Left,
            frame_count: 0,
            needs_clear: false,
            search_query: String::new(),
            search_mode: false,
            filter_high_risk_only: false,
            status_message: String::new(),
            show_confirmation: false,
            confirmation_message: String::new(),
            auto_analysis_complete: false,
            is_initial_loading: true,
            analysis_paused: false,
            continuous_refresh_counter: 0,
            center_tab: 0,
            current_nav_view: NavView::Main,
            nav_sidebar_expanded: false,
            selected_action_index: 0,
            show_map: false,
            show_language_modal: false,
            language_selection_index: 0,
            language_scroll_offset: 0,
            show_welcome_dialog: false,
            welcome_index: 0,
            show_file_search_modal: false,
            file_search_state: FileSearchState::default(),
            translator,
            hunter_mode: false,
            action_in_progress: None,
            status_message_tx: None,
            status_message_rx: None,
            pending_confirmation_action: None,
        }
    }
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

impl NetworkDataState {
    pub fn new() -> Self {
        Self {
            network_connections: Vec::new(),
            processes: Vec::new(),
            app_connections: Vec::new(),
            selected_app_index: 0,
            selected_connection_index: 0,
            icon_cache: IconCache::new(),
            data_rx: None,
            grouping_rx: None,
            icon_extraction_rx: None,
            cached_filtered_indices: Vec::new(),
        }
    }
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

impl GeoState {
    pub fn new() -> Self {
        let (geo_tx, geo_rx) = mpsc::unbounded_channel();
        let (user_info_tx, user_info_rx) = mpsc::unbounded_channel();
        Self {
            geoip: GeoIpService::new().expect("Failed to initialize GeoIpService"),
            geo_tx,
            geo_rx,
            pending_geo_lookups: 0,
            user_geo: None,
            user_info_rx,
            user_info_tx,
        }
    }
}

pub struct InvestigationState {
    pub investigation_report: Option<InvestigationReport>,
    pub is_investigating: bool,
    pub inv_tx: mpsc::UnboundedSender<InvestigationReport>,
    pub inv_rx: mpsc::UnboundedReceiver<InvestigationReport>,
}

impl InvestigationState {
    pub fn new() -> Self {
        let (inv_tx, inv_rx) = mpsc::unbounded_channel();
        Self {
            investigation_report: None,
            is_investigating: false,
            inv_tx,
            inv_rx,
        }
    }
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

impl FirewallState {
    pub fn new() -> Self {
        Self {
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
        }
    }
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

impl UpdateState {
    pub fn new() -> Self {
        Self {
            show_update_dialog: false,
            latest_remote_version: String::new(),
            update_rx: None,
            update_task_rx: None,
            is_updating: false,
            update_done: false,
            update_success: false,
            update_message: String::new(),
            update_progress: 0.0,
        }
    }
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

impl StorageState {
    pub fn new() -> Self {
        Self {
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
            file_sort_mode: FileSortMode::ByName,
            last_storage_refresh: std::time::Instant::now(),
            search_progress_running: false,
            search_progress_found: 0,
            search_progress_rx: None,
            search_progress_count: None,
            search_progress_abort: None,
        }
    }
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
    pub docker_hub_search_rx: Option<
        std::sync::mpsc::Receiver<Result<Vec<crate::app::containers::DockerHubImage>, String>>,
    >,
    pub docker_hub_create_rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    pub pending_container_action: Option<ContainerAction>,
    pub pending_docker_action: Option<DockerAction>,
    pub docker_action_in_progress: Option<DockerAction>,
    pub container_action_in_progress: Option<ContainerAction>,
    pub container_action_rx: Option<std::sync::mpsc::Receiver<(String, Result<(), String>)>>,
}

impl ContainerState {
    pub fn new() -> Self {
        Self {
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
            docker_hub_search: DockerHubSearchState {
                search_query: String::new(),
                results: Vec::new(),
                selected_result_index: 0,
                container_name: String::new(),
                ports: String::new(),
                env_vars: String::new(),
                focused_field: 0,
            },
            docker_hub_search_rx: None,
            docker_hub_create_rx: None,
            pending_container_action: None,
            pending_docker_action: None,
            docker_action_in_progress: None,
            container_action_in_progress: None,
            container_action_rx: None,
        }
    }
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

impl LibraryState {
    pub fn new() -> Self {
        Self {
            libraries: Vec::new(),
            libraries_loading: false,
            selected_library_process_index: 0,
            selected_library_index: 0,
            library_process_scroll: 0,
            library_lib_scroll: 0,
            libraries_loaded_once: false,
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
            libraries_rx: None,
        }
    }
}

pub struct TrendState {
    pub cpu_history: Vec<f64>,
    pub conn_count_history: Vec<u64>,
}

impl TrendState {
    pub fn new() -> Self {
        Self {
            cpu_history: Vec::new(),
            conn_count_history: Vec::new(),
        }
    }
}
