#[cfg(test)]
mod whois_tests {
    use crate::utils::whois::WhoisService;

    #[test]
    fn test_clean_whois_removes_comments() {
        let raw = "% comment line\n\
                   # hash comment\n\
                   key: value\n\
                   \n\
                   another: data";
        let cleaned = WhoisService::clean_whois(raw.to_string());
        assert!(!cleaned.contains('%'));
        assert!(!cleaned.contains('#'));
        assert!(cleaned.contains("key: value"));
        assert!(cleaned.contains("another: data"));
    }

    #[test]
    fn test_clean_whois_removes_empty_lines() {
        let raw = "line1\n\n\nline2\n\nline3";
        let cleaned = WhoisService::clean_whois(raw.to_string());
        assert_eq!(cleaned, "line1\nline2\nline3");
    }

    #[test]
    fn test_clean_whois_all_comments() {
        let raw = "% all\n% comments\n# only";
        let cleaned = WhoisService::clean_whois(raw.to_string());
        assert!(cleaned.is_empty());
    }

    #[test]
    fn test_clean_whois_empty_input() {
        let cleaned = WhoisService::clean_whois(String::new());
        assert!(cleaned.is_empty());
    }

    #[test]
    fn test_clean_whois_preserves_data_lines() {
        let raw = "OrgName: Example Corp\n\
                   % RIR specific\n\
                   Address: 123 Main St\n\
                   # internal note\n\
                   Country: US";
        let cleaned = WhoisService::clean_whois(raw.to_string());
        assert_eq!(
            cleaned,
            "OrgName: Example Corp\nAddress: 123 Main St\nCountry: US"
        );
    }

    #[test]
    fn test_clean_whois_truncates_long_output() {
        let mut lines = Vec::new();
        for i in 0..30 {
            lines.push(format!("line {}", i));
        }
        let raw = lines.join("\n");
        let cleaned = WhoisService::clean_whois(raw);
        let result_lines: Vec<&str> = cleaned.lines().collect();
        assert!(result_lines.len() <= 15);
    }

    #[test]
    fn test_clean_whois_mixed_content() {
        let raw = "% header\n\
                   Domain: example.com\n\
                   # note\n\
                   \n\
                   Registrar: ExampleReg\n\
                   \n\
                   Status: active";
        let cleaned = WhoisService::clean_whois(raw.to_string());
        assert_eq!(
            cleaned,
            "Domain: example.com\nRegistrar: ExampleReg\nStatus: active"
        );
    }
}
