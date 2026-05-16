use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Dashboard,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarFocus {
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
