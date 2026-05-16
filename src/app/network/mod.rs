use serde::{Deserialize, Serialize};
use std::process::Command;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub protocol: String,
    pub local_address: String,
    pub local_port: u16,
    pub foreign_address: String,
    pub foreign_port: u16,
    pub state: String,
    pub pid: u32,
    pub location: Option<String>,
    pub isp: Option<String>,
}
#[derive(Debug)]
pub struct NetworkAnalyzer {
    connections: Vec<NetworkConnection>,
}
impl NetworkAnalyzer {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
        }
    }
    pub fn refresh_connections(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.connections.clear();
        #[cfg(target_os = "windows")]
        {
            self.parse_netstat_windows()?;
        }
        #[cfg(target_os = "linux")]
        {
            self.parse_netstat_linux()?;
        }
        Ok(())
    }
    #[cfg(target_os = "windows")]
    fn parse_netstat_windows(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("netstat").args(["-ano"]).output()?;
        if !output.status.success() {
            return Err("Failed to execute netstat".into());
        }
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("ESTABLISHED") {
                if let Some(conn) = self.parse_netstat_line_windows(line) {
                    self.connections.push(conn);
                }
            }
        }
        Ok(())
    }
    #[cfg(target_os = "linux")]
    fn parse_netstat_linux(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("ss").args(["-tunp"]).output();
        let (output_str, is_ss) = match output {
            Ok(o) if o.status.success() => {
                let text = String::from_utf8_lossy(&o.stdout).to_string();
                if text.lines().count() == 0 {
                    let fallback = Command::new("netstat").args(["-tunp"]).output()?;
                    if !fallback.status.success() {
                        return Err("Failed to execute ss or netstat".into());
                    }
                    (String::from_utf8_lossy(&fallback.stdout).to_string(), false)
                } else {
                    (text, true)
                }
            }
            _ => {
                let fallback = Command::new("netstat").args(["-tunp"]).output()?;
                if !fallback.status.success() {
                    return Err("Failed to execute ss or netstat".into());
                }
                (String::from_utf8_lossy(&fallback.stdout).to_string(), false)
            }
        };
        let valid_states = [
            "ESTAB",
            "TIME-WAIT",
            "TIME_WAIT",
            "CLOSE-WAIT",
            "CLOSE_WAIT",
            "FIN-WAIT",
            "FIN_WAIT",
            "LAST-ACK",
            "LAST_ACK",
            "CLOSING",
            "SYN-SENT",
            "SYN_RCVD",
        ];
        for line in output_str.lines() {
            if valid_states.iter().any(|s| line.contains(s)) {
                let conn = if is_ss {
                    self.parse_ss_line(line)
                } else {
                    self.parse_netstat_line_linux(line)
                };
                if let Some(conn) = conn {
                    self.connections.push(conn);
                }
            }
        }
        Ok(())
    }
    #[cfg(target_os = "linux")]
    fn parse_ss_line(&self, line: &str) -> Option<NetworkConnection> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            return None;
        }
        let protocol = if parts[0].contains("tcp") {
            "TCP"
        } else if parts[0].contains("udp") {
            "UDP"
        } else {
            parts[0]
        }
        .to_string();
        let state = parts[1].to_string();
        let local_addr = parts[4].to_string();
        let foreign_addr = parts[5].to_string();
        let (local_ip, local_port_str) = parse_addr_port(&local_addr)?;
        let (foreign_ip, foreign_port_str) = parse_addr_port(&foreign_addr)?;
        let local_port: u16 = local_port_str.parse().ok()?;
        let foreign_port: u16 = foreign_port_str.parse().ok()?;
        let pid = if parts.len() > 6 {
            extract_pid_ss(parts[6])
        } else {
            0
        };
        Some(NetworkConnection {
            protocol,
            local_address: local_ip,
            local_port,
            foreign_address: foreign_ip,
            foreign_port,
            state,
            pid,
            location: None,
            isp: None,
        })
    }
    #[cfg(target_os = "linux")]
    fn parse_netstat_line_linux(&self, line: &str) -> Option<NetworkConnection> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            return None;
        }
        let protocol = if parts[0].contains("tcp") {
            "TCP"
        } else if parts[0].contains("udp") {
            "UDP"
        } else {
            parts[0]
        }
        .to_string();
        let state = parts[5].to_string();
        let local_addr = parts[3].to_string();
        let foreign_addr = parts[4].to_string();
        let (local_ip, local_port_str) = parse_addr_port(&local_addr)?;
        let (foreign_ip, foreign_port_str) = parse_addr_port(&foreign_addr)?;
        let local_port: u16 = local_port_str.parse().ok()?;
        let foreign_port: u16 = foreign_port_str.parse().ok()?;
        let pid = if parts.len() > 6 {
            let last = parts[6];
            if let Some(pos) = last.find('/') {
                last[..pos].parse().ok()
            } else {
                last.parse().ok()
            }
        } else {
            None
        }
        .unwrap_or(0);
        Some(NetworkConnection {
            protocol,
            local_address: local_ip,
            local_port,
            foreign_address: foreign_ip,
            foreign_port,
            state,
            pid,
            location: None,
            isp: None,
        })
    }
    #[cfg(windows)]
    pub fn parse_netstat_line_windows(&self, line: &str) -> Option<NetworkConnection> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return None;
        }
        let protocol = parts[0].to_string();
        let local_addr = parts[1];
        let foreign_addr = parts[2];
        let state = parts[3];
        let pid_str = parts[4];
        let (local_ip, local_port_str) = local_addr.rsplit_once(':')?;
        let (foreign_ip, foreign_port_str) = foreign_addr.rsplit_once(':')?;
        let local_port = local_port_str.parse().ok()?;
        let foreign_port = foreign_port_str.parse().ok()?;
        let pid = pid_str.parse().ok()?;
        Some(NetworkConnection {
            protocol,
            local_address: local_ip.to_string(),
            local_port,
            foreign_address: foreign_ip.to_string(),
            foreign_port,
            state: state.to_string(),
            pid,
            location: None,
            isp: None,
        })
    }
    pub fn get_connections(&self) -> &[NetworkConnection] {
        &self.connections
    }
}
#[cfg(target_os = "linux")]
fn parse_addr_port(addr: &str) -> Option<(String, String)> {
    if addr.starts_with('[') {
        let end_bracket = addr.find("]:")?;
        let ip = addr[1..end_bracket].to_string();
        let port = addr[end_bracket + 2..].to_string();
        Some((ip, port))
    } else {
        let (ip, port) = addr.rsplit_once(':')?;
        Some((ip.to_string(), port.to_string()))
    }
}
pub fn has_ss() -> bool {
    std::process::Command::new("sh")
        .args(["-c", "command -v ss"])
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
pub fn has_netstat() -> bool {
    std::process::Command::new("sh")
        .args(["-c", "command -v netstat"])
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
#[cfg(target_os = "linux")]
fn extract_pid_ss(field: &str) -> u32 {
    if let Some(pid_part) = field.split("pid=").nth(1) {
        let clean = pid_part.trim_end_matches(')').trim_end_matches(',');
        if let Some(comma) = clean.find(',') {
            clean[..comma].parse().unwrap_or(0)
        } else {
            clean.parse().unwrap_or(0)
        }
    } else {
        0
    }
}
