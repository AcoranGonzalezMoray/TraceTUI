use crate::app::agents::constants::{
    NO_DEPENDENCY_CONTEXT_PLACEHOLDER, NO_NETWORK_DATA_ERROR, NO_PROCESS_DATA_ERROR,
    NO_PROCESS_NAME_PLACEHOLDER,
};
use crate::app::network::NetworkConnection;
use crate::app::process::ProcessInfo;
use crate::app::types::AgentMission;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionDataKind {
    Process,
    Network,
}

impl MissionDataKind {
    pub fn requires(self, has_processes: bool, has_networks: bool) -> Result<(), String> {
        match self {
            Self::Process if !has_processes => Err(NO_PROCESS_DATA_ERROR.to_string()),
            Self::Network if !has_networks => Err(NO_NETWORK_DATA_ERROR.to_string()),
            _ => Ok(()),
        }
    }
}

impl AgentMission {
    pub fn title(self) -> &'static str {
        match self {
            Self::ProcessAnalysis => "Process Analysis",
            Self::NetworkAnalysis => "Network Analysis",
            Self::DnsAnalysis => "DNS Analysis",
            Self::FileAnalyzer => "File Analyzer",
            Self::PortScanner => "Port Scanner",
            Self::LogAnalyzer => "Log Analyzer",
            Self::MemoryAnalyzer => "Memory Analyzer",
            Self::VulnerabilityCheck => "Vulnerability Check",
            Self::ThreatIntel => "Threat Intel",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::ProcessAnalysis => "Process",
            Self::NetworkAnalysis => "Network",
            Self::DnsAnalysis => "DNS",
            Self::FileAnalyzer => "Files",
            Self::PortScanner => "Ports",
            Self::LogAnalyzer => "Logs",
            Self::MemoryAnalyzer => "Memory",
            Self::VulnerabilityCheck => "CVE",
            Self::ThreatIntel => "Intel",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::ProcessAnalysis => "\u{F01A7}",
            Self::NetworkAnalysis => "\u{F06F3}",
            Self::DnsAnalysis => "\u{F059F}",
            Self::FileAnalyzer => "\u{F0219}",
            Self::PortScanner => "\u{F04FE}",
            Self::LogAnalyzer => "\u{F0331}",
            Self::MemoryAnalyzer => "\u{F035B}",
            Self::VulnerabilityCheck => "\u{F0483}",
            Self::ThreatIntel => "\u{F0CE6}",
        }
    }

    pub fn focus(self) -> &'static str {
        match self {
            Self::ProcessAnalysis => "legitimacy, path, command line, resource usage, suspicious naming, recommended actions",
            Self::NetworkAnalysis => "remote IPs, ports, geo/ISP anomalies, connection states, suspicious network patterns",
            Self::DnsAnalysis => "DNS-like destinations, suspicious domains, DGA indicators, C2 patterns, DNS tunneling hints",
            Self::FileAnalyzer => "binary/script paths, file names, extensions, signature/hash-check recommendations, malicious file indicators",
            Self::PortScanner => "open/local and remote ports per process, unexpected services, exposed listeners, backdoor indicators",
            Self::LogAnalyzer => "recent system-event correlation ideas, crash/error patterns, suspicious timestamps and follow-up logs to inspect",
            Self::MemoryAnalyzer => "memory consumption, anomalous allocation patterns, injection/leak indicators, process behavior outliers",
            Self::VulnerabilityCheck => "software/version clues, likely CVE exposure, exploitability, patching priority; use upstream facts only when present in input",
            Self::ThreatIntel => "IP/domain reputation enrichment, suspicious ASN/hosting patterns, C2 infrastructure indicators",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::ProcessAnalysis => "process_analysis",
            Self::NetworkAnalysis => "network_analysis",
            Self::DnsAnalysis => "dns_analysis",
            Self::FileAnalyzer => "file_analyzer",
            Self::PortScanner => "port_scanner",
            Self::LogAnalyzer => "log_analyzer",
            Self::MemoryAnalyzer => "memory_analyzer",
            Self::VulnerabilityCheck => "vulnerability_check",
            Self::ThreatIntel => "threat_intel",
        }
    }

    pub fn data_kind(self) -> MissionDataKind {
        match self {
            Self::ProcessAnalysis
            | Self::FileAnalyzer
            | Self::MemoryAnalyzer
            | Self::LogAnalyzer => MissionDataKind::Process,
            Self::NetworkAnalysis
            | Self::DnsAnalysis
            | Self::PortScanner
            | Self::VulnerabilityCheck
            | Self::ThreatIntel => MissionDataKind::Network,
        }
    }
}

pub fn all_missions() -> &'static [AgentMission] {
    &[
        AgentMission::ProcessAnalysis,
        AgentMission::NetworkAnalysis,
        AgentMission::DnsAnalysis,
        AgentMission::FileAnalyzer,
        AgentMission::PortScanner,
        AgentMission::LogAnalyzer,
        AgentMission::MemoryAnalyzer,
        AgentMission::VulnerabilityCheck,
        AgentMission::ThreatIntel,
    ]
}

pub fn mission_history_lookup() -> &'static [(&'static str, AgentMission)] {
    &[
        ("process_analysis", AgentMission::ProcessAnalysis),
        ("network_analysis", AgentMission::NetworkAnalysis),
        ("dns_analysis", AgentMission::DnsAnalysis),
        ("file_analyzer", AgentMission::FileAnalyzer),
        ("port_scanner", AgentMission::PortScanner),
        ("log_analyzer", AgentMission::LogAnalyzer),
        ("memory_analyzer", AgentMission::MemoryAnalyzer),
        ("vulnerability_check", AgentMission::VulnerabilityCheck),
        ("threat_intel", AgentMission::ThreatIntel),
    ]
}

pub fn mission_filename_slug(mission: AgentMission) -> String {
    mission.title().replace(' ', "_").to_lowercase()
}

pub fn missing_process_name_placeholder() -> &'static str {
    NO_PROCESS_NAME_PLACEHOLDER
}

pub fn missing_dependency_context_placeholder() -> &'static str {
    NO_DEPENDENCY_CONTEXT_PLACEHOLDER
}

pub fn serialize_processes(processes: &[ProcessInfo]) -> String {
    serde_json::to_string_pretty(processes).unwrap_or_default()
}

pub fn serialize_connections(connections: &[NetworkConnection]) -> String {
    serde_json::to_string_pretty(connections).unwrap_or_default()
}
