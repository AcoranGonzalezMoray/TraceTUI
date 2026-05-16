use crate::app::network::NetworkConnection;
use crate::app::process::ProcessInfo;
use crate::app::risk::RiskAnalyzer;
use crate::app::types::AppConnection;
use crate::config;
use crate::utils::signatures::{SignatureStatus, SignatureVerifier};
use std::collections::{HashMap, HashSet};
pub struct ConnectionGrouper;
impl ConnectionGrouper {
    pub fn group(
        processes: &[ProcessInfo],
        network_connections: &[NetworkConnection],
        hunter_mode: bool,
        mut icon_callback: impl FnMut(&str, &str) -> String,
    ) -> Vec<AppConnection> {
        let pids_with_connections: HashSet<u32> =
            network_connections.iter().map(|c| c.pid).collect();
        let mut app_map: HashMap<u32, AppConnection> = HashMap::new();
        for process in processes {
            if !pids_with_connections.contains(&process.pid) {
                continue;
            }
            let exe_path = process.path.as_deref().unwrap_or("");
            let icon = icon_callback(exe_path, &process.name);
            let sig_status = SignatureVerifier::verify(exe_path);
            if hunter_mode
                && sig_status == SignatureStatus::Valid
                && config::KNOWN_SAFE_PROCESSES
                    .iter()
                    .any(|&s| process.name.to_lowercase().contains(s))
            {
                continue;
            }
            let risk_level = RiskAnalyzer::calculate(process, network_connections);
            let conns_for_pid: Vec<NetworkConnection> = network_connections
                .iter()
                .filter(|c| c.pid == process.pid)
                .cloned()
                .collect();
            app_map.insert(
                process.pid,
                AppConnection {
                    process_name: process.name.clone(),
                    process_path: process.path.clone().unwrap_or_default(),
                    icon,
                    pid: process.pid,
                    connections: conns_for_pid,
                    cpu_usage: process.cpu_usage,
                    memory_usage: process.memory_usage,
                    risk_level,
                    signature_status: sig_status,
                },
            );
        }
        let mut result: Vec<AppConnection> = app_map.into_values().collect();
        result.sort_by_key(|b| std::cmp::Reverse(b.connections.len()));
        result
    }
}
