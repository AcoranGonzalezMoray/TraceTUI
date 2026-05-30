use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Dashboard,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavView {
    Main,
    TrendGraphs,
    Storage,
    LibraryInspection,
    Containers,
    Agents,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentProvider {
    Ollama,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub api_url: String,
    pub models: Vec<String>,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentMission {
    ProcessAnalysis,
    NetworkAnalysis,
}
#[derive(Debug, Clone)]
pub enum AgentStatus {
    Idle,
    Queued,
    Running(String),
    Completed(String),
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct AgentLaunchData {
    pub mission: AgentMission,
    pub model: String,
    pub config: OllamaConfig,
    pub processes: Option<Vec<crate::app::process::ProcessInfo>>,
    pub networks: Option<(Vec<crate::app::network::NetworkConnection>, String)>,
}
#[derive(Debug, Clone)]
pub struct AgentInstance {
    pub mission: AgentMission,
    pub provider: AgentProvider,
    pub model: String,
    pub status: AgentStatus,
    pub started_at_frame: u64,
    pub completed_at_frame: Option<u64>,
    pub target_name: String,
    pub target_path: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarFocus {
    Nav,
    Left,
    Center,
    Right,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FirewallPanel {
    Connections,
    BlockedList,
    Actions,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConnection {
    pub process_name: String,
    pub process_path: String,
    pub icon: String,
    pub pid: u32,
    pub connections: Vec<crate::app::network::NetworkConnection>,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub risk_level: String,
    pub signature_status: crate::utils::signatures::SignatureStatus,
}
#[derive(Debug, Clone)]
pub enum UpdateEvent {
    Progress(f64),
    Finished(bool, String),
}
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum FileSortMode {
    ByName,
    BySize,
    ByDate,
}
impl FileSortMode {
    pub fn next(&self) -> Self {
        match self {
            Self::ByName => Self::BySize,
            Self::BySize => Self::ByDate,
            Self::ByDate => Self::ByName,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::ByName => "Name",
            Self::BySize => "Size",
            Self::ByDate => "Date",
        }
    }
}
#[derive(Debug, Clone, Default)]
pub struct FileSearchState {
    pub query: String,
    pub recursive: bool,
    pub extension_idx: usize,
    pub focused_field: usize,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfirmationAction {
    KillProcess,
    KillAllConnections,
}
