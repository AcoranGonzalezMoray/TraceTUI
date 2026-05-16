#[cfg(test)]
mod translator_tests {
    use crate::i18n::Translator;

    #[test]
    fn test_new_en_translator() {
        let t = Translator::new("en");
        assert_eq!(t.locale, "en");
    }

    #[test]
    fn test_new_es_translator() {
        let t = Translator::new("es");
        assert_eq!(t.locale, "es");
    }

    #[test]
    fn test_new_unknown_locale_falls_back_to_en() {
        let t = Translator::new("xx");
        assert_eq!(t.locale, "en");
    }

    #[test]
    fn test_get_known_key() {
        let t = Translator::new("en");
        assert_eq!(t.get("app.title"), "TRACE");
    }

    #[test]
    fn test_get_unknown_key_returns_key() {
        let t = Translator::new("en");
        assert_eq!(t.get("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn test_get_fmt_with_args() {
        let t = Translator::new("en");
        let result = t.get_fmt(
            "status.analysis_complete",
            &["5".to_string(), "2".to_string()],
        );
        assert!(!result.is_empty());
        assert!(result.contains("5"));
    }

    #[test]
    fn test_get_fmt_no_args() {
        let t = Translator::new("en");
        let result = t.get_fmt("app.title", &[]);
        assert_eq!(result, "TRACE");
    }

    #[test]
    fn test_available_locales_contains_all() {
        let locales = Translator::available_locales();
        let codes: Vec<&str> = locales.iter().map(|(c, _)| *c).collect();
        assert!(codes.contains(&"en"));
        assert!(codes.contains(&"es"));
        assert!(codes.contains(&"pt"));
        assert!(codes.contains(&"fr"));
        assert!(codes.contains(&"de"));
        assert!(codes.contains(&"it"));
        assert!(codes.contains(&"ja"));
        assert!(codes.contains(&"zh"));
        assert!(codes.contains(&"ru"));
    }

    #[test]
    fn test_available_locales_count() {
        assert_eq!(Translator::available_locales().len(), 9);
    }

    #[test]
    fn test_translator_debug() {
        let t = Translator::new("en");
        let debug = format!("{:?}", t);
        assert!(debug.contains("en"));
    }

    #[test]
    fn test_translator_clone() {
        let t = Translator::new("fr");
        let cloned = t.clone();
        assert_eq!(cloned.locale, "fr");
        assert_eq!(cloned.get("app.title"), "TRACE");
    }

    #[test]
    fn test_get_fmt_multiple_args() {
        let t = Translator::new("en");
        let result = t.get_fmt(
            "status.analysis_complete",
            &["10".to_string(), "3".to_string()],
        );
        assert!(result.contains("10"));
    }

    #[test]
    fn test_get_fmt_more_args_than_placeholders() {
        let t = Translator::new("en");
        let result = t.get_fmt("app.title", &["extra1".to_string(), "extra2".to_string()]);
        assert_eq!(result, "TRACE");
    }
}
