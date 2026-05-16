use crate::app::types::FirewallPanel;
use crate::config;
use crate::utils::db::Database;
pub struct FirewallManager;
impl FirewallManager {
    pub fn block_ip(ip: &str, process_name: &str, database: &Database) -> String {
        let rule_name = format!("{}{}", config::FIREWALL_RULE_PREFIX, ip.replace('.', "_"));
        let _ = std::process::Command::new("netsh")
            .args([
                "advfirewall",
                "firewall",
                "add",
                "rule",
                &format!("name={}", rule_name),
                "dir=out",
                "action=block",
                &format!("remoteip={}", ip),
            ])
            .output();
        let _ = database.add_firewall_block(ip, process_name, &rule_name);
        format!("[!] IP {} BLOCKED via Firewall", ip)
    }
    pub fn unblock_ip(ip: &str, database: &Database) {
        let rule_name = format!("{}{}", config::FIREWALL_RULE_PREFIX, ip.replace('.', "_"));
        let _ = std::process::Command::new("netsh")
            .args([
                "advfirewall",
                "firewall",
                "delete",
                "rule",
                &format!("name={}", rule_name),
            ])
            .output();
        let _ = database.remove_firewall_block(ip);
    }
    pub fn get_firewall_action_count() -> usize {
        3
    }
    pub fn cycle_focus_forward(focus: FirewallPanel) -> FirewallPanel {
        match focus {
            FirewallPanel::Connections => FirewallPanel::BlockedList,
            FirewallPanel::BlockedList => FirewallPanel::Actions,
            FirewallPanel::Actions => FirewallPanel::Connections,
        }
    }
    pub fn cycle_focus_backward(focus: FirewallPanel) -> FirewallPanel {
        match focus {
            FirewallPanel::Connections => FirewallPanel::Actions,
            FirewallPanel::BlockedList => FirewallPanel::Connections,
            FirewallPanel::Actions => FirewallPanel::BlockedList,
        }
    }
    pub fn panel_from_x(x: u16) -> FirewallPanel {
        let (term_width, _) = crossterm::terminal::size()
            .unwrap_or((config::DEFAULT_TERM_WIDTH, config::DEFAULT_TERM_HEIGHT));
        Self::panel_from_x_with_width(x, term_width)
    }
    pub fn panel_from_x_with_width(x: u16, term_width: u16) -> FirewallPanel {
        let conn = (term_width as f32 * config::FIREWALL_CLICK_CONN_BOUNDARY as f32 / 100.0) as u16;
        let blocked =
            (term_width as f32 * config::FIREWALL_CLICK_BLOCKED_BOUNDARY as f32 / 100.0) as u16;
        if x < conn {
            FirewallPanel::Connections
        } else if x < blocked {
            FirewallPanel::BlockedList
        } else {
            FirewallPanel::Actions
        }
    }
}
