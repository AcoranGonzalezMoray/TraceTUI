use crate::app::agents::mission::{
    missing_dependency_context_placeholder, missing_process_name_placeholder, serialize_connections,
    serialize_processes,
};
use crate::app::network::NetworkConnection;
use crate::app::process::ProcessInfo;
use crate::app::types::AgentMission;

const PROMPT_TEMPLATE: &str = r#"You are a senior incident-response agent running the "{title}" mission.

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
- Respond in the language corresponding to locale code "{locale}"."#;

pub struct PromptInput<'a> {
    pub mission: AgentMission,
    pub processes: Option<&'a [ProcessInfo]>,
    pub networks: Option<(&'a [NetworkConnection], &'a str)>,
    pub dependency_context: Option<&'a str>,
    pub locale: &'a str,
}

pub fn build(input: PromptInput<'_>) -> Result<String, String> {
    let has_processes = input
        .processes
        .map(|p| !p.is_empty())
        .unwrap_or(false);
    let has_networks = input
        .networks
        .map(|(n, _)| !n.is_empty())
        .unwrap_or(false);

    input
        .mission
        .data_kind()
        .requires(has_processes, has_networks)?;

    let proc_text = input
        .processes
        .map(serialize_processes)
        .unwrap_or_else(|| "[]".to_string());
    let (net_text, process_name) = match input.networks {
        Some((connections, name)) => (serialize_connections(connections), name.to_string()),
        None => ("[]".to_string(), missing_process_name_placeholder().to_string()),
    };

    let dependency_context = input
        .dependency_context
        .unwrap_or(missing_dependency_context_placeholder());

    Ok(PROMPT_TEMPLATE
        .replace("{title}", input.mission.title())
        .replace("{locale}", input.locale)
        .replace("{process_name}", &process_name)
        .replace("{proc_text}", &proc_text)
        .replace("{net_text}", &net_text)
        .replace("{dependency_context}", dependency_context)
        .replace("{focus}", input.mission.focus()))
}
