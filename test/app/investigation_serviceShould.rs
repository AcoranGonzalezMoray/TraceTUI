#[cfg(test)]
mod investigation_service_tests {
    use crate::app::investigation_service::InvestigationReport;

    #[test]
    fn test_investigation_report_new() {
        let report = InvestigationReport::new("8.8.8.8".to_string(), 443);
        assert_eq!(report.ip, "8.8.8.8");
        assert_eq!(report.port, 443);
    }

    #[test]
    fn test_investigation_report_defaults() {
        let report = InvestigationReport::new("1.2.3.4".to_string(), 80);
        assert!(report.domain.is_none());
        assert!(report.organization.is_none());
        assert!(report.city.is_none());
        assert!(report.country.is_none());
        assert!(report.isp.is_none());
        assert!(report.ping_ms.is_none());
        assert!(report.hops.is_empty());
        assert!(report.hop_coords.is_empty());
        assert_eq!(report.risk_score, 0);
        assert!(report.risk_factors.is_empty());
        assert_eq!(report.lat, 0.0);
        assert_eq!(report.lon, 0.0);
        assert!(report.whois_data.is_none());
    }

    #[test]
    fn test_investigation_report_serde() {
        let mut report = InvestigationReport::new("8.8.8.8".to_string(), 443);
        report.domain = Some("dns.google".to_string());
        report.organization = Some("Google LLC".to_string());
        report.country = Some("United States".to_string());
        report.risk_score = 25;
        report.risk_factors = vec!["Test factor".to_string()];

        let json = serde_json::to_string(&report).unwrap();
        let deserialized: InvestigationReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.ip, "8.8.8.8");
        assert_eq!(deserialized.port, 443);
        assert_eq!(deserialized.domain.unwrap(), "dns.google");
        assert_eq!(deserialized.risk_score, 25);
        assert_eq!(deserialized.risk_factors.len(), 1);
    }

    #[test]
    fn test_investigation_report_debug() {
        let report = InvestigationReport::new("127.0.0.1".to_string(), 8080);
        let debug = format!("{:?}", report);
        assert!(debug.contains("127.0.0.1"));
        assert!(debug.contains("8080"));
    }

    #[test]
    fn test_investigation_report_new_with_path() {
        let report = InvestigationReport::new("10.0.0.1".to_string(), 53);
        assert!(report.hop_coords.is_empty());
        assert_eq!(report.ip, "10.0.0.1");
    }

    #[test]
    fn test_investigation_report_clone() {
        let report = InvestigationReport::new("1.1.1.1".to_string(), 853);
        let cloned = report.clone();
        assert_eq!(cloned.ip, report.ip);
        assert_eq!(cloned.port, report.port);
        assert_eq!(cloned.risk_score, report.risk_score);
    }
}
