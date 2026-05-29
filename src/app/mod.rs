pub mod containers;
pub mod firewall_service;
pub mod grouping;
pub mod installation;
pub mod investigation_service;
pub mod io;
pub mod libraries;
pub mod nerdfont;
pub mod network;
pub mod process;
pub mod risk;
pub mod services;
pub mod states;
use self::states::{InstallState, NerdFontState};
pub mod storage;
pub mod types;
pub mod ui;
use crate::config;
use crate::i18n::{self, Translator};
use crate::tr;
use crate::utils::db::Database;
pub use investigation_service::InvestigationReport;
pub use io::{restore_terminal, setup_terminal};
pub use states::{
    ContainerState, FirewallState, GeoState, InvestigationState, LibraryState, NetworkDataState,
    StorageState, TrendState, UiState, UpdateState,
};
pub use types::{AppConnection, AppState, FirewallPanel, NavView, SidebarFocus};

pub struct App {
    pub ui: UiState,
    pub network: NetworkDataState,
    pub geo: GeoState,
    pub investigation: InvestigationState,
    pub firewall: FirewallState,
    pub update: UpdateState,
    pub storage: StorageState,
    pub containers: ContainerState,
    pub libraries: LibraryState,
    pub trend: TrendState,
    pub install: InstallState,
    pub nerdfont: NerdFontState,
    pub database: Database,
}

impl App {
    pub fn new() -> Self {
        let detected_locale = config::load_language().unwrap_or_else(i18n::detect_system_locale);
        let translator = Translator::new(&detected_locale);

        #[allow(unused_mut)]
        let mut app = Self {
            ui: UiState::new(translator),
            network: NetworkDataState::new(),
            geo: GeoState::new(),
            investigation: InvestigationState::new(),
            firewall: FirewallState::new(),
            update: UpdateState::new(),
            storage: StorageState::new(),
            containers: ContainerState::new(),
            libraries: LibraryState::new(),
            trend: TrendState::new(),
            install: InstallState::new(),
            nerdfont: NerdFontState::new(),
            database: Database::new().expect("Failed to init database"),
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
                app.ui.show_welcome_dialog = true;
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
        self.network
            .app_connections
            .iter()
            .filter(|app| {
                let matches_search = if self.ui.search_query.is_empty() {
                    true
                } else {
                    let q = self.ui.search_query.to_lowercase();
                    let name_match = app.process_name.to_lowercase().contains(&q);
                    let ip_match = app.connections.iter().any(|conn| {
                        conn.foreign_address.to_lowercase().contains(&q)
                            || conn.local_address.to_lowercase().contains(&q)
                    });
                    name_match || ip_match
                };
                let matches_risk = if self.ui.filter_high_risk_only {
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
        filtered.get(self.network.selected_app_index).copied()
    }
    pub fn get_selected_container(&self) -> Option<&crate::app::containers::ContainerInfo> {
        self.containers
            .containers
            .get(self.containers.selected_container_index)
    }
    pub fn get_selected_disk(&self) -> Option<&crate::app::storage::DiskInfo> {
        self.storage.disks.get(self.storage.selected_disk_index)
    }
    pub fn compute_filtered_indices(&mut self) {
        let ext_idx = self.storage.file_search_extension_idx.min(
            crate::app::storage::FILE_EXTENSION_FILTERS
                .len()
                .saturating_sub(1),
        );
        let query = self.storage.file_search_query.to_lowercase();
        let (_, exts) = crate::app::storage::FILE_EXTENSION_FILTERS[ext_idx];
        self.network.cached_filtered_indices = if self.storage.file_search_mode {
            self.storage
                .file_entries
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
            (0..self.storage.file_entries.len()).collect()
        };
    }
    pub fn abort_search(&mut self) {
        if let Some(ref abort) = self.storage.search_progress_abort {
            abort.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        if self.storage.search_progress_running {
            self.storage.search_progress_running = false;
            self.storage.search_progress_found = 0;
        }
        self.storage.search_progress_rx = None;
        self.storage.search_progress_count = None;
        self.storage.search_progress_abort = None;
    }
    pub fn refresh_libraries(&mut self) {
        if self.libraries.libraries_loading {
            return;
        }

        if self.network.processes.is_empty() && self.network.app_connections.is_empty() {
            self.libraries.libraries_loading = true;
            self.ui.status_message =
                tr!(self.ui.translator, "libraries.status.refreshing").to_string();
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let processes = self.network.processes.clone();
        let app_conns = self.network.app_connections.clone();
        self.libraries.libraries.clear();
        std::thread::spawn(move || {
            crate::app::libraries::inspect_libraries_batched(&processes, &app_conns, tx);
        });
        self.libraries.libraries_rx = Some(rx);
        self.libraries.libraries_loading = true;
        self.ui.status_message = tr!(self.ui.translator, "libraries.status.refreshing").to_string();
    }
}

impl App {
    pub fn trigger_geo_lookup_for_selected_app(&mut self) {
        crate::app::services::analysis_service::trigger_geo_lookup_for_selected_app(self);
    }
    pub fn start_batch_analysis(&mut self) {
        crate::app::services::analysis_service::start_batch_analysis(self);
    }
    pub fn start_investigation(&mut self) {
        crate::app::services::analysis_service::start_investigation(self);
    }
    pub fn start_self_update(&mut self) {
        crate::app::services::analysis_service::start_self_update(self);
    }
}
