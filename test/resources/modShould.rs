#[cfg(test)]
mod resources_tests {
    #[test]
    fn test_external_urls_loaded() {
        let urls = &crate::resources::URLS;
        assert!(!urls.github_api_releases.is_empty());
        assert!(!urls.github_releases_page.is_empty());
        assert!(!urls.google_search_url.is_empty());
        assert!(!urls.user_agent.is_empty());
    }

    #[test]
    fn test_github_api_url() {
        assert_eq!(
            crate::resources::URLS.github_api_releases,
            "https://api.github.com/repos/AcoranGonzalezMoray/TraceTUI/releases/latest"
        );
    }

    #[test]
    fn test_github_releases_page() {
        assert_eq!(
            crate::resources::URLS.github_releases_page,
            "https://github.com/AcoranGonzalezMoray/TraceTUI/releases/latest"
        );
    }

    #[test]
    fn test_google_search_url() {
        assert_eq!(
            crate::resources::URLS.google_search_url,
            "https://www.google.com/search?q="
        );
    }

    #[test]
    fn test_user_agent() {
        assert_eq!(crate::resources::URLS.user_agent, "tracetui");
    }

    #[test]
    fn test_nerd_font_repo_url() {
        assert!(crate::resources::URLS
            .nerd_font_repo_url
            .contains("github.com"));
        assert!(crate::resources::URLS
            .nerd_font_repo_url
            .contains("JetBrainsMono.zip"));
    }

    #[test]
    fn test_geoip_api_base() {
        assert_eq!(
            crate::resources::URLS.geoip_api_base,
            "http://ip-api.com/json"
        );
    }
}
