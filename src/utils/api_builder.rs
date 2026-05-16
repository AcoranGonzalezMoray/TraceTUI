use crate::resources;
use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct ApiUrlBuilder {
    base_url: String,
    path: String,
    params: HashMap<String, String>,
}
impl ApiUrlBuilder {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            path: String::new(),
            params: HashMap::new(),
        }
    }
    pub fn with_path(mut self, path: &str) -> Self {
        let clean_path = path.trim_start_matches('/');
        self.path = format!("/{}", clean_path);
        self
    }
    pub fn with_fields(mut self, fields: &[&str]) -> Self {
        let fields_str = fields.join(",");
        self.params.insert("fields".to_string(), fields_str);
        self
    }
    pub fn build(self) -> String {
        let mut url = format!("{}{}", self.base_url, self.path);
        if !self.params.is_empty() {
            url.push('?');
            let param_strings: Vec<String> = self
                .params
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect();
            url.push_str(&param_strings.join("&"));
        }
        url
    }
}
pub const GEOIP_FIELDS: &[&str] = &[
    "status",
    "message",
    "country",
    "countryCode",
    "regionName",
    "city",
    "zip",
    "isp",
    "org",
    "lat",
    "lon",
    "timezone",
    "as",
    "query",
    "mobile",
    "proxy",
    "hosting",
];

#[derive(Debug, Clone)]
pub struct GeoIpApiBuilder;
impl GeoIpApiBuilder {
    pub fn geo_builder() -> ApiUrlBuilder {
        ApiUrlBuilder::new(&resources::URLS.geoip_api_base).with_fields(GEOIP_FIELDS)
    }
    pub fn lookup_ip(ip: &str) -> ApiUrlBuilder {
        Self::geo_builder().with_path(ip)
    }
}
