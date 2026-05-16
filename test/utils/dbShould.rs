#[cfg(test)]
mod db_tests {
    use crate::utils::db::Database;

    #[test]
    fn test_database_new() {
        let db = Database::new();
        assert!(db.is_ok());
    }

    #[test]
    fn test_get_blocked_ips_empty_at_start() {
        if let Ok(db) = Database::new() {
            let blocked = db.get_blocked_ips().unwrap();
            assert!(blocked.is_empty());
        }
    }

    #[test]
    fn test_save_get_investigation() {
        if let Ok(db) = Database::new() {
            db.save_investigation("8.8.8.8", "test_proc", 25, "US", "Google", "whois data")
                .unwrap();
        }
    }

    #[test]
    fn test_add_firewall_block() {
        if let Ok(db) = Database::new() {
            db.add_firewall_block("10.0.0.1", "test", "rule_1").unwrap();
            let blocked = db.get_blocked_ips().unwrap();
            let exists = blocked.iter().any(|(ip, _, _)| ip == "10.0.0.1");
            assert!(exists);
            db.remove_firewall_block("10.0.0.1").unwrap();
        }
    }

    #[test]
    fn test_remove_firewall_block() {
        if let Ok(db) = Database::new() {
            db.add_firewall_block("192.168.1.1", "proc", "rule_x")
                .unwrap();
            db.remove_firewall_block("192.168.1.1").unwrap();
            let blocked = db.get_blocked_ips().unwrap();
            let exists = blocked.iter().any(|(ip, _, _)| ip == "192.168.1.1");
            assert!(!exists);
        }
    }

    #[test]
    fn test_add_duplicate_firewall_block() {
        if let Ok(db) = Database::new() {
            db.add_firewall_block("10.0.0.2", "proc1", "rule_a")
                .unwrap();
            db.add_firewall_block("10.0.0.2", "proc2", "rule_b")
                .unwrap();
            let blocked = db.get_blocked_ips().unwrap();
            let matches: Vec<_> = blocked
                .iter()
                .filter(|(ip, _, _)| ip == "10.0.0.2")
                .collect();
            assert_eq!(matches.len(), 1);
            db.remove_firewall_block("10.0.0.2").unwrap();
        }
    }

    #[test]
    fn test_get_blocked_ips_returns_tuples() {
        if let Ok(db) = Database::new() {
            db.add_firewall_block("1.2.3.4", "notepad", "rule_n")
                .unwrap();
            let blocked = db.get_blocked_ips().unwrap();
            let entry = blocked.iter().find(|(ip, _, _)| ip == "1.2.3.4");
            assert!(entry.is_some());
            if let Some((_, pname, rule)) = entry {
                assert_eq!(pname, "notepad");
                assert_eq!(rule, "rule_n");
            }
            db.remove_firewall_block("1.2.3.4").unwrap();
        }
    }
}
