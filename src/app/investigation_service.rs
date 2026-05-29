use crate::config;
use crate::services::geoip_service::GeoIpService;
use std::process::Command;
use tokio::sync::mpsc;
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InvestigationReport {
    pub ip: String,
    pub port: u16,
    pub domain: Option<String>,
    pub organization: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub zip: Option<String>,
    pub isp: Option<String>,
    pub as_info: Option<String>,
    pub timezone: Option<String>,
    pub mobile: Option<bool>,
    pub proxy: Option<bool>,
    pub hosting: Option<bool>,
    pub ping_ms: Option<String>,
    pub hops: Vec<String>,
    pub hop_coords: Vec<(f64, f64)>,
    pub risk_score: u8,
    pub risk_factors: Vec<String>,
    pub lat: f64,
    pub lon: f64,
    pub whois_data: Option<String>,
}
pub struct InvestigationService {
    geo_service: GeoIpService,
    process_name: String,
}
impl InvestigationService {
    pub fn new(geo_service: GeoIpService, process_name: String) -> Self {
        Self {
            geo_service,
            process_name,
        }
    }
    pub async fn investigate(
        &self,
        ip: String,
        port: u16,
        tx: mpsc::UnboundedSender<InvestigationReport>,
    ) {
        let report = self.build_investigation_report(ip.clone(), port).await;
        let _ = tx.send(report);
    }
    async fn build_investigation_report(&self, ip: String, port: u16) -> InvestigationReport {
        let mut report = InvestigationReport::new(ip.clone(), port);
        self.lookup_geographic_info(&mut report).await;
        self.perform_nslookup(&mut report);
        self.perform_ping_test(&mut report);
        self.perform_traceroute(&mut report, &ip).await;
        self.perform_whois_lookup(&mut report, &ip).await;
        self.calculate_risk_score(&mut report);
        report
    }
    async fn lookup_geographic_info(&self, report: &mut InvestigationReport) {
        if let Ok(Some(info)) = self.geo_service.lookup(&report.ip).await {
            report.organization = Some(info.org);
            report.city = Some(info.city);
            report.region = info.regionName;
            report.country = Some(info.country);
            report.country_code = Some(info.countryCode);
            report.zip = info.zip;
            report.isp = Some(info.isp);
            report.as_info = info.as_info;
            report.timezone = info.timezone;
            report.mobile = info.mobile;
            report.proxy = info.proxy;
            report.hosting = info.hosting;
            report.lat = info.lat;
            report.lon = info.lon;
        }
    }
    fn perform_nslookup(&self, report: &mut InvestigationReport) {
        let output = if cfg!(windows) {
            Command::new("nslookup").arg(&report.ip).output()
        } else {
            let dig = Command::new("dig")
                .args(["+short", "+noall", "+answer", &report.ip])
                .output();
            if let Ok(ref out) = dig {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    if !stdout.trim().is_empty() {
                        report.domain = Some(stdout.trim().to_string());
                        return;
                    }
                }
            }
            Command::new("nslookup").arg(&report.ip).output()
        };
        if let Ok(out) = output {
            let out_str = String::from_utf8_lossy(&out.stdout);
            for line in out_str.lines() {
                if line.contains("Name:") {
                    report.domain = Some(line.split(':').nth(1).unwrap_or("").trim().to_string());
                }
            }
        }
    }
    fn perform_ping_test(&self, report: &mut InvestigationReport) {
        let ping = if cfg!(windows) {
            Command::new("ping").args(["-n", "1", &report.ip]).output()
        } else {
            Command::new("ping").args(["-c", "1", &report.ip]).output()
        };
        if let Ok(out) = ping {
            let out_str = String::from_utf8_lossy(&out.stdout);
            if let Some(time_idx) = out_str.find("time=") {
                let part = &out_str[time_idx + 5..];
                if let Some(end_idx) = part.find("ms") {
                    report.ping_ms = Some(part[..end_idx].trim().to_string());
                }
            } else if out_str.find("bytes from").is_some() {
                if let Some(rtt_idx) = out_str.find("ttl=") {
                    let part = &out_str[rtt_idx..];
                    if let Some(time_start) = part.find("time=") {
                        let time_part = &part[time_start + 5..];
                        if let Some(end) = time_part.find(" ") {
                            report.ping_ms = Some(time_part[..end].trim().to_string());
                        }
                    }
                }
            }
        }
    }
    async fn perform_traceroute(&self, report: &mut InvestigationReport, ip: &str) {
        let trace = if cfg!(windows) {
            Command::new("tracert").args(["-h", "5", "-d", ip]).output()
        } else {
            Command::new("traceroute")
                .args(["-m", "5", "-n", ip])
                .output()
        };
        if let Ok(out) = trace {
            let out_str = String::from_utf8_lossy(&out.stdout);
            for line in out_str.lines() {
                if line.contains(".") || line.contains("ms") {
                    let cleaned = line.replace("*", "").trim().to_string();
                    if !cleaned.is_empty()
                        && !cleaned.contains("traceroute")
                        && !cleaned.contains("tracing")
                    {
                        report.hops.push(cleaned.clone());
                        self.extract_hop_coordinates(report, line).await;
                    }
                }
            }
        }
    }
    async fn extract_hop_coordinates(&self, report: &mut InvestigationReport, line: &str) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(last) = parts.last() {
            if last.contains('.')
                && last.chars().all(|c| c.is_ascii_digit() || c == '.')
                && !last.starts_with("192.168.")
                && !last.starts_with("10.")
                && !last.starts_with("127.")
            {
                if let Ok(Some(hop_info)) = self.geo_service.lookup(last).await {
                    if hop_info.lat != 0.0 {
                        report.hop_coords.push((hop_info.lat, hop_info.lon));
                    }
                }
            }
        }
    }
    async fn perform_whois_lookup(&self, report: &mut InvestigationReport, ip: &str) {
        if let Some(w) = crate::utils::whois::WhoisService::lookup(ip).await {
            report.whois_data = Some(w);
        }
    }
    fn calculate_risk_score(&self, report: &mut InvestigationReport) {
        let mut score: u8 = config::INV_RISK_BASE;
        self.evaluate_domain_process_mismatch(&mut score, report);
        self.evaluate_network_anonymity_risk(&mut score, report);
        self.evaluate_latency_risk(&mut score, report);
        report.risk_score = score.min(100);
    }
    fn evaluate_domain_process_mismatch(&self, score: &mut u8, report: &mut InvestigationReport) {
        if let Some(domain) = &report.domain {
            let dom_lower = domain.to_lowercase();
            let proc_lower = self.process_name.to_lowercase();
            if !dom_lower.contains(&proc_lower)
                && !proc_lower.contains("system")
                && !proc_lower.contains("svchost")
                && !config::DOMAIN_ALLOWLIST
                    .iter()
                    .any(|&d| dom_lower.contains(d))
            {
                *score += config::INV_RISK_DOMAIN_MISMATCH;
                report
                    .risk_factors
                    .push("Domain/Process mismatch".to_string());
            }
        } else {
            *score += config::INV_RISK_NO_REVERSE_DNS;
            report
                .risk_factors
                .push("Hidden Identity (No Reverse DNS)".to_string());
        }
    }
    fn evaluate_network_anonymity_risk(&self, score: &mut u8, report: &mut InvestigationReport) {
        if report.proxy == Some(true) {
            *score += config::INV_RISK_PROXY;
            report
                .risk_factors
                .push(config::RISK_FACTOR_PROXY.to_string());
        }
        if report.hosting == Some(true) {
            *score += config::INV_RISK_HOSTING;
            report
                .risk_factors
                .push(config::RISK_FACTOR_HOSTING.to_string());
        }
        if report.mobile == Some(true) {
            *score += config::INV_RISK_MOBILE;
            report
                .risk_factors
                .push(config::RISK_FACTOR_MOBILE.to_string());
        }
    }
    fn evaluate_latency_risk(&self, score: &mut u8, report: &mut InvestigationReport) {
        if let Some(ms) = &report.ping_ms {
            if let Ok(ping_val) = ms.parse::<u32>() {
                if ping_val > config::INV_RISK_LATENCY_THRESHOLD_MS {
                    *score += config::INV_RISK_HIGH_LATENCY;
                    report
                        .risk_factors
                        .push("High Latency (Potential proxy/VPN)".to_string());
                }
            }
        }
    }
}
impl InvestigationReport {
    pub fn new(ip: String, port: u16) -> Self {
        Self {
            ip,
            port,
            ..Default::default()
        }
    }
}

impl Default for InvestigationReport {
    fn default() -> Self {
        Self {
            ip: String::new(),
            port: 0,
            domain: None,
            organization: None,
            city: None,
            region: None,
            country: None,
            country_code: None,
            zip: None,
            isp: None,
            as_info: None,
            timezone: None,
            mobile: None,
            proxy: None,
            hosting: None,
            ping_ms: None,
            hops: Vec::new(),
            hop_coords: Vec::new(),
            risk_score: 0,
            risk_factors: Vec::new(),
            lat: 0.0,
            lon: 0.0,
            whois_data: None,
        }
    }
}
