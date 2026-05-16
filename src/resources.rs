use once_cell::sync::Lazy;
use serde_json::Value;

pub struct ExternalUrls {
    pub github_api_releases: String,
    pub github_releases_page: String,
    pub google_search_url: String,
    pub nerd_font_repo_url: String,
    pub geoip_api_base: String,
    pub geoip_api_batch: String,
    pub user_agent: String,
}

pub static URLS: Lazy<ExternalUrls> = Lazy::new(|| {
    let json: Value = serde_json::from_str(include_str!("../resources/external_urls.json"))
        .expect("Failed to parse external_urls.json");
    ExternalUrls {
        github_api_releases: get_str(&json, "github_api_releases"),
        github_releases_page: get_str(&json, "github_releases_page"),
        google_search_url: get_str(&json, "google_search_url"),
        nerd_font_repo_url: get_str(&json, "nerd_font_repo_url"),
        geoip_api_base: get_str(&json, "geoip_api_base"),
        geoip_api_batch: get_str(&json, "geoip_api_batch"),
        user_agent: get_str(&json, "user_agent"),
    }
});

fn get_str(json: &Value, key: &str) -> String {
    json[key]
        .as_str()
        .unwrap_or_else(|| panic!("Missing or invalid key in external_urls.json: {}", key))
        .to_string()
}
