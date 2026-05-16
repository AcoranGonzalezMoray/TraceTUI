#[cfg(test)]
mod firewall_service_tests {
    use crate::app::firewall_service::FirewallManager;
    use crate::app::types::FirewallPanel;

    #[test]
    fn test_get_firewall_action_count() {
        assert_eq!(FirewallManager::get_firewall_action_count(), 3);
    }

    #[test]
    fn test_cycle_focus_forward() {
        assert_eq!(
            FirewallManager::cycle_focus_forward(FirewallPanel::Connections),
            FirewallPanel::BlockedList
        );
        assert_eq!(
            FirewallManager::cycle_focus_forward(FirewallPanel::BlockedList),
            FirewallPanel::Actions
        );
        assert_eq!(
            FirewallManager::cycle_focus_forward(FirewallPanel::Actions),
            FirewallPanel::Connections
        );
    }

    #[test]
    fn test_cycle_focus_backward() {
        assert_eq!(
            FirewallManager::cycle_focus_backward(FirewallPanel::Connections),
            FirewallPanel::Actions
        );
        assert_eq!(
            FirewallManager::cycle_focus_backward(FirewallPanel::BlockedList),
            FirewallPanel::Connections
        );
        assert_eq!(
            FirewallManager::cycle_focus_backward(FirewallPanel::Actions),
            FirewallPanel::BlockedList
        );
    }

    #[test]
    fn test_panel_from_x_connections() {
        let panel = FirewallManager::panel_from_x_with_width(0, 100);
        assert_eq!(panel, FirewallPanel::Connections);
        let panel = FirewallManager::panel_from_x_with_width(32, 100);
        assert_eq!(panel, FirewallPanel::Connections);
    }

    #[test]
    fn test_panel_from_x_blocked() {
        let panel = FirewallManager::panel_from_x_with_width(33, 100);
        assert_eq!(panel, FirewallPanel::BlockedList);
        let panel = FirewallManager::panel_from_x_with_width(65, 100);
        assert_eq!(panel, FirewallPanel::BlockedList);
    }

    #[test]
    fn test_panel_from_x_actions() {
        let panel = FirewallManager::panel_from_x_with_width(66, 100);
        assert_eq!(panel, FirewallPanel::Actions);
        let panel = FirewallManager::panel_from_x_with_width(100, 100);
        assert_eq!(panel, FirewallPanel::Actions);
    }

    #[test]
    fn test_firewall_panel_debug() {
        assert_eq!(format!("{:?}", FirewallPanel::Connections), "Connections");
        assert_eq!(format!("{:?}", FirewallPanel::BlockedList), "BlockedList");
        assert_eq!(format!("{:?}", FirewallPanel::Actions), "Actions");
    }

    #[test]
    fn test_firewall_panel_partial_eq() {
        assert_eq!(FirewallPanel::Connections, FirewallPanel::Connections);
        assert_ne!(FirewallPanel::Connections, FirewallPanel::BlockedList);
    }
}
