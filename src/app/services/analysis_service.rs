use crate::app::grouping::ConnectionGrouper;
use crate::app::nerdfont;
use crate::app::risk::RiskAnalyzer;
use crate::app::App;
use crate::config;
use crate::resources;
use crate::tr;
use semver;
use std::collections::HashMap;

pub fn is_newer(local: &str, remote: &str) -> bool {
    let local_v = semver::Version::parse(local.trim_start_matches(['v', 'V']));
    let remote_v = semver::Version::parse(remote.trim_start_matches(['v', 'V']));
    match (local_v, remote_v) {
        (Ok(l), Ok(r)) => r > l,
        _ => false,
    }
}
pub fn on_tick(app: &mut App) {
    app.ui.frame_count = app.ui.frame_count.wrapping_add(1);
    if app.ui.current_nav_view == crate::app::NavView::Main {
        if let Some(a) = app.get_selected_app() {
            app.trend.cpu_history.push(a.cpu_usage as f64);
            if app.trend.cpu_history.len() > config::CPU_HISTORY_MAX {
                app.trend.cpu_history.remove(0);
            }
        }
        let total = app
            .network
            .app_connections
            .iter()
            .map(|a| a.connections.len() as u64)
            .sum();
        app.trend.conn_count_history.push(total);
        if app.trend.conn_count_history.len() > config::CONN_HISTORY_MAX {
            app.trend.conn_count_history.remove(0);
        }
    }
    check_analysis_complete(app);
    process_deferred_icon_extraction(app);
    if app.install.installing {
        check_install_complete(app);
    }
    if app.nerdfont.installing && !app.nerdfont.install_done {
        check_nerdfont_install_complete(app);
    }
    process_geo_results(app);
    process_investigation_results(app);
    process_user_location(app);
    process_update_result(app);
    process_update_task(app);
    process_search_results(app);
    process_status_messages(app);
    app.process_container_results();
    app.process_container_action_results();
    app.process_libraries_results();
    if app.ui.current_nav_view == crate::app::NavView::Containers
        && !app.containers.containers_loaded_once
        && !app.containers.containers_loading
    {
        app.refresh_containers_async();
    }
    if app.ui.auto_analysis_complete
        && !app.ui.analysis_paused
        && (app.ui.current_nav_view == crate::app::NavView::Main
            || app.ui.current_nav_view == crate::app::NavView::TrendGraphs)
    {
        app.ui.continuous_refresh_counter = app.ui.continuous_refresh_counter.wrapping_add(1);
        if app.ui.continuous_refresh_counter >= config::REFRESH_COUNTER_THRESHOLD {
            app.ui.continuous_refresh_counter = 0;
            trigger_background_refresh(app);
        }
    }
    if app.ui.auto_analysis_complete
        && app.ui.current_nav_view == crate::app::NavView::Storage
        && app.storage.last_storage_refresh.elapsed()
            >= std::time::Duration::from_secs(config::STORAGE_REFRESH_INTERVAL_SECS)
    {
        app.storage.last_storage_refresh = std::time::Instant::now();
        if !app.storage.file_search_mode && !app.storage.search_progress_running {
            app.storage.disks = crate::app::storage::StorageManager::list_disks();
            let current = app.storage.current_directory.clone();
            app.storage.file_entries =
                crate::app::storage::StorageManager::list_directory(&current).unwrap_or_default();

            crate::app::storage::StorageManager::sort_entries(
                &mut app.storage.file_entries,
                app.storage.file_sort_mode,
            );
            app.compute_filtered_indices();
        }
    }
}
pub fn perform_auto_analysis(app: &mut App) {
    if !app.ui.auto_analysis_complete {
        app.ui.status_message = tr!(app.ui.translator, "status.auto_analyzing").to_string();
        let geo = app.geo.geoip.clone();
        let tx = app.geo.user_info_tx.clone();
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
        app.network.data_rx = Some(rx_data);
    }
}
pub fn trigger_background_refresh(app: &mut App) {
    if app.network.data_rx.is_some() {
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
    app.network.data_rx = Some(rx_data);
}
fn check_analysis_complete(app: &mut App) {
    if let Some(ref rx) = app.network.data_rx {
        if let Ok((conns, procs)) = rx.try_recv() {
            app.network.network_connections = conns;
            app.network.processes = procs;
            app.network.data_rx = None;
            if app.network.network_connections.is_empty()
                && cfg!(target_os = "linux")
                && !crate::app::network::has_netstat()
            {
                app.install.show_dialog = true;
                app.install.message = tr!(app.ui.translator, "dialog.net_tools_msg").to_string();
                app.ui.status_message =
                    tr!(app.ui.translator, "status.install_required").to_string();
            } else {
                app.ui.status_message = tr!(
                    app.ui.translator,
                    "status.processing",
                    app.network.network_connections.len(),
                    app.network.processes.len()
                );
            }
            let conns_for_thread = app.network.network_connections.clone();
            let procs_for_thread = app.network.processes.clone();
            let hunter = app.ui.hunter_mode;
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
            app.network.grouping_rx = Some(rx_g);
        }
    }
    if let Some(ref rx) = app.network.grouping_rx {
        if let Ok(mut app_conns) = rx.try_recv() {
            let geo_cache: HashMap<(u32, String), (Option<String>, Option<String>)> = app
                .network
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
            app.network.app_connections = app_conns;
            app.network.grouping_rx = None;
            trigger_geo_lookup_for_selected_app(app);
            if !app.nerdfont.dialog_dismissed && !nerdfont::has_nerdfont() {
                app.nerdfont.show_dialog = true;
                app.ui.status_message =
                    tr!(app.ui.translator, "status.nerdfont_missing").to_string();
            }
            app.ui.is_initial_loading = false;
            let is_background = app.ui.auto_analysis_complete;
            app.ui.auto_analysis_complete = true;
            if app.ui.current_nav_view == crate::app::NavView::LibraryInspection {
                app.refresh_libraries();
            }
            if is_background {
                for app_conn in &mut app.network.app_connections {
                    if app_conn.icon.is_empty() {
                        let icon = app
                            .network
                            .icon_cache
                            .get_icon(&app_conn.process_path, &app_conn.process_name);
                        if !icon.is_empty() {
                            app_conn.icon = icon;
                        }
                    }
                }
            } else {
                let app_conns = app.network.app_connections.clone();
                let (tx_icon, rx_icon) = std::sync::mpsc::channel::<(String, String)>();
                std::thread::spawn(move || {
                    let mut icon_cache = crate::utils::icon_extractor::IconCache::new();
                    for a in &app_conns {
                        let icon = icon_cache.get_icon(&a.process_path, &a.process_name);
                        let _ = tx_icon.send((a.process_path.clone(), icon));
                    }
                });
                app.network.icon_extraction_rx = Some(rx_icon);
            }
        }
    }
}
fn process_search_results(app: &mut App) {
    if !app.storage.search_progress_running {
        return;
    }
    if let Some(ref count) = app.storage.search_progress_count {
        app.storage.search_progress_found = count.load(std::sync::atomic::Ordering::Relaxed);
    }
    if let Some(ref rx) = app.storage.search_progress_rx {
        if let Ok(entries) = rx.try_recv() {
            app.storage.file_entries = entries;
            app.storage.file_scroll = 0;
            app.compute_filtered_indices();
            app.storage.search_progress_running = false;
            app.storage.search_progress_found = 0;
            app.storage.search_progress_rx = None;
            app.storage.search_progress_count = None;
            app.storage.search_progress_abort = None;
        }
    }
}
fn process_deferred_icon_extraction(app: &mut App) {
    if let Some(ref rx) = app.network.icon_extraction_rx {
        while let Ok((exe_path, icon)) = rx.try_recv() {
            app.network.icon_cache.insert_icon(&exe_path, icon.clone());
            if let Some(app_conn) = app
                .network
                .app_connections
                .iter_mut()
                .find(|a| a.process_path == exe_path)
            {
                app_conn.icon = icon;
            }
        }
    }
}
pub fn start_batch_analysis(app: &mut App) {
    refresh_network_connections(app);
    refresh_processes(app);
    group_connections_inline(app);
    let high_risk = app
        .network
        .app_connections
        .iter()
        .filter(|a| RiskAnalyzer::is_high_or_critical(&a.risk_level))
        .count();
    app.ui.is_initial_loading = false;
    app.ui.status_message = tr!(
        app.ui.translator,
        "status.analysis_complete",
        app.network.app_connections.len(),
        high_risk
    );
}
fn refresh_network_connections(app: &mut App) {
    let mut analyzer = crate::app::network::NetworkAnalyzer::new();
    match analyzer.refresh_connections() {
        Ok(_) => {
            app.network.network_connections = analyzer.get_connections().to_vec();
            if app.network.network_connections.is_empty()
                && cfg!(target_os = "linux")
                && !crate::app::network::has_netstat()
            {
                app.install.show_dialog = true;
                app.install.message = tr!(app.ui.translator, "dialog.net_tools_msg").to_string();
                app.ui.status_message =
                    tr!(app.ui.translator, "status.install_required").to_string();
            } else {
                app.ui.status_message = tr!(
                    app.ui.translator,
                    "status.found_conns",
                    app.network.network_connections.len()
                );
            }
        }
        Err(e) => {
            let msg = format!("{}", e);
            if cfg!(target_os = "linux")
                && !crate::app::network::has_netstat()
                && !crate::app::network::has_ss()
            {
                app.install.show_dialog = true;
                app.install.message =
                    tr!(app.ui.translator, "dialog.net_tools_missing").to_string();
                app.ui.status_message = tr!(app.ui.translator, "status.tools_missing").to_string();
            } else {
                app.ui.status_message = tr!(app.ui.translator, "status.conn_fail", msg);
            }
        }
    }
}
fn refresh_processes(app: &mut App) {
    let mut manager = crate::app::process::ProcessManager::new();
    if manager.refresh_processes().is_ok() {
        app.network.processes = manager.get_all_processes().to_vec();
        app.ui.status_message = tr!(
            app.ui.translator,
            "status.found_procs",
            app.network.processes.len()
        );
    } else {
        app.ui.status_message = tr!(app.ui.translator, "status.proc_fail").to_string();
    }
}
fn group_connections_inline(app: &mut App) {
    let hunter = app.ui.hunter_mode;
    let icon_cache = &mut app.network.icon_cache;
    let is_initial_loading = app.ui.is_initial_loading;
    let result = ConnectionGrouper::group(
        &app.network.processes,
        &app.network.network_connections,
        hunter,
        |path, name| {
            if is_initial_loading {
                String::new()
            } else {
                icon_cache.get_icon(path, name)
            }
        },
    );
    app.network.app_connections = result;
}
pub fn trigger_geo_lookup_for_selected_app(app: &mut App) {
    if let Some(app_conn) = app.get_selected_app().cloned() {
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
        let geo_service = app.geo.geoip.clone();
        let tx = app.geo.geo_tx.clone();
        app.geo.pending_geo_lookups = app.geo.pending_geo_lookups.saturating_add(ips.len());

        tokio::spawn(async move {
            let results = geo_service.lookup_batch(&ips).await;
            for (ip, info) in results {
                let _ = tx.send((pid, ip, info));
            }
        });
    }
}
pub fn start_investigation(app: &mut App) {
    if let Some(app_conn) = app.get_selected_app() {
        if let Some(conn) = app_conn
            .connections
            .get(app.network.selected_connection_index)
        {
            let ip = conn.foreign_address.clone();
            let port = conn.foreign_port;
            let process_name = app_conn.process_name.clone();
            app.investigation.is_investigating = true;
            app.ui.analysis_paused = true;
            app.investigation.investigation_report = None;
            app.ui.selected_action_index = 0;
            app.ui.status_message = tr!(app.ui.translator, "status.investigating", ip, port);
            let tx = app.investigation.inv_tx.clone();
            let geo_service = app.geo.geoip.clone();
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
fn process_user_location(app: &mut App) {
    if let Ok(info) = app.geo.user_info_rx.try_recv() {
        app.geo.user_geo = Some(info);
    }
}
pub fn check_for_updates(app: &mut App) {
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    app.update.update_rx = Some(rx);
    let local = env!("CARGO_PKG_VERSION").to_string();
    app.ui.status_message = tr!(app.ui.translator, "status.checking_updates").to_string();

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
                Ok(r) if r.status() == 404 => Err("No GitHub releases published yet".to_string()),
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
fn process_update_result(app: &mut App) {
    if let Some(rx) = &app.update.update_rx {
        while let Ok(version) = rx.try_recv() {
            if version.starts_with("ERROR:") {
                let err_msg = version.trim_start_matches("ERROR:").to_string();
                app.ui.status_message = format!("[-] Update Check: {}", err_msg);
            } else if !version.is_empty() {
                app.update.latest_remote_version = version;
                app.update.show_update_dialog = true;
                app.ui.status_message =
                    tr!(app.ui.translator, "status.update_available").to_string();
            }
        }
    }
}
pub fn start_self_update(app: &mut App) {
    if app.update.latest_remote_version.is_empty() || app.update.is_updating {
        return;
    }
    app.update.is_updating = true;
    app.update.update_done = false;
    app.update.update_success = false;
    app.update.update_progress = 0.0;
    app.ui.status_message = tr!(app.ui.translator, "status.updating").to_string();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<crate::app::types::UpdateEvent>();
    app.update.update_task_rx = Some(rx);
    crate::app::installation::spawn_self_update(tx, app.update.latest_remote_version.clone());
}
fn process_update_task(app: &mut App) {
    if let Some(rx) = &mut app.update.update_task_rx {
        while let Ok(event) = rx.try_recv() {
            match event {
                crate::app::types::UpdateEvent::Progress(p) => {
                    app.update.update_progress = p;
                }
                crate::app::types::UpdateEvent::Finished(success, msg) => {
                    app.update.is_updating = false;
                    app.update.update_done = true;
                    app.update.update_success = success;
                    if success {
                        app.update.update_message =
                            tr!(app.ui.translator, "dialog.update_success_msg").to_string();
                        app.ui.status_message =
                            tr!(app.ui.translator, "status.update_done").to_string();
                    } else {
                        app.update.update_message = format!(
                            "{}: {}",
                            tr!(app.ui.translator, "dialog.update_failed_msg"),
                            msg
                        );
                        app.ui.status_message = tr!(app.ui.translator, "status.update_fail", msg);
                    }
                }
            }
        }
    }
}
fn process_geo_results(app: &mut App) {
    while let Ok((pid, ip, info)) = app.geo.geo_rx.try_recv() {
        app.geo.pending_geo_lookups = app.geo.pending_geo_lookups.saturating_sub(1);
        for app_conn in &mut app.network.app_connections {
            if app_conn.pid == pid {
                for conn in &mut app_conn.connections {
                    if conn.foreign_address == ip {
                        let flag = crate::services::geoip_service::GeoIpService::get_flag_emoji(
                            &info.countryCode,
                        );
                        conn.location = Some(format!("{} {}, {}", flag, info.city, info.country));
                        conn.isp = Some(info.isp.clone());
                    }
                }
            }
        }
    }
}
fn process_investigation_results(app: &mut App) {
    while let Ok(report) = app.investigation.inv_rx.try_recv() {
        let _ = app.database.save_investigation(
            &report.ip,
            report.domain.as_deref().unwrap_or("?"),
            report.risk_score,
            report.country.as_deref().unwrap_or("?"),
            report.isp.as_deref().unwrap_or("?"),
            report.whois_data.as_deref().unwrap_or(""),
        );
        app.investigation.investigation_report = Some(report);
        app.investigation.is_investigating = false;
        app.ui.status_message = tr!(app.ui.translator, "status.intel_ready").to_string();
    }
}
fn check_install_complete(app: &mut App) {
    if let Some(rx) = &mut app.install.child {
        if let Ok(output) = rx.try_recv() {
            app.install.child = None;
            if output.status.success() {
                app.install.installing = false;
                app.install.done = true;
                app.install.success = true;
                app.install.log = String::from_utf8_lossy(&output.stdout).to_string();
                app.install.message =
                    tr!(app.ui.translator, "dialog.net_tools_success").to_string();
                app.ui.status_message =
                    tr!(app.ui.translator, "status.install_installed").to_string();
                start_batch_analysis(app);
            } else if !app.install.needs_password {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.to_lowercase().contains("password") {
                    app.install.needs_password = true;
                    app.install.show_password_modal = true;
                    app.install.password.clear();
                    app.install.message =
                        tr!(app.ui.translator, "dialog.password_required").to_string();
                    app.ui.status_message =
                        tr!(app.ui.translator, "status.install_password").to_string();
                } else {
                    app.install.installing = false;
                    app.install.done = true;
                    app.install.success = false;
                    app.install.log = stderr.to_string();
                    let trimmed = stderr.trim().to_string();
                    app.install.message =
                        tr!(app.ui.translator, "dialog.net_tools_fail_msg", trimmed);
                    app.ui.status_message =
                        tr!(app.ui.translator, "status.install_failed").to_string();
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                app.install.log = format!("{}\n{}", stdout, stderr);
                if stderr.to_lowercase().contains("password")
                    || stderr.to_lowercase().contains("incorrect")
                    || stderr.to_lowercase().contains("try again")
                {
                    app.install.show_password_modal = true;
                    app.install.password.clear();
                    app.install.message =
                        tr!(app.ui.translator, "dialog.password_wrong").to_string();
                    app.ui.status_message =
                        tr!(app.ui.translator, "status.install_wrong_pw").to_string();
                } else {
                    app.install.installing = false;
                    app.install.done = true;
                    app.install.success = false;
                    let trimmed = stderr.trim().to_string();
                    app.install.message =
                        tr!(app.ui.translator, "dialog.net_tools_fail_msg", trimmed);
                    let last_line = stderr
                        .trim()
                        .lines()
                        .last()
                        .unwrap_or("unknown error")
                        .to_string();
                    app.ui.status_message =
                        tr!(app.ui.translator, "status.install_fail_detail", last_line);
                }
            }
        }
    }
}
fn check_nerdfont_install_complete(app: &mut App) {
    if let Some(rx) = &mut app.nerdfont.install_rx {
        if let Ok(msg) = rx.try_recv() {
            app.nerdfont.install_rx = None;
            app.nerdfont.install_done = true;
            app.nerdfont.install_success = msg.starts_with("Installed");
            app.nerdfont.install_message = msg.clone();
            app.ui.status_message = if app.nerdfont.install_success {
                tr!(app.ui.translator, "status.nerdfont_installed").to_string()
            } else {
                let first_line = msg.lines().next().unwrap_or("failed").to_string();
                tr!(app.ui.translator, "status.nerdfont_fail", first_line)
            };
        }
    }
}

fn process_status_messages(app: &mut App) {
    let mut messages = Vec::new();
    if let Some(ref rx) = app.ui.status_message_rx {
        while let Ok(msg) = rx.try_recv() {
            messages.push(msg);
        }
    }

    for msg in messages {
        app.ui.status_message = msg;
        app.ui.action_in_progress = None;
        app.start_batch_analysis();
    }
}
