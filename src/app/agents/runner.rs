use crate::app::agents::constants::{
    CANCELLED_ERROR, NO_RESPONSE_ERROR, OLLAMA_FETCH_TIMEOUT, OLLAMA_TAGS_PATH,
    OLLAMA_FETCH_ERROR_PREFIX, PARSE_RESPONSE_ERROR_PREFIX, PROVIDER_HTTP_ERROR_PREFIX,
    PROVIDER_REQUEST_ERROR_PREFIX, SSE_DATA_PREFIX, SSE_DONE_SENTINEL, STREAM_NEWLINE,
};
use crate::app::agents::provider::{build_http_client, ProviderSpec};
use crate::app::agents::prompt::{self, PromptInput};
use crate::app::network::NetworkConnection;
use crate::app::process::ProcessInfo;
use crate::app::types::{AgentMission, AgentProviderConfig, AgentStatus};
use futures_util::StreamExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

pub fn spawn(
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
        let prompt = match build_prompt_or_fail(
            mission,
            process_data.as_deref(),
            connection_data
                .as_ref()
                .map(|(connections, name)| (connections.as_slice(), name.as_str())),
            dependency_context.as_deref(),
            &locale,
            agent_index,
            &status_tx,
        ) {
            Some(p) => p,
            None => return,
        };

        let client = match build_http_client() {
            Ok(c) => c,
            Err(e) => {
                let _ = status_tx.send((agent_index, AgentStatus::Failed(e)));
                return;
            }
        };

        let _ = status_tx.send((agent_index, AgentStatus::Running(String::new())));
        let result = run_streaming(
            client,
            &config,
            &model,
            &prompt,
            agent_index,
            status_tx.clone(),
            abort,
        )
        .await;

        match result {
            Ok(report) => {
                let _ = status_tx.send((agent_index, AgentStatus::Completed(report)));
            }
            Err(e) => {
                let _ = status_tx.send((agent_index, AgentStatus::Failed(e)));
            }
        }
    });
}

fn build_prompt_or_fail(
    mission: AgentMission,
    processes: Option<&[ProcessInfo]>,
    networks: Option<(&[NetworkConnection], &str)>,
    dependency_context: Option<&str>,
    locale: &str,
    agent_index: usize,
    status_tx: &mpsc::UnboundedSender<(usize, AgentStatus)>,
) -> Option<String> {
    match prompt::build(PromptInput {
        mission,
        processes,
        networks,
        dependency_context,
        locale,
    }) {
        Ok(p) => Some(p),
        Err(e) => {
            let _ = status_tx.send((agent_index, AgentStatus::Failed(e)));
            None
        }
    }
}

async fn run_streaming(
    client: reqwest::Client,
    config: &AgentProviderConfig,
    model: &str,
    prompt: &str,
    agent_index: usize,
    status_tx: mpsc::UnboundedSender<(usize, AgentStatus)>,
    abort: Arc<AtomicBool>,
) -> Result<String, String> {
    let spec = ProviderSpec::for_provider(config.provider);
    let url = spec.build_url(&config.api_url);
    let body = spec.build_body(model, prompt, true);

    let mut request = client.post(&url).json(&body);
    request = spec.apply_auth(request, &config.api_key);

    let response = request
        .send()
        .await
        .map_err(|e| format!("{}: {}", PROVIDER_REQUEST_ERROR_PREFIX, e))?;

    if !response.status().is_success() {
        return Err(format!("{} {}", PROVIDER_HTTP_ERROR_PREFIX, response.status()));
    }

    collect_stream(response, &spec, agent_index, status_tx, abort).await
}

async fn collect_stream(
    response: reqwest::Response,
    spec: &ProviderSpec,
    agent_index: usize,
    status_tx: mpsc::UnboundedSender<(usize, AgentStatus)>,
    abort: Arc<AtomicBool>,
) -> Result<String, String> {
    use crate::app::agents::constants::STREAM_ERROR_PREFIX;
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut report = String::new();

    while let Some(chunk) = stream.next().await {
        if abort.load(Ordering::Relaxed) {
            return Err(CANCELLED_ERROR.to_string());
        }
        let chunk = chunk.map_err(|e| format!("{}: {}", STREAM_ERROR_PREFIX, e))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        drain_sse_buffer(&mut buffer, &mut report, spec, agent_index, &status_tx);
    }

    if report.trim().is_empty() {
        Err(NO_RESPONSE_ERROR.to_string())
    } else {
        Ok(report)
    }
}

fn drain_sse_buffer(
    buffer: &mut String,
    report: &mut String,
    spec: &ProviderSpec,
    agent_index: usize,
    status_tx: &mpsc::UnboundedSender<(usize, AgentStatus)>,
) {
    while let Some(newline) = buffer.find(STREAM_NEWLINE) {
        let line = buffer[..newline].trim().to_string();
        *buffer = buffer[newline + 1..].to_string();
        if let Some(text) = parse_event_line(&line, spec) {
            report.push_str(&text);
            let _ = status_tx.send((agent_index, AgentStatus::Running(report.clone())));
        }
    }
}

fn parse_event_line(line: &str, spec: &ProviderSpec) -> Option<String> {
    if line.is_empty() {
        return None;
    }
    let trimmed = line.strip_prefix(SSE_DATA_PREFIX).unwrap_or(line).trim();
    if trimmed == SSE_DONE_SENTINEL {
        return None;
    }
    let json: serde_json::Value = serde_json::from_str(trimmed).ok()?;
    spec.extract_delta(&json).map(|s| s.to_string())
}

pub async fn fetch_ollama_models(api_url: &str) -> Result<Vec<String>, String> {
    use crate::app::agents::constants::HTTP_CLIENT_ERROR_PREFIX;
    let base = api_url.trim_end_matches('/');
    let path = OLLAMA_TAGS_PATH.trim_start_matches('/');
    let url = if base.ends_with(&path) {
        base.to_string()
    } else {
        format!("{}/{}", base, path)
    };
    let client = reqwest::Client::builder()
        .timeout(OLLAMA_FETCH_TIMEOUT)
        .build()
        .map_err(|e| format!("{}: {}", HTTP_CLIENT_ERROR_PREFIX, e))?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("{}: {}", OLLAMA_FETCH_ERROR_PREFIX, e))?;

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("{}: {}", PARSE_RESPONSE_ERROR_PREFIX, e))?;

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
