#[cfg(test)]
mod types_tests {
    use crate::app::types::{AppState, FirewallPanel, SidebarFocus};

    #[test]
    fn test_app_state_debug() {
        let state = AppState::Dashboard;
        assert_eq!(format!("{:?}", state), "Dashboard");
    }

    #[test]
    fn test_sidebar_focus_cycling() {
        assert_eq!(SidebarFocus::Left as u8, 0);
        assert_eq!(SidebarFocus::Center as u8, 1);
        assert_eq!(SidebarFocus::Right as u8, 2);
    }

    #[test]
    fn test_firewall_panel_equality() {
        assert_eq!(FirewallPanel::Connections, FirewallPanel::Connections);
        assert_eq!(FirewallPanel::BlockedList, FirewallPanel::BlockedList);
        assert_eq!(FirewallPanel::Actions, FirewallPanel::Actions);
        assert_ne!(FirewallPanel::Connections, FirewallPanel::Actions);
    }

    #[test]
    fn test_app_state_partial_eq() {
        assert_eq!(AppState::Dashboard, AppState::Dashboard);
    }

    #[test]
    fn test_sidebar_focus_partial_eq() {
        assert_eq!(SidebarFocus::Left, SidebarFocus::Left);
        assert_ne!(SidebarFocus::Left, SidebarFocus::Right);
    }
}
