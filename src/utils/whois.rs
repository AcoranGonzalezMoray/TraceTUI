use crate::config;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
pub struct WhoisService;
impl WhoisService {
    const IANA_ROOT: &'static str = "whois.iana.org";
    pub async fn lookup(ip: &str) -> Option<String> {
        if let Ok(rir_server) = Self::query_server(Self::IANA_ROOT, ip).await {
            if let Some(refer) = rir_server.lines().find(|l| l.contains("refer:")) {
                let next_server = refer.split(':').nth(1)?.trim();
                if let Ok(final_data) = Self::query_server(next_server, ip).await {
                    return Some(Self::clean_whois(final_data));
                }
            }
            return Some(Self::clean_whois(rir_server));
        }
        None
    }
    async fn query_server(
        server: &str,
        query: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut stream = tokio::time::timeout(
            Duration::from_secs(config::WHOIS_TIMEOUT_SECS),
            TcpStream::connect(format!("{}:43", server)),
        )
        .await??;
        stream
            .write_all(format!("{}\r\n", query).as_bytes())
            .await?;
        let mut response = String::new();
        stream.read_to_string(&mut response).await?;
        Ok(response)
    }
    pub fn clean_whois(raw: String) -> String {
        raw.lines()
            .filter(|l| !l.starts_with('%') && !l.starts_with('#') && !l.is_empty())
            .take(config::WHOIS_MAX_LINES)
            .collect::<Vec<&str>>()
            .join("\n")
    }
}
