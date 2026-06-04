use crate::app::network::NetworkConnection;
use crate::app::process::ProcessInfo;
use crate::app::types::{
    AgentInstance, AgentMission, AgentProvider, AgentProviderConfig, AgentStatus,
};
use futures_util::StreamExt;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::mpsc;

fn process_json(processes: &[ProcessInfo]) -> String {
    serde_json::to_string_pretty(processes).unwrap_or_default()
}

fn network_json(connections: &[NetworkConnection]) -> String {
    serde_json::to_string_pretty(connections).unwrap_or_default()
}

fn mission_title(mission: AgentMission) -> &'static str {
    match mission {
        AgentMission::ProcessAnalysis => "Process Analysis",
        AgentMission::NetworkAnalysis => "Network Analysis",
        AgentMission::DnsAnalysis => "DNS Analysis",
        AgentMission::FileAnalyzer => "File Analyzer",
        AgentMission::PortScanner => "Port Scanner",
        AgentMission::LogAnalyzer => "Log Analyzer",
        AgentMission::MemoryAnalyzer => "Memory Analyzer",
        AgentMission::VulnerabilityCheck => "Vulnerability Check",
        AgentMission::ThreatIntel => "Threat Intel",
    }
}

fn mission_focus(mission: AgentMission) -> &'static str {
    match mission {
        AgentMission::ProcessAnalysis => "legitimacy, path, command line, resource usage, suspicious naming, recommended actions",
        AgentMission::NetworkAnalysis => "remote IPs, ports, geo/ISP anomalies, connection states, suspicious network patterns",
        AgentMission::DnsAnalysis => "DNS-like destinations, suspicious domains, DGA indicators, C2 patterns, DNS tunneling hints",
        AgentMission::FileAnalyzer => "binary/script paths, file names, extensions, signature/hash-check recommendations, malicious file indicators",
        AgentMission::PortScanner => "open/local and remote ports per process, unexpected services, exposed listeners, backdoor indicators",
        AgentMission::LogAnalyzer => "recent system-event correlation ideas, crash/error patterns, suspicious timestamps and follow-up logs to inspect",
        AgentMission::MemoryAnalyzer => "memory consumption, anomalous allocation patterns, injection/leak indicators, process behavior outliers",
        AgentMission::VulnerabilityCheck => "software/version clues, likely CVE exposure, exploitability, patching priority; use upstream facts only when present in input",
        AgentMission::ThreatIntel => "IP/domain reputation enrichment, suspicious ASN/hosting patterns, C2 infrastructure indicators",
    }
}

fn build_prompt(
    mission: AgentMission,
    processes: Option<&[ProcessInfo]>,
    networks: Option<(&[NetworkConnection], &str)>,
    dependency_context: Option<&str>,
    locale: &str,
) -> Result<String, String> {
    let proc_text = processes
        .map(process_json)
        .unwrap_or_else(|| "[]".to_string());
    let (net_text, process_name) = networks
        .map(|(connections, name)| (network_json(connections), name.to_string()))
        .unwrap_or_else(|| ("[]".to_string(), "N/A".to_string()));

    if matches!(
        mission,
        AgentMission::ProcessAnalysis
            | AgentMission::FileAnalyzer
            | AgentMission::MemoryAnalyzer
            | AgentMission::LogAnalyzer
    ) && processes.map(|p| p.is_empty()).unwrap_or(true)
    {
        return Err("No process data available".to_string());
    }

    if matches!(
        mission,
        AgentMission::NetworkAnalysis
            | AgentMission::DnsAnalysis
            | AgentMission::PortScanner
            | AgentMission::VulnerabilityCheck
            | AgentMission::ThreatIntel
    ) && networks.map(|(n, _)| n.is_empty()).unwrap_or(true)
    {
        return Err("No network data available".to_string());
    }

    Ok(format!(
        r#"You are a senior incident-response agent running the "{title}" mission.

Locale: {locale}
Target process/group: {process_name}

Process data:
```json
{proc_text}
```

Network data:
```json
{net_text}
```

Dependency/context from previous agents:
{dependency_context}

Analyze: {focus}

Output rules:
- Respond in Markdown using clear headings, bullets, tables where useful, and a final verdict.
- Keep it actionable and concise.
- Call out uncertainty explicitly. Do not invent live threat-intel/CVE results that are not in the input.
- Include risk as LOW/MEDIUM/HIGH/CRITICAL and recommended next steps.
- Respond in the language corresponding to locale code "{locale}"."#,
        title = mission_title(mission),
        locale = locale,
        process_name = process_name,
        proc_text = proc_text,
        net_text = net_text,
        dependency_context = dependency_context.unwrap_or("None"),
        focus = mission_focus(mission),
    ))
}

fn ollama_payload(model: &str, prompt: &str, stream: bool) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": stream,
        "options": { "temperature": 0.3, "num_predict": 4096 }
    })
}

fn openai_payload(model: &str, prompt: &str, stream: bool) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "stream": stream,
        "messages": [
            { "role": "system", "content": "You are a precise security analysis agent." },
            { "role": "user", "content": prompt }
        ],
        "temperature": 0.3,
        "max_tokens": 4096
    })
}

fn anthropic_payload(model: &str, prompt: &str, stream: bool) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "stream": stream,
        "max_tokens": 4096,
        "temperature": 0.3,
        "messages": [{ "role": "user", "content": prompt }]
    })
}

async fn send_streaming_request(
    client: reqwest::Client,
    config: AgentProviderConfig,
    model: String,
    prompt: String,
    agent_index: usize,
    status_tx: mpsc::UnboundedSender<(usize, AgentStatus)>,
    abort: Arc<AtomicBool>,
) -> Result<String, String> {
    let (url, payload) = match config.provider {
        AgentProvider::Ollama => (
            format!("{}/api/generate", config.api_url.trim_end_matches('/')),
            ollama_payload(&model, &prompt, true),
        ),
        AgentProvider::OpenAI | AgentProvider::LlamaCpp => (
            format!(
                "{}/v1/chat/completions",
                config.api_url.trim_end_matches('/')
            ),
            openai_payload(&model, &prompt, true),
        ),
        AgentProvider::Anthropic => (
            format!("{}/v1/messages", config.api_url.trim_end_matches('/')),
            anthropic_payload(&model, &prompt, true),
        ),
    };

    let mut request = client.post(&url).json(&payload);
    if !config.api_key.is_empty() {
        request = match config.provider {
            AgentProvider::Anthropic => request
                .header("x-api-key", &config.api_key)
                .header("anthropic-version", "2023-06-01"),
            _ => request.bearer_auth(&config.api_key),
        };
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("Provider request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Provider HTTP {}", response.status()));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut report = String::new();

    while let Some(chunk) = stream.next().await {
        if abort.load(Ordering::Relaxed) {
            return Err("Cancelled".to_string());
        }
        let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(newline) = buffer.find('\n') {
            let line = buffer[..newline].trim().to_string();
            buffer = buffer[newline + 1..].to_string();
            if line.is_empty() {
                continue;
            }
            let line = line.strip_prefix("data:").unwrap_or(&line).trim();
            if line == "[DONE]" {
                continue;
            }
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                let delta = match config.provider {
                    AgentProvider::Ollama => json["response"].as_str(),
                    AgentProvider::OpenAI | AgentProvider::LlamaCpp => {
                        json["choices"][0]["delta"]["content"].as_str()
                    }
                    AgentProvider::Anthropic => json["delta"]["text"]
                        .as_str()
                        .or_else(|| json["content_block"]["text"].as_str()),
                };
                if let Some(text) = delta {
                    report.push_str(text);
                    let _ = status_tx.send((agent_index, AgentStatus::Running(report.clone())));
                }
            }
        }
    }

    if report.trim().is_empty() {
        Err("No response from model".to_string())
    } else {
        Ok(report)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_agent_async(
    mission: AgentMission,
    model: String,
    config: AgentProviderConfig,
    agent_index: usize,
    status_tx: mpsc::UnboundedSender<(usize, AgentStatus)>,
    process_data: Option<Vec<ProcessInfo>>,
    connection_data: Option<(Vec<NetworkConnection>, String)>,
    locale: String,
    dependency_context: Option<String>,
    abort: Arc<AtomicBool>,
) {
    tokio::spawn(async move {
        let prompt = match build_prompt(
            mission,
            process_data.as_deref(),
            connection_data
                .as_ref()
                .map(|(connections, name)| (connections.as_slice(), name.as_str())),
            dependency_context.as_deref(),
            &locale,
        ) {
            Ok(prompt) => prompt,
            Err(e) => {
                let _ = status_tx.send((agent_index, AgentStatus::Failed(e)));
                return;
            }
        };

        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(180))
            .build()
        {
            Ok(client) => client,
            Err(e) => {
                let _ = status_tx.send((
                    agent_index,
                    AgentStatus::Failed(format!("HTTP client error: {}", e)),
                ));
                return;
            }
        };

        let _ = status_tx.send((agent_index, AgentStatus::Running(String::new())));
        match send_streaming_request(
            client,
            config,
            model,
            prompt,
            agent_index,
            status_tx.clone(),
            abort,
        )
        .await
        {
            Ok(report) => {
                let _ = status_tx.send((agent_index, AgentStatus::Completed(report)));
            }
            Err(e) => {
                let _ = status_tx.send((agent_index, AgentStatus::Failed(e)));
            }
        }
    });
}

pub async fn fetch_ollama_models(api_url: &str) -> Result<Vec<String>, String> {
    let url = format!("{}/api/tags", api_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to reach Ollama: {}", e))?;

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let models = body["models"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(models)
}

pub fn save_agent_report(
    target: &str,
    mission: AgentMission,
    report: &str,
    provider_label: &str,
    model: &str,
) -> Option<String> {
    let dir = crate::config::config_dir().join("agent_history");
    std::fs::create_dir_all(&dir).ok()?;
    let clean_target: String = target
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let path = dir.join(format!(
        "{}_{}_{}.md",
        timestamp,
        mission_title(mission).replace(' ', "_").to_lowercase(),
        clean_target
    ));
    let header = format!("agent:{}|{}\n", provider_label, model);
    let content = format!("{}{}", header, report);
    std::fs::write(&path, content).ok()?;
    Some(path.display().to_string())
}

fn parse_agent_meta_line(content: &str) -> Option<(&str, &str)> {
    let first_line = content.lines().next()?;
    let rest = first_line.strip_prefix("agent:")?;
    let (provider_label, model) = rest.split_once('|')?;
    Some((provider_label, model))
}

fn provider_from_label(label: &str) -> AgentProvider {
    match label {
        "OpenAI" => AgentProvider::OpenAI,
        "Anthropic" => AgentProvider::Anthropic,
        "llama.cpp" => AgentProvider::LlamaCpp,
        _ => AgentProvider::Ollama,
    }
}

pub fn load_agent_history() -> Vec<AgentInstance> {
    let dir = crate::config::config_dir().join("agent_history");
    if !dir.exists() {
        return Vec::new();
    }

    let lookup: Vec<(&str, AgentMission)> = vec![
        ("process_analysis", AgentMission::ProcessAnalysis),
        ("network_analysis", AgentMission::NetworkAnalysis),
        ("dns_analysis", AgentMission::DnsAnalysis),
        ("file_analyzer", AgentMission::FileAnalyzer),
        ("port_scanner", AgentMission::PortScanner),
        ("log_analyzer", AgentMission::LogAnalyzer),
        ("memory_analyzer", AgentMission::MemoryAnalyzer),
        ("vulnerability_check", AgentMission::VulnerabilityCheck),
        ("threat_intel", AgentMission::ThreatIntel),
    ];

    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .collect();

    entries.sort_by_key(|e| e.file_name());

    let mut agents = Vec::new();
    for entry in entries {
        let path = entry.path();
        let file_stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };

        let parts: Vec<&str> = file_stem.split('_').collect();
        if parts.len() < 4 {
            continue;
        }

        let mut matched = false;
        for (name, mission) in &lookup {
            let name_parts: Vec<&str> = name.split('_').collect();
            if parts.len() >= 2 + name_parts.len()
                && parts[2..2 + name_parts.len()] == name_parts[..]
            {
                let target = parts[2 + name_parts.len()..].join("_");
                if let Ok(raw) = std::fs::read_to_string(&path) {
                    let (provider, model, content) =
                        if let Some((pl, m)) = parse_agent_meta_line(&raw) {
                            let rest: String = raw.lines().skip(1).collect::<Vec<_>>().join("\n");
                            (provider_from_label(pl), m.to_string(), rest)
                        } else {
                            (AgentProvider::Ollama, String::new(), raw)
                        };
                    agents.push(AgentInstance {
                        mission: *mission,
                        provider,
                        model,
                        status: AgentStatus::Completed(content),
                        started_at_frame: 0,
                        completed_at_frame: None,
                        target_name: target,
                        target_path: None,
                        launch_data: None,
                        history_path: Some(path.display().to_string()),
                    });
                }
                matched = true;
                break;
            }
        }
        if !matched {
            continue;
        }
    }

    agents
}
