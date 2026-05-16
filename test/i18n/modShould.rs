#[cfg(test)]
mod i18n_mod_tests {
    use crate::i18n;

    #[test]
    fn test_detect_system_locale_returns_string() {
        let locale = i18n::detect_system_locale();
        assert!(!locale.is_empty());
        assert!(locale.len() <= 5);
    }

    #[test]
    fn test_detect_system_locale_known_prefixes() {
        let valid = ["en", "es", "pt", "fr", "de", "it", "ja", "zh", "ru"];
        let locale = i18n::detect_system_locale();
        assert!(
            valid.contains(&locale.as_str()),
            "Unknown locale: {}",
            locale
        );
    }
}
