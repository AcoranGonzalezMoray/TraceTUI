use crate::config;
use thiserror::Error;
#[derive(Debug, Clone)]
pub struct ApiClient {
    client: reqwest::Client,
}
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("JSON deserialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
}
impl ApiClient {
    pub fn new() -> Result<Self, ApiError> {
        let client = reqwest::Client::builder()
            .timeout(config::http_timeout())
            .user_agent("TraceTUI/1.0")
            .build()
            .map_err(ApiError::Request)?;
        Ok(Self { client })
    }
    pub async fn get<T>(&self, url: &str) -> Result<T, ApiError>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self.client.get(url).send().await?;
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ApiError::RateLimit("API rate limit exceeded".to_string()));
        }
        let text = response.text().await?;
        serde_json::from_str(&text).map_err(ApiError::Json)
    }

    pub async fn post<T, B>(&self, url: &str, body: &B) -> Result<T, ApiError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let response = self.client.post(url).json(body).send().await?;
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ApiError::RateLimit("API rate limit exceeded".to_string()));
        }
        let text = response.text().await?;
        serde_json::from_str(&text).map_err(ApiError::Json)
    }
}
