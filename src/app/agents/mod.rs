use crate::app::types::{AgentMission, AgentStatus, OllamaConfig};
use crate::app::process::ProcessInfo;
use crate::app::network::NetworkConnection;
use tokio::sync::mpsc;

fn build_process_system_prompt(process: &ProcessInfo, locale: &str) -> String {
    format!(
        r#"You are a process analysis expert. Analyze the following process and provide a concise security assessment.

Process Information:
- Name: {}
- PID: {}
- Path: {}
- Command Line: {}
- CPU Usage: {}%
- Memory Usage: {} bytes
- Status: {}

Focus on:
1. Whether this process is expected/legitimate
2. Suspicious characteristics (location, resource usage, naming)
3. Risk assessment (LOW/MEDIUM/HIGH/CRITICAL)
4. Recommended actions

IMPORTANT RULES:
- Respond in Markdown format using headings (# ##), bullet lists, bold (**), code blocks (```), and tables where appropriate.
- Respond in the language corresponding to locale code: "{}".
- Be concise but thorough.
- End with a clear verdict section."#,
        process.name,
        process.pid,
        process.path.as_deref().unwrap_or("N/A"),
        process.command_line.as_deref().unwrap_or("N/A"),
        process.cpu_usage,
        process.memory_usage,
        process.status,
        locale,
    )
}

fn build_network_system_prompt(
    connections: &[NetworkConnection],
    process_name: &str,
    locale: &str,
) -> String {
    let conns_json = serde_json::to_string_pretty(connections).unwrap_or_default();
    format!(
        r#"You are a network analysis expert. Analyze the following network connections and provide a concise security assessment.

Process: {}
Total Connections: {}

Connections Data:
{}

Focus on:
1. Unusual destination IPs or ports
2. Geographic anomalies (cross-region connections)
3. Known malicious patterns
4. Risk assessment (LOW/MEDIUM/HIGH/CRITICAL)
5. Recommended actions

IMPORTANT RULES:
- Respond in Markdown format using headings (# ##), bullet lists, bold (**), code blocks (```), and tables where appropriate.
- Respond in the language corresponding to locale code: "{}".
- Be concise but thorough.
- End with a clear verdict section."#,
        process_name,
        connections.len(),
        conns_json,
        locale,
    )
}

fn build_ollama_payload(model: &str, prompt: &str) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": 0.3,
            "num_predict": 4096
        }
    })
}

pub fn spawn_agent_async(
    mission: AgentMission,
    model: String,
    config: OllamaConfig,
    agent_index: usize,
    status_tx: mpsc::UnboundedSender<(usize, AgentStatus)>,
    process_data: Option<Vec<ProcessInfo>>,
    connection_data: Option<(Vec<NetworkConnection>, String)>,
    locale: String,
) {
    tokio::spawn(async move {
        let prompt = match mission {
            AgentMission::ProcessAnalysis => {
                if let Some(procs) = process_data {
                    if let Some(proc) = procs.first() {
                        build_process_system_prompt(proc, &locale)
                    } else {
                        let _ = status_tx.send((agent_index, AgentStatus::Failed("No process data available".to_string())));
                        return;
                    }
                } else {
                    let _ = status_tx.send((agent_index, AgentStatus::Failed("No process selected".to_string())));
                    return;
                }
            }
            AgentMission::NetworkAnalysis => {
                if let Some((conns, name)) = connection_data {
                    build_network_system_prompt(&conns, &name, &locale)
                } else {
                    let _ = status_tx.send((agent_index, AgentStatus::Failed("No network data available".to_string())));
                    return;
                }
            }
        };

        let payload = build_ollama_payload(&model, &prompt);
        let url = format!("{}/api/generate", config.api_url.trim_end_matches('/'));

        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                let _ = status_tx.send((agent_index, AgentStatus::Failed(format!("HTTP client error: {}", e))));
                return;
            }
        };

        let resp = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                let _ = status_tx.send((agent_index, AgentStatus::Failed(format!("Ollama request failed: {}", e))));
                return;
            }
        };

        let body: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                let _ = status_tx.send((agent_index, AgentStatus::Failed(format!("Failed to parse response: {}", e))));
                return;
            }
        };

        let response_text = body["response"]
            .as_str()
            .unwrap_or("No response from model")
            .to_string();

        let _ = status_tx.send((agent_index, AgentStatus::Completed(response_text)));
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
