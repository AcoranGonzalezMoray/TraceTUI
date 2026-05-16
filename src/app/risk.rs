use crate::app::network::NetworkConnection;
use crate::app::process::ProcessInfo;
use crate::config;
pub struct RiskAnalyzer;
impl RiskAnalyzer {
    pub fn calculate(process: &ProcessInfo, connections: &[NetworkConnection]) -> String {
        let conn_count = connections.iter().filter(|c| c.pid == process.pid).count();
        let mut risk_score = 0u8;
        if conn_count > config::RISK_CONN_HIGH {
            risk_score += 3;
        } else if conn_count > config::RISK_CONN_MEDIUM {
            risk_score += 2;
        } else if conn_count > config::RISK_CONN_LOW {
            risk_score += 1;
        }
        if process.cpu_usage > config::RISK_CPU_THRESHOLD {
            risk_score += 1;
        }
        if process.memory_usage > config::RISK_MEM_HIGH {
            risk_score += 2;
        } else if process.memory_usage > config::RISK_MEM_MEDIUM {
            risk_score += 1;
        }
        let name_lower = process.name.to_lowercase();
        if config::SUSPICIOUS_PROCESS_NAMES
            .iter()
            .any(|&s| name_lower.contains(s))
        {
            risk_score += 2;
        }
        if process.path.is_none() || process.path.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
            risk_score += 1;
        }
        match risk_score {
            0..=1 => "LOW",
            2..=3 => "MEDIUM",
            4..=5 => "HIGH",
            _ => "CRITICAL",
        }
        .to_string()
    }
    pub fn is_high_or_critical(risk: &str) -> bool {
        risk.contains("HIGH") || risk.contains("CRITICAL")
    }
}
