use crate::app::App;
use crate::tr;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
    pub ports: String,
    pub networks: String,
    pub created: String,
    pub running_for: String,
    pub size: String,
    pub cpu_percent: Option<f64>,
    pub memory_usage: String,
    pub memory_percent: Option<f64>,
    pub net_io: String,
    pub block_io: String,
    pub pids: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ContainerAction {
    Refresh,
    Logs,
    Start,
    Stop,
    Restart,
    PauseToggle,
}

impl ContainerAction {
    pub const COUNT: usize = 6;

    pub fn from_index(index: usize) -> Self {
        match index {
            1 => Self::Logs,
            2 => Self::Start,
            3 => Self::Stop,
            4 => Self::Restart,
            5 => Self::PauseToggle,
            _ => Self::Refresh,
        }
    }
}

impl App {
    pub fn refresh_containers_async(&mut self) {
        if self.containers_loading {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(ContainerManager::list());
        });
        self.container_rx = Some(rx);
        self.containers_loading = true;
        self.containers_error = None;
        self.status_message = tr!(self.translator, "containers.status.refreshing").to_string();
    }

    pub fn refresh_selected_container_logs_async(&mut self) {
        if self.container_logs_loading {
            return;
        }
        let Some(container) = self.get_selected_container() else {
            self.status_message =
                tr!(self.translator, "containers.status.no_selection").to_string();
            return;
        };
        let id = container.id.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(ContainerManager::logs(&id));
        });
        self.container_logs_rx = Some(rx);
        self.container_logs_loading = true;
        self.container_detail_scroll = 0;
        self.status_message = tr!(self.translator, "containers.status.loading_logs").to_string();
    }

    pub fn process_container_results(&mut self) {
        if let Some(rx) = &self.container_rx {
            if let Ok(result) = rx.try_recv() {
                self.containers_loading = false;
                self.container_rx = None;
                match result {
                    Ok(containers) => {
                        let count = containers.len();
                        self.containers = containers;
                        self.containers_loaded_once = true;
                        if self.selected_container_index >= count {
                            self.selected_container_index = count.saturating_sub(1);
                        }
                        self.containers_error = None;
                        self.status_message =
                            tr!(self.translator, "containers.status.ready", count);
                    }
                    Err(err) => {
                        self.containers_loaded_once = true;
                        self.containers_error = Some(err.clone());
                        self.status_message = tr!(self.translator, "containers.status.error", err);
                    }
                }
            }
        }

        if let Some(rx) = &self.container_logs_rx {
            if let Ok(result) = rx.try_recv() {
                self.container_logs_loading = false;
                self.container_logs_rx = None;
                match result {
                    Ok(logs) => {
                        let count = logs.len();
                        self.container_logs = logs;
                        self.status_message =
                            tr!(self.translator, "containers.status.logs_ready", count);
                    }
                    Err(err) => {
                        self.container_logs.clear();
                        self.status_message = tr!(self.translator, "containers.status.error", err);
                    }
                }
            }
        }
    }

    pub fn execute_container_action(&mut self) {
        match ContainerAction::from_index(self.selected_container_action_index) {
            ContainerAction::Refresh => self.refresh_containers_async(),
            ContainerAction::Logs => self.refresh_selected_container_logs_async(),
            action => self.run_selected_container_action(action),
        }
    }

    fn run_selected_container_action(&mut self, action: ContainerAction) {
        let Some(container) = self.get_selected_container() else {
            self.status_message =
                tr!(self.translator, "containers.status.no_selection").to_string();
            return;
        };
        let id = container.id.clone();
        let name = container.name.clone();
        let is_paused = container.state.eq_ignore_ascii_case("paused");
        let result = match action {
            ContainerAction::Start => ContainerManager::run_action("start", &id),
            ContainerAction::Stop => ContainerManager::run_action("stop", &id),
            ContainerAction::Restart => ContainerManager::run_action("restart", &id),
            ContainerAction::PauseToggle if is_paused => {
                ContainerManager::run_action("unpause", &id)
            }
            ContainerAction::PauseToggle => ContainerManager::run_action("pause", &id),
            ContainerAction::Refresh | ContainerAction::Logs => Ok(()),
        };

        match result {
            Ok(()) => {
                self.status_message = tr!(self.translator, "containers.status.action_done", name);
                self.refresh_containers_async();
            }
            Err(err) => {
                self.status_message = tr!(self.translator, "containers.status.error", err);
            }
        }
    }
}

pub struct ContainerManager;

impl ContainerManager {
    pub fn list() -> Result<Vec<ContainerInfo>, String> {
        let ps_output = Command::new("docker")
            .args(["ps", "-a", "--no-trunc", "--format", "{{json .}}"])
            .output()
            .map_err(|e| format!("docker ps: {}", e))?;

        if !ps_output.status.success() {
            return Err(command_error("docker ps", &ps_output.stderr));
        }

        let mut containers = Vec::new();
        let stdout = String::from_utf8_lossy(&ps_output.stdout);
        for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
            let value: Value =
                serde_json::from_str(line).map_err(|e| format!("docker ps JSON: {}", e))?;
            containers.push(ContainerInfo {
                id: short_id(field(&value, "ID")),
                name: field(&value, "Names"),
                image: field(&value, "Image"),
                status: field(&value, "Status"),
                state: field(&value, "State"),
                ports: empty_as_dash(field(&value, "Ports")),
                networks: empty_as_dash(field(&value, "Networks")),
                created: field(&value, "CreatedAt"),
                running_for: field(&value, "RunningFor"),
                size: empty_as_dash(field(&value, "Size")),
                ..Default::default()
            });
        }

        let stats = Self::stats().unwrap_or_default();
        for container in &mut containers {
            if let Some(stat) = stats
                .get(&container.id)
                .or_else(|| stats.get(&container.name))
            {
                container.cpu_percent = stat.cpu_percent;
                container.memory_usage = stat.memory_usage.clone();
                container.memory_percent = stat.memory_percent;
                container.net_io = stat.net_io.clone();
                container.block_io = stat.block_io.clone();
                container.pids = stat.pids.clone();
            }
        }

        Ok(containers)
    }

    pub fn logs(id: &str) -> Result<Vec<String>, String> {
        let output = Command::new("docker")
            .args(["logs", "--tail", "80", id])
            .output()
            .map_err(|e| format!("docker logs: {}", e))?;

        if !output.status.success() {
            return Err(command_error("docker logs", &output.stderr));
        }

        let mut lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.to_string())
            .collect();
        let stderr = String::from_utf8_lossy(&output.stderr);
        lines.extend(stderr.lines().map(|line| line.to_string()));
        Ok(lines)
    }

    pub fn run_action(action: &str, id: &str) -> Result<(), String> {
        let output = Command::new("docker")
            .args([action, id])
            .output()
            .map_err(|e| format!("docker {}: {}", action, e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(command_error(&format!("docker {}", action), &output.stderr))
        }
    }

    fn stats() -> Result<HashMap<String, ContainerInfo>, String> {
        let output = Command::new("docker")
            .args(["stats", "--no-stream", "--format", "{{json .}}"])
            .output()
            .map_err(|e| format!("docker stats: {}", e))?;

        if !output.status.success() {
            return Err(command_error("docker stats", &output.stderr));
        }

        let mut stats = HashMap::new();
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
            let value: Value =
                serde_json::from_str(line).map_err(|e| format!("docker stats JSON: {}", e))?;
            let id = short_id(field(&value, "Container"));
            let stat = ContainerInfo {
                id: id.clone(),
                name: field(&value, "Name"),
                cpu_percent: parse_percent(&field(&value, "CPUPerc")),
                memory_usage: empty_as_dash(field(&value, "MemUsage")),
                memory_percent: parse_percent(&field(&value, "MemPerc")),
                net_io: empty_as_dash(field(&value, "NetIO")),
                block_io: empty_as_dash(field(&value, "BlockIO")),
                pids: empty_as_dash(field(&value, "PIDs")),
                ..Default::default()
            };
            stats.insert(id, stat.clone());
            stats.insert(stat.name.clone(), stat);
        }
        Ok(stats)
    }
}

fn field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn short_id(id: String) -> String {
    id.chars().take(12).collect()
}

fn empty_as_dash(value: String) -> String {
    if value.trim().is_empty() {
        "-".to_string()
    } else {
        value
    }
}

fn parse_percent(value: &str) -> Option<f64> {
    value.trim().trim_end_matches('%').parse::<f64>().ok()
}

fn command_error(command: &str, stderr: &[u8]) -> String {
    let message = String::from_utf8_lossy(stderr).trim().to_string();
    if message.is_empty() {
        format!("{} failed", command)
    } else {
        message
    }
}
