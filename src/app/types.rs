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
#[derive(Debug, Clone)]
pub struct FileSearchState {
    pub query: String,
    pub recursive: bool,
    pub extension_idx: usize,
    pub focused_field: usize,
}
impl Default for FileSearchState {
    fn default() -> Self {
        Self {
            query: String::new(),
            recursive: false,
            extension_idx: 0,
            focused_field: 0,
        }
    }
}
