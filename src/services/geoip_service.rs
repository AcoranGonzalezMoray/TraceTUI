use crate::config;
use crate::resources;
use crate::services::api_client::{ApiClient, ApiError};
use crate::utils::api_builder::GeoIpApiBuilder;
use crate::utils::rate_limiter::RateLimiter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct GeoInfo {
    pub status: String,
    pub country: String,
    pub countryCode: String,
    pub city: String,
    pub regionName: Option<String>,
    pub zip: Option<String>,
    pub isp: String,
    pub org: String,
    pub lat: f64,
    pub lon: f64,
    pub timezone: Option<String>,
    #[serde(rename = "as")]
    pub as_info: Option<String>,
    pub query: Option<String>,
    pub mobile: Option<bool>,
    pub proxy: Option<bool>,
    pub hosting: Option<bool>,
}

#[derive(Debug, Serialize)]
struct BatchItem {
    query: String,
}

#[derive(Debug, Clone)]
pub struct GeoIpService {
    cache: Arc<RwLock<HashMap<String, GeoInfo>>>,
    rate_limiter: RateLimiter,
    api_client: ApiClient,
}
impl GeoIpService {
    pub fn new() -> Result<Self, ApiError> {
        Ok(Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: RateLimiter::new(
                config::GEOIP_RATE_LIMIT,
                config::GEOIP_RATE_WINDOW_SECS,
            ),
            api_client: ApiClient::new()?,
        })
    }
    pub async fn lookup(&self, ip: &str) -> Result<Option<GeoInfo>, ApiError> {
        if Self::is_private_ip(ip) {
            return Ok(None);
        }
        {
            let cache = self.cache.read().await;
            if let Some(info) = cache.get(ip) {
                return Ok(Some(info.clone()));
            }
        }
        if let Err(e) = self.rate_limiter.acquire_request().await {
            return Err(ApiError::RateLimit(format!("Rate limit: {}", e)));
        }
        let url = GeoIpApiBuilder::lookup_ip(ip).build();
        let result = match self.api_client.get::<GeoInfo>(&url).await {
            Ok(info) => {
                if info.status == "success" {
                    let mut cache = self.cache.write().await;
                    cache.insert(ip.to_string(), info.clone());
                    Ok(Some(info))
                } else {
                    Ok(None)
                }
            }
            Err(e) => Err(e),
        };
        self.rate_limiter.release_request();
        result
    }

    pub async fn lookup_batch(&self, ips: &[String]) -> HashMap<String, GeoInfo> {
        let mut results = HashMap::new();
        let mut to_fetch: Vec<String> = Vec::new();

        {
            let cache = self.cache.read().await;
            for ip in ips {
                if Self::is_private_ip(ip) {
                    continue;
                }
                if let Some(info) = cache.get(ip) {
                    results.insert(ip.clone(), info.clone());
                } else {
                    to_fetch.push(ip.clone());
                }
            }
        }

        if to_fetch.is_empty() {
            return results;
        }

        if self.rate_limiter.acquire_request().await.is_err() {
            return results;
        }

        let url = &resources::URLS.geoip_api_batch;
        let batch_body: Vec<BatchItem> = to_fetch
            .iter()
            .map(|ip| BatchItem { query: ip.clone() })
            .collect();

        if let Ok(infos) = self
            .api_client
            .post::<Vec<GeoInfo>, Vec<BatchItem>>(url, &batch_body)
            .await
        {
            let mut cache = self.cache.write().await;
            for info in infos {
                if info.status == "success" {
                    let query_ip = info.query.clone().unwrap_or_default();
                    if !query_ip.is_empty() {
                        cache.insert(query_ip.clone(), info.clone());
                        results.insert(query_ip, info);
                    }
                }
            }
        }

        self.rate_limiter.release_request();
        results
    }
    pub fn is_private_ip(ip: &str) -> bool {
        ip.is_empty()
            || ip == "127.0.0.1"
            || ip == "0.0.0.0"
            || ip == "::1"
            || ip.starts_with("192.168.")
            || ip.starts_with("10.")
            || ip.starts_with("172.16.")
            || ip.starts_with("169.254.")
            || ip.starts_with("fc00:")
            || ip.starts_with("fe80:")
    }
    pub fn get_flag_emoji(country_code: &str) -> String {
        if country_code.len() != 2 {
            return "\u{f0320}".to_string();
        }
        let mut flag = String::new();
        for c in country_code.to_uppercase().chars() {
            let cp = 0x1F1E6 + (c as u32 - 'A' as u32);
            if let Some(ch) = std::char::from_u32(cp) {
                flag.push(ch);
            }
        }
        flag
    }
}
