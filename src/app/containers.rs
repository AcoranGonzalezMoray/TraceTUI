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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockerStatus {
    On,
    Starting,
    Off,
    Missing,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum ContainerAction {
    Refresh,
    Logs,
    Console,
    Start,
    Stop,
    Restart,
    PauseToggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockerAction {
    StartDocker,
    StopDocker,
    SearchDockerHub,
}

impl ContainerAction {
    pub const COUNT: usize = 7;

    pub fn from_index(index: usize) -> Self {
        match index {
            1 => Self::Logs,
            2 => Self::Console,
            3 => Self::Start,
            4 => Self::Stop,
            5 => Self::Restart,
            6 => Self::PauseToggle,
            _ => Self::Refresh,
        }
    }
}

impl DockerAction {
    pub const COUNT: usize = 3;

    pub fn from_index(index: usize) -> Self {
        match index {
            1 => Self::StopDocker,
            2 => Self::SearchDockerHub,
            _ => Self::StartDocker,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DockerHubImage {
    pub name: String,
    pub description: String,
    pub official: bool,
    pub automated: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DockerHubSearchState {
    pub search_query: String,
    pub results: Vec<DockerHubImage>,
    pub selected_result_index: usize,
    pub container_name: String,
    pub ports: String,
    pub env_vars: String,
    pub focused_field: usize, // 0: search, 1: container_name, 2: ports, 3: env_vars, 4: create button, 5: cancel button
}

pub const DOCKER_ACTION_OFFSET: usize = ContainerAction::COUNT;
pub const CONTAINER_RIGHT_ACTION_COUNT: usize = ContainerAction::COUNT + DockerAction::COUNT;

impl App {
    pub fn docker_status(&self) -> DockerStatus {
        if self.containers_loading {
            DockerStatus::Starting
        } else if let Some(err) = &self.containers_error {
            ContainerManager::classify_error(err)
        } else if self.containers_loaded_once {
            DockerStatus::On
        } else {
            DockerStatus::Unknown
        }
    }

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
        self.container_logs_scroll = 0;
        self.show_container_logs_modal = true;
        self.status_message = tr!(self.translator, "containers.status.loading_logs").to_string();
    }

    pub fn open_selected_container_console(&mut self) {
        let Some(container) = self.get_selected_container() else {
            self.status_message =
                tr!(self.translator, "containers.status.no_selection").to_string();
            return;
        };
        let name = container.name.clone();
        self.show_container_console_modal = true;
        self.container_console_scroll = 0;
        if self.container_console_output.is_empty() {
            self.container_console_output.push(tr!(
                self.translator,
                "containers.console_welcome",
                name
            ));
        }
    }

    pub fn execute_container_console_command_async(&mut self) {
        if self.container_console_loading {
            return;
        }
        let command = self.container_console_input.trim().to_string();
        if command.is_empty() {
            return;
        }
        let Some(container) = self.get_selected_container() else {
            self.status_message =
                tr!(self.translator, "containers.status.no_selection").to_string();
            return;
        };
        let id = container.id.clone();
        self.container_console_output
            .push(format!("$ {}", command.as_str()));
        self.container_console_input.clear();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(ContainerManager::exec_shell_command(&id, &command));
        });
        self.container_console_rx = Some(rx);
        self.container_console_loading = true;
        self.status_message = tr!(self.translator, "containers.status.console_running").to_string();
    }

    pub fn process_container_results(&mut self) {
        if let Some(rx) = &self.container_rx {
            if let Ok(result) = rx.try_recv() {
                self.containers_loading = false;
                self.container_rx = None;
                self.containers_loaded_once = true;
                match result {
                    Ok(containers) => {
                        let count = containers.len();
                        self.containers = containers;
                        if self.selected_container_index >= count {
                            self.selected_container_index = count.saturating_sub(1);
                        }
                        self.containers_error = None;
                        self.status_message =
                            tr!(self.translator, "containers.status.ready", count);
                    }
                    Err(err) => {
                        self.containers.clear();
                        self.containers_error = Some(err.clone());
                        self.status_message = self
                            .translator
                            .get(docker_status_key(ContainerManager::classify_error(&err)))
                            .to_string();
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

        if let Some(rx) = &self.container_console_rx {
            if let Ok(result) = rx.try_recv() {
                self.container_console_loading = false;
                self.container_console_rx = None;
                match result {
                    Ok(lines) => {
                        if lines.is_empty() {
                            self.container_console_output
                                .push(tr!(self.translator, "containers.console_empty_output"));
                        } else {
                            self.container_console_output.extend(lines);
                        }
                        self.status_message =
                            tr!(self.translator, "containers.status.console_done").to_string();
                    }
                    Err(err) => {
                        self.container_console_output.push(err.clone());
                        self.status_message = tr!(self.translator, "containers.status.error", err);
                    }
                }
            }
        }

        if let Some(rx) = &self.docker_hub_search_rx {
            if let Ok(result) = rx.try_recv() {
                self.docker_hub_search_rx = None;
                match result {
                    Ok(images) => {
                        self.docker_hub_search.results = images;
                        self.docker_hub_search.selected_result_index = 0;
                        let count = self.docker_hub_search.results.len();
                        self.status_message = tr!(
                            self.translator,
                            "containers.docker_hub_results_found",
                            count
                        );
                    }
                    Err(err) => {
                        self.docker_hub_search.results.clear();
                        self.status_message =
                            tr!(self.translator, "containers.docker_hub_error", err);
                    }
                }
            }
        }

        if let Some(rx) = &self.docker_hub_create_rx {
            if let Ok(result) = rx.try_recv() {
                self.docker_hub_create_rx = None;
                match result {
                    Ok(container_id) => {
                        self.status_message = tr!(
                            self.translator,
                            "containers.docker_hub_created",
                            container_id
                        );
                        self.show_docker_hub_modal = false;
                        self.docker_hub_search = DockerHubSearchState::default();
                        self.refresh_containers_async();
                    }
                    Err(err) => {
                        self.status_message =
                            tr!(self.translator, "containers.docker_hub_error", err);
                    }
                }
            }
        }
    }

    pub fn execute_container_right_action(&mut self) {
        if self.selected_container_action_index >= DOCKER_ACTION_OFFSET {
            let docker_index = self
                .selected_container_action_index
                .saturating_sub(DOCKER_ACTION_OFFSET);
            let action = DockerAction::from_index(docker_index);
            match action {
                DockerAction::SearchDockerHub => {
                    self.show_docker_hub_modal = true;
                    self.docker_hub_search = DockerHubSearchState::default();
                }
                DockerAction::StartDocker => {
                    self.pending_docker_action = Some(DockerAction::StartDocker);
                    self.pending_container_action = None;
                    self.confirmation_message =
                        tr!(self.translator, "dialog.start_docker_confirm").to_string();
                    self.show_confirmation = true;
                }
                DockerAction::StopDocker => {
                    self.pending_docker_action = Some(DockerAction::StopDocker);
                    self.pending_container_action = None;
                    self.confirmation_message =
                        tr!(self.translator, "dialog.stop_docker_confirm").to_string();
                    self.show_confirmation = true;
                }
            }
            return;
        }

        match ContainerAction::from_index(self.selected_container_action_index) {
            ContainerAction::Refresh => self.refresh_containers_async(),
            ContainerAction::Logs => self.refresh_selected_container_logs_async(),
            ContainerAction::Console => self.open_selected_container_console(),
            ContainerAction::Start => {
                if let Some(c) = self.get_selected_container() {
                    let container_name = c.name.clone();
                    self.pending_docker_action = None;
                    self.pending_container_action = Some(ContainerAction::Start);
                    self.confirmation_message = tr!(
                        self.translator,
                        "dialog.start_container_confirm",
                        &container_name
                    )
                    .to_string();
                    self.show_confirmation = true;
                } else {
                    self.status_message =
                        tr!(self.translator, "containers.status.no_selection").to_string();
                }
            }
            ContainerAction::Stop => {
                if let Some(c) = self.get_selected_container() {
                    let container_name = c.name.clone();
                    self.pending_docker_action = None;
                    self.pending_container_action = Some(ContainerAction::Stop);
                    self.confirmation_message = tr!(
                        self.translator,
                        "dialog.stop_container_confirm",
                        &container_name
                    )
                    .to_string();
                    self.show_confirmation = true;
                } else {
                    self.status_message =
                        tr!(self.translator, "containers.status.no_selection").to_string();
                }
            }
            ContainerAction::Restart => {
                if let Some(c) = self.get_selected_container() {
                    let container_name = c.name.clone();
                    self.pending_docker_action = None;
                    self.pending_container_action = Some(ContainerAction::Restart);
                    self.confirmation_message = tr!(
                        self.translator,
                        "dialog.restart_container_confirm",
                        &container_name
                    )
                    .to_string();
                    self.show_confirmation = true;
                } else {
                    self.status_message =
                        tr!(self.translator, "containers.status.no_selection").to_string();
                }
            }
            ContainerAction::PauseToggle => {
                if let Some(c) = self.get_selected_container() {
                    let container_name = c.name.clone();
                    let is_paused = c.state.eq_ignore_ascii_case("paused");
                    self.pending_docker_action = None;
                    self.pending_container_action = Some(ContainerAction::PauseToggle);
                    let key = if is_paused {
                        "dialog.unpause_container_confirm"
                    } else {
                        "dialog.pause_container_confirm"
                    };
                    self.confirmation_message = self
                        .translator
                        .get_fmt(key, &[format!("{}", container_name)])
                        .to_string();
                    self.show_confirmation = true;
                } else {
                    self.status_message =
                        tr!(self.translator, "containers.status.no_selection").to_string();
                }
            }
        }
    }

    pub fn execute_docker_action_confirmed(&mut self, action: DockerAction) {
        let result = match action {
            DockerAction::StartDocker => ContainerManager::start_docker(),
            DockerAction::StopDocker => ContainerManager::stop_docker(),
            _ => Ok(()),
        };

        match result {
            Ok(()) => {
                self.containers_loaded_once = false;
                self.status_message = match action {
                    DockerAction::StartDocker => {
                        tr!(self.translator, "containers.status.docker_start_requested")
                    }
                    DockerAction::StopDocker => {
                        tr!(self.translator, "containers.status.docker_stop_requested")
                    }
                    _ => String::new(),
                };
                self.refresh_containers_async();
            }
            Err(err) => {
                self.status_message = tr!(self.translator, "containers.status.error", err);
            }
        }
    }

    pub fn run_selected_container_action_confirmed(&mut self, action: ContainerAction) {
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
            ContainerAction::Refresh | ContainerAction::Logs | ContainerAction::Console => Ok(()),
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
            .args(["logs", "--tail", "300", id])
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

    pub fn exec_shell_command(id: &str, command: &str) -> Result<Vec<String>, String> {
        let output = Command::new("docker")
            .args(["exec", id, "sh", "-lc", command])
            .output()
            .map_err(|e| format!("docker exec: {}", e))?;

        let mut lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.to_string())
            .collect();
        let stderr = String::from_utf8_lossy(&output.stderr);
        lines.extend(stderr.lines().map(|line| line.to_string()));

        if output.status.success() {
            Ok(lines)
        } else if lines.is_empty() {
            Err(command_error("docker exec", &output.stderr))
        } else {
            Ok(lines)
        }
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

    pub fn start_docker() -> Result<(), String> {
        docker_service_command(true)
    }

    pub fn stop_docker() -> Result<(), String> {
        docker_service_command(false)
    }

    pub fn classify_error(error: &str) -> DockerStatus {
        let lower = error.to_lowercase();
        if lower.contains("program not found")
            || lower.contains("not recognized")
            || lower.contains("no such file")
            || lower.contains("executable")
            || lower.contains("docker ps:")
        {
            DockerStatus::Missing
        } else if lower.contains("cannot connect")
            || lower.contains("connection refused")
            || lower.contains("daemon")
            || lower.contains("pipe")
        {
            DockerStatus::Off
        } else {
            DockerStatus::Unknown
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

    pub fn search_docker_hub(
        query: &str,
    ) -> Result<Vec<crate::app::containers::DockerHubImage>, String> {
        let search_output = Command::new("docker")
            .args(["search", query, "--format", "{{json .}}"])
            .output()
            .map_err(|e| format!("docker search: {}", e))?;

        if !search_output.status.success() {
            return Err(command_error("docker search", &search_output.stderr));
        }

        let mut images = Vec::new();
        let stdout = String::from_utf8_lossy(&search_output.stdout);
        for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
            if let Ok(value) = serde_json::from_str::<Value>(line) {
                let name = field(&value, "Name");
                let description = field(&value, "Description");
                let official = field(&value, "Official").to_lowercase() == "ok";
                let automated = field(&value, "Automated").to_lowercase() == "ok";

                images.push(crate::app::containers::DockerHubImage {
                    name,
                    description,
                    official,
                    automated,
                });
            }
        }

        Ok(images)
    }

    pub fn create_and_run(
        image: &str,
        name: &str,
        ports: &str,
        env_vars: &str,
    ) -> Result<String, String> {
        let mut args = vec!["run", "-d"];

        // Add name if provided
        if !name.is_empty() {
            args.push("--name");
            args.push(name);
        }

        // Add port mappings if provided
        let port_list: Vec<&str> = if !ports.is_empty() {
            ports.split(',').map(|p| p.trim()).collect()
        } else {
            Vec::new()
        };

        for port in &port_list {
            if !port.is_empty() {
                args.push("-p");
                args.push(port);
            }
        }

        // Add environment variables if provided
        let env_list: Vec<&str> = if !env_vars.is_empty() {
            env_vars.split(',').map(|e| e.trim()).collect()
        } else {
            Vec::new()
        };

        for env in &env_list {
            if !env.is_empty() {
                args.push("-e");
                args.push(env);
            }
        }

        // Finally, add image name
        args.push(image);

        let output = Command::new("docker")
            .args(&args)
            .output()
            .map_err(|e| format!("docker run: {}", e))?;

        if output.status.success() {
            let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(short_id(container_id))
        } else {
            Err(command_error("docker run", &output.stderr))
        }
    }
}

fn docker_status_key(status: DockerStatus) -> &'static str {
    match status {
        DockerStatus::Missing => "containers.status.docker_missing",
        DockerStatus::Off => "containers.status.docker_off",
        DockerStatus::Starting => "containers.status.docker_starting",
        DockerStatus::On => "containers.status.docker_on",
        DockerStatus::Unknown => "containers.status.docker_unknown",
    }
}

fn docker_service_command(start: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let output = {
        let command = if start {
            "Start-Service -Name com.docker.service"
        } else {
            "Stop-Service -Name com.docker.service"
        };
        Command::new("powershell")
            .args(["-NoProfile", "-Command", command])
            .output()
    };

    #[cfg(target_os = "linux")]
    let output = {
        let action = if start { "start" } else { "stop" };
        Command::new("systemctl").args([action, "docker"]).output()
    };

    #[cfg(target_os = "macos")]
    let output = {
        if start {
            Command::new("open").args(["-a", "Docker"]).output()
        } else {
            Command::new("osascript")
                .args(["-e", "quit app \"Docker\""])
                .output()
        }
    };

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    let output: Result<std::process::Output, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "unsupported OS",
    ));

    let output = output.map_err(|e| format!("docker service: {}", e))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(command_error("docker service", &output.stderr))
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
