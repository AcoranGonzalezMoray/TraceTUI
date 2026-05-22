use crate::app::grouping::ConnectionGrouper;
use crate::app::nerdfont;
use crate::app::risk::RiskAnalyzer;
use crate::app::App;
use crate::config;
use crate::resources;
use crate::tr;
use semver;
use std::collections::HashMap;

fn is_newer(local: &str, remote: &str) -> bool {
    let local_v = semver::Version::parse(local.trim_start_matches(['v', 'V']));
    let remote_v = semver::Version::parse(remote.trim_start_matches(['v', 'V']));
    match (local_v, remote_v) {
        (Ok(l), Ok(r)) => r > l,
        _ => false,
    }
}
impl App {
    pub fn on_tick(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
        if self.current_nav_view == crate::app::NavView::Main {
            if let Some(a) = self.get_selected_app() {
                self.cpu_history.push(a.cpu_usage as f64);
                if self.cpu_history.len() > config::CPU_HISTORY_MAX {
                    self.cpu_history.remove(0);
                }
            }
            let total = self
                .app_connections
                .iter()
                .map(|a| a.connections.len() as u64)
                .sum();
            self.conn_count_history.push(total);
            if self.conn_count_history.len() > config::CONN_HISTORY_MAX {
                self.conn_count_history.remove(0);
            }
        }
        self.check_analysis_complete();
        self.process_deferred_icon_extraction();
        if self.is_installing {
            self.check_install_complete();
        }
        if self.nerdfont_installing && !self.nerdfont_install_done {
            self.check_nerdfont_install_complete();
        }
        self.process_geo_results();
        self.process_investigation_results();
        self.process_user_location();
        self.process_update_result();
        self.process_update_task();
        self.process_search_results();
        self.process_container_results();
        if self.current_nav_view == crate::app::NavView::Containers
            && !self.containers_loaded_once
            && !self.containers_loading
        {
            self.refresh_containers_async();
        }
        if self.auto_analysis_complete
            && !self.analysis_paused
            && (self.current_nav_view == crate::app::NavView::Main
                || self.current_nav_view == crate::app::NavView::TrendGraphs)
        {
            self.continuous_refresh_counter = self.continuous_refresh_counter.wrapping_add(1);
            if self.continuous_refresh_counter >= config::REFRESH_COUNTER_THRESHOLD {
                self.continuous_refresh_counter = 0;
                self.trigger_background_refresh();
            }
        }
    }
    pub fn perform_auto_analysis(&mut self) {
        if !self.auto_analysis_complete {
            self.status_message = tr!(self.translator, "status.auto_analyzing").to_string();
            let geo = self.geoip.clone();
            let tx = self.user_info_tx.clone();
            tokio::spawn(async move {
                if let Ok(Some(info)) = geo.lookup("").await {
                    let _ = tx.send(info);
                }
            });
            let (tx_data, rx_data) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let mut analyzer = crate::app::network::NetworkAnalyzer::new();
                let conns = match analyzer.refresh_connections() {
                    Ok(_) => analyzer.get_connections().to_vec(),
                    Err(_) => Vec::new(),
                };
                let mut manager = crate::app::process::ProcessManager::new();
                let procs = match manager.refresh_processes() {
                    Ok(_) => manager.get_all_processes().to_vec(),
                    Err(_) => Vec::new(),
                };
                let _ = tx_data.send((conns, procs));
            });
            self.data_rx = Some(rx_data);
        }
    }
    pub fn trigger_background_refresh(&mut self) {
        if self.data_rx.is_some() {
            return;
        }
        let (tx_data, rx_data) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut analyzer = crate::app::network::NetworkAnalyzer::new();
            let conns = match analyzer.refresh_connections() {
                Ok(_) => analyzer.get_connections().to_vec(),
                Err(_) => Vec::new(),
            };
            let mut manager = crate::app::process::ProcessManager::new();
            let procs = match manager.refresh_processes() {
                Ok(_) => manager.get_all_processes().to_vec(),
                Err(_) => Vec::new(),
            };
            let _ = tx_data.send((conns, procs));
        });
        self.data_rx = Some(rx_data);
    }
    fn check_analysis_complete(&mut self) {
        if let Some(ref rx) = self.data_rx {
            if let Ok((conns, procs)) = rx.try_recv() {
                self.network_connections = conns;
                self.processes = procs;
                self.data_rx = None;
                if self.network_connections.is_empty()
                    && cfg!(target_os = "linux")
                    && !crate::app::network::has_netstat()
                {
                    self.show_install_dialog = true;
                    self.install_message = tr!(self.translator, "dialog.net_tools_msg").to_string();
                    self.status_message =
                        tr!(self.translator, "status.install_required").to_string();
                } else {
                    self.status_message = tr!(
                        self.translator,
                        "status.processing",
                        self.network_connections.len(),
                        self.processes.len()
                    );
                }
                let conns_for_thread = self.network_connections.clone();
                let procs_for_thread = self.processes.clone();
                let hunter = self.hunter_mode;
                let (tx_g, rx_g) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let result = ConnectionGrouper::group(
                        &procs_for_thread,
                        &conns_for_thread,
                        hunter,
                        |_, _| String::new(),
                    );
                    let _ = tx_g.send(result);
                });
                self.grouping_rx = Some(rx_g);
            }
        }
        if let Some(ref rx) = self.grouping_rx {
            if let Ok(mut app_conns) = rx.try_recv() {
                let geo_cache: HashMap<(u32, String), (Option<String>, Option<String>)> = self
                    .app_connections
                    .iter()
                    .flat_map(|a| {
                        a.connections.iter().filter_map(|c| {
                            c.location.as_ref().map(|_| {
                                (
                                    (a.pid, c.foreign_address.clone()),
                                    (c.location.clone(), c.isp.clone()),
                                )
                            })
                        })
                    })
                    .collect();
                for app in &mut app_conns {
                    for conn in &mut app.connections {
                        if let Some((loc, isp)) =
                            geo_cache.get(&(app.pid, conn.foreign_address.clone()))
                        {
                            conn.location = loc.clone();
                            conn.isp = isp.clone();
                        }
                    }
                }
                self.app_connections = app_conns;
                self.grouping_rx = None;
                self.trigger_geo_lookup_for_selected_app();
                if !self.nerdfont_dialog_dismissed && !nerdfont::has_nerdfont() {
                    self.show_nerdfont_dialog = true;
                    self.status_message =
                        tr!(self.translator, "status.nerdfont_missing").to_string();
                }
                self.is_initial_loading = false;
                let is_background = self.auto_analysis_complete;
                self.auto_analysis_complete = true;
                if is_background {
                    for app_conn in &mut self.app_connections {
                        if app_conn.icon.is_empty() {
                            let icon = self
                                .icon_cache
                                .get_icon(&app_conn.process_path, &app_conn.process_name);
                            if !icon.is_empty() {
                                app_conn.icon = icon;
                            }
                        }
                    }
                } else {
                    let app_conns = self.app_connections.clone();
                    let (tx_icon, rx_icon) = std::sync::mpsc::channel::<(String, String)>();
                    std::thread::spawn(move || {
                        let mut icon_cache = crate::utils::icon_extractor::IconCache::new();
                        for a in &app_conns {
                            let icon = icon_cache.get_icon(&a.process_path, &a.process_name);
                            let _ = tx_icon.send((a.process_path.clone(), icon));
                        }
                    });
                    self.icon_extraction_rx = Some(rx_icon);
                }
            }
        }
    }
    fn process_search_results(&mut self) {
        if !self.search_progress_running {
            return;
        }
        if let Some(ref count) = self.search_progress_count {
            self.search_progress_found = count.load(std::sync::atomic::Ordering::Relaxed);
        }
        if let Some(ref rx) = self.search_progress_rx {
            if let Ok(entries) = rx.try_recv() {
                self.file_entries = entries;
                self.file_scroll = 0;
                self.search_progress_running = false;
                self.search_progress_found = 0;
                self.search_progress_rx = None;
                self.search_progress_count = None;
                self.search_progress_abort = None;
            }
        }
    }
    fn process_deferred_icon_extraction(&mut self) {
        if let Some(ref rx) = self.icon_extraction_rx {
            while let Ok((exe_path, icon)) = rx.try_recv() {
                self.icon_cache.insert_icon(&exe_path, icon.clone());
                if let Some(app_conn) = self
                    .app_connections
                    .iter_mut()
                    .find(|a| a.process_path == exe_path)
                {
                    app_conn.icon = icon;
                }
            }
        }
    }
    pub fn start_batch_analysis(&mut self) {
        self.refresh_network_connections();
        self.refresh_processes();
        self.group_connections_inline();
        let high_risk = self
            .app_connections
            .iter()
            .filter(|a| RiskAnalyzer::is_high_or_critical(&a.risk_level))
            .count();
        self.is_initial_loading = false;
        self.status_message = tr!(
            self.translator,
            "status.analysis_complete",
            self.app_connections.len(),
            high_risk
        );
    }
    fn refresh_network_connections(&mut self) {
        let mut analyzer = crate::app::network::NetworkAnalyzer::new();
        match analyzer.refresh_connections() {
            Ok(_) => {
                self.network_connections = analyzer.get_connections().to_vec();
                if self.network_connections.is_empty()
                    && cfg!(target_os = "linux")
                    && !crate::app::network::has_netstat()
                {
                    self.show_install_dialog = true;
                    self.install_message = tr!(self.translator, "dialog.net_tools_msg").to_string();
                    self.status_message =
                        tr!(self.translator, "status.install_required").to_string();
                } else {
                    self.status_message = tr!(
                        self.translator,
                        "status.found_conns",
                        self.network_connections.len()
                    );
                }
            }
            Err(e) => {
                let msg = format!("{}", e);
                if cfg!(target_os = "linux")
                    && !crate::app::network::has_netstat()
                    && !crate::app::network::has_ss()
                {
                    self.show_install_dialog = true;
                    self.install_message =
                        tr!(self.translator, "dialog.net_tools_missing").to_string();
                    self.status_message = tr!(self.translator, "status.tools_missing").to_string();
                } else {
                    self.status_message = tr!(self.translator, "status.conn_fail", msg);
                }
            }
        }
    }
    fn refresh_processes(&mut self) {
        let mut manager = crate::app::process::ProcessManager::new();
        if manager.refresh_processes().is_ok() {
            self.processes = manager.get_all_processes().to_vec();
            self.status_message = tr!(self.translator, "status.found_procs", self.processes.len());
        } else {
            self.status_message = tr!(self.translator, "status.proc_fail").to_string();
        }
    }
    fn group_connections_inline(&mut self) {
        let hunter = self.hunter_mode;
        let icon_cache = &mut self.icon_cache;
        let is_initial_loading = self.is_initial_loading;
        let result = ConnectionGrouper::group(
            &self.processes,
            &self.network_connections,
            hunter,
            |path, name| {
                if is_initial_loading {
                    String::new()
                } else {
                    icon_cache.get_icon(path, name)
                }
            },
        );
        self.app_connections = result;
    }
    pub fn trigger_geo_lookup_for_selected_app(&mut self) {
        if let Some(app_conn) = self.get_selected_app().cloned() {
            let ips: Vec<String> = app_conn
                .connections
                .iter()
                .filter(|c| c.location.is_none())
                .map(|c| c.foreign_address.clone())
                .collect();

            if ips.is_empty() {
                return;
            }

            let pid = app_conn.pid;
            let geo_service = self.geoip.clone();
            let tx = self.geo_tx.clone();
            self.pending_geo_lookups = self.pending_geo_lookups.saturating_add(ips.len());

            tokio::spawn(async move {
                let results = geo_service.lookup_batch(&ips).await;
                for (ip, info) in results {
                    let _ = tx.send((pid, ip, info));
                }
            });
        }
    }
    pub fn start_investigation(&mut self) {
        if let Some(app_conn) = self.get_selected_app() {
            if let Some(conn) = app_conn.connections.get(self.selected_connection_index) {
                let ip = conn.foreign_address.clone();
                let port = conn.foreign_port;
                let process_name = app_conn.process_name.clone();
                self.is_investigating = true;
                self.analysis_paused = true;
                self.investigation_report = None;
                self.selected_action_index = 0;
                self.status_message = tr!(self.translator, "status.investigating", ip, port);
                let tx = self.inv_tx.clone();
                let geo_service = self.geoip.clone();
                tokio::spawn(async move {
                    let service = crate::app::investigation_service::InvestigationService::new(
                        geo_service,
                        process_name,
                    );
                    service.investigate(ip, port, tx).await;
                });
            }
        }
    }
    fn process_user_location(&mut self) {
        if let Ok(info) = self.user_info_rx.try_recv() {
            self.user_geo = Some(info);
        }
    }
    pub fn check_for_updates(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        self.update_rx = Some(rx);
        let local = env!("CARGO_PKG_VERSION").to_string();
        self.status_message = tr!(self.translator, "status.checking_updates").to_string();

        tokio::spawn(async move {
            let url = &resources::URLS.github_api_releases;
            let client = reqwest::Client::builder()
                .user_agent(&resources::URLS.user_agent)
                .build()
                .ok();

            let result = match client {
                Some(c) => match c.get(url).send().await {
                    Ok(r) if r.status().is_success() => match r.json::<serde_json::Value>().await {
                        Ok(v) => {
                            let version = v["tag_name"]
                                .as_str()
                                .map(|s| s.trim_start_matches(['v', 'V']).to_string())
                                .unwrap_or_default();
                            if version.is_empty() {
                                Err("No release tags found in GitHub".to_string())
                            } else {
                                Ok(version)
                            }
                        }
                        Err(e) => Err(format!("API error: {}", e)),
                    },
                    Ok(r) if r.status() == 404 => {
                        Err("No GitHub releases published yet".to_string())
                    }
                    Ok(r) => Err(format!("GitHub API: HTTP {}", r.status())),
                    Err(e) => Err(format!("Connection error: {}", e)),
                },
                None => Err("Failed to start update client".to_string()),
            };

            match result {
                Ok(version) => {
                    if is_newer(&local, &version) {
                        let _ = tx.send(version);
                    }
                }
                Err(e) => {
                    let _ = tx.send(format!("ERROR:{}", e));
                }
            }
        });
    }
    fn process_update_result(&mut self) {
        if let Some(rx) = &self.update_rx {
            while let Ok(version) = rx.try_recv() {
                if version.starts_with("ERROR:") {
                    let err_msg = version.trim_start_matches("ERROR:").to_string();
                    self.status_message = format!("[-] Update Check: {}", err_msg);
                } else if !version.is_empty() {
                    self.latest_remote_version = version;
                    self.show_update_dialog = true;
                    self.status_message =
                        tr!(self.translator, "status.update_available").to_string();
                }
            }
        }
    }
    pub fn start_self_update(&mut self) {
        if self.latest_remote_version.is_empty() || self.is_updating {
            return;
        }
        self.is_updating = true;
        self.update_done = false;
        self.update_success = false;
        self.update_progress = 0.0;
        self.status_message = tr!(self.translator, "status.updating").to_string();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<crate::app::types::UpdateEvent>();
        self.update_task_rx = Some(rx);
        crate::app::installation::spawn_self_update(tx, self.latest_remote_version.clone());
    }
    fn process_update_task(&mut self) {
        if let Some(rx) = &mut self.update_task_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    crate::app::types::UpdateEvent::Progress(p) => {
                        self.update_progress = p;
                    }
                    crate::app::types::UpdateEvent::Finished(success, msg) => {
                        self.is_updating = false;
                        self.update_done = true;
                        self.update_success = success;
                        if success {
                            self.update_message =
                                tr!(self.translator, "dialog.update_success_msg").to_string();
                            self.status_message =
                                tr!(self.translator, "status.update_done").to_string();
                        } else {
                            self.update_message = format!(
                                "{}: {}",
                                tr!(self.translator, "dialog.update_failed_msg"),
                                msg
                            );
                            self.status_message = tr!(self.translator, "status.update_fail", msg);
                        }
                    }
                }
            }
        }
    }
    fn process_geo_results(&mut self) {
        while let Ok((pid, ip, info)) = self.geo_rx.try_recv() {
            self.pending_geo_lookups = self.pending_geo_lookups.saturating_sub(1);
            for app_conn in &mut self.app_connections {
                if app_conn.pid == pid {
                    for conn in &mut app_conn.connections {
                        if conn.foreign_address == ip {
                            let flag = crate::services::geoip_service::GeoIpService::get_flag_emoji(
                                &info.countryCode,
                            );
                            conn.location =
                                Some(format!("{} {}, {}", flag, info.city, info.country));
                            conn.isp = Some(info.isp.clone());
                        }
                    }
                }
            }
        }
    }
    fn process_investigation_results(&mut self) {
        while let Ok(report) = self.inv_rx.try_recv() {
            let _ = self.database.save_investigation(
                &report.ip,
                report.domain.as_deref().unwrap_or("?"),
                report.risk_score,
                report.country.as_deref().unwrap_or("?"),
                report.isp.as_deref().unwrap_or("?"),
                report.whois_data.as_deref().unwrap_or(""),
            );
            self.investigation_report = Some(report);
            self.is_investigating = false;
            self.status_message = tr!(self.translator, "status.intel_ready").to_string();
        }
    }
    fn check_install_complete(&mut self) {
        if let Some(rx) = &mut self.install_child {
            if let Ok(output) = rx.try_recv() {
                self.install_child = None;
                if output.status.success() {
                    self.is_installing = false;
                    self.install_done = true;
                    self.install_success = true;
                    self.install_log = String::from_utf8_lossy(&output.stdout).to_string();
                    self.install_message =
                        tr!(self.translator, "dialog.net_tools_success").to_string();
                    self.status_message =
                        tr!(self.translator, "status.install_installed").to_string();
                    self.start_batch_analysis();
                } else if !self.install_needs_password {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.to_lowercase().contains("password") {
                        self.install_needs_password = true;
                        self.show_password_modal = true;
                        self.install_password.clear();
                        self.install_message =
                            tr!(self.translator, "dialog.password_required").to_string();
                        self.status_message =
                            tr!(self.translator, "status.install_password").to_string();
                    } else {
                        self.is_installing = false;
                        self.install_done = true;
                        self.install_success = false;
                        self.install_log = stderr.to_string();
                        let trimmed = stderr.trim().to_string();
                        self.install_message =
                            tr!(self.translator, "dialog.net_tools_fail_msg", trimmed);
                        self.status_message =
                            tr!(self.translator, "status.install_failed").to_string();
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    self.install_log = format!("{}\n{}", stdout, stderr);
                    if stderr.to_lowercase().contains("password")
                        || stderr.to_lowercase().contains("incorrect")
                        || stderr.to_lowercase().contains("try again")
                    {
                        self.show_password_modal = true;
                        self.install_password.clear();
                        self.install_message =
                            tr!(self.translator, "dialog.password_wrong").to_string();
                        self.status_message =
                            tr!(self.translator, "status.install_wrong_pw").to_string();
                    } else {
                        self.is_installing = false;
                        self.install_done = true;
                        self.install_success = false;
                        let trimmed = stderr.trim().to_string();
                        self.install_message =
                            tr!(self.translator, "dialog.net_tools_fail_msg", trimmed);
                        let last_line = stderr
                            .trim()
                            .lines()
                            .last()
                            .unwrap_or("unknown error")
                            .to_string();
                        self.status_message =
                            tr!(self.translator, "status.install_fail_detail", last_line);
                    }
                }
            }
        }
    }
    fn check_nerdfont_install_complete(&mut self) {
        if let Some(rx) = &mut self.nerdfont_install_rx {
            if let Ok(msg) = rx.try_recv() {
                self.nerdfont_install_rx = None;
                self.nerdfont_install_done = true;
                self.nerdfont_install_success = msg.starts_with("Installed");
                self.nerdfont_install_message = msg.clone();
                self.status_message = if self.nerdfont_install_success {
                    tr!(self.translator, "status.nerdfont_installed").to_string()
                } else {
                    let first_line = msg.lines().next().unwrap_or("failed").to_string();
                    tr!(self.translator, "status.nerdfont_fail", first_line)
                };
            }
        }
    }
}
