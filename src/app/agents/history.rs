use crate::app::agents::constants::{
    HISTORY_DIR_NAME, HISTORY_FILE_EXT, HISTORY_META_NEWLINE, HISTORY_META_PREFIX,
    HISTORY_META_SEPARATOR, HISTORY_TIMESTAMP_FORMAT,
};
use crate::app::agents::mission::{mission_filename_slug, mission_history_lookup};
use crate::app::agents::provider::provider_label_from_string;
use crate::app::types::{AgentInstance, AgentMission, AgentProvider, AgentStatus};

pub fn save_report(
    target: &str,
    mission: AgentMission,
    report: &str,
    provider_label: &str,
    model: &str,
) -> Option<String> {
    let dir = history_dir()?;
    let clean_target: String = target.chars().map(sanitize_target_char).collect();
    let timestamp = chrono::Local::now().format(HISTORY_TIMESTAMP_FORMAT);
    let filename = format!(
        "{}_{}_{}.{}",
        timestamp,
        mission_filename_slug(mission),
        clean_target,
        HISTORY_FILE_EXT
    );
    let path = dir.join(filename);
    let header = format!(
        "{}{}{}{}\n",
        HISTORY_META_PREFIX, provider_label, HISTORY_META_SEPARATOR, model
    );
    let content = format!("{}{}", header, report);
    std::fs::write(&path, content).ok()?;
    Some(path.display().to_string())
}

fn sanitize_target_char(c: char) -> char {
    if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
        c
    } else {
        '_'
    }
}

fn history_dir() -> Option<std::path::PathBuf> {
    let dir = crate::config::config_dir().join(HISTORY_DIR_NAME);
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

pub fn load_all() -> Vec<AgentInstance> {
    let dir = match crate::config::config_dir().join(HISTORY_DIR_NAME).exists() {
        true => crate::config::config_dir().join(HISTORY_DIR_NAME),
        false => return Vec::new(),
    };

    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(is_markdown_entry)
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut agents = Vec::new();
    for entry in entries {
        if let Some(instance) = parse_entry(&entry.path()) {
            agents.push(instance);
        }
    }
    agents
}

fn is_markdown_entry(entry: &std::fs::DirEntry) -> bool {
    entry
        .path()
        .extension()
        .map(|ext| ext == HISTORY_FILE_EXT)
        .unwrap_or(false)
}

fn parse_entry(path: &std::path::Path) -> Option<AgentInstance> {
    let file_stem = path.file_stem().and_then(|s| s.to_str())?.to_string();
    let parts: Vec<&str> = file_stem.split('_').collect();
    if parts.len() < 4 {
        return None;
    }

    let mission = mission_history_lookup()
        .iter()
        .find(|(name, _)| parts_match_mission(parts.as_slice(), name))?;
    let target = extract_target(parts.as_slice(), name_parts_count(mission.0));

    let raw = std::fs::read_to_string(path).ok()?;
    let (provider, model, content) = parse_metadata(&raw);

    Some(AgentInstance {
        mission: mission.1,
        provider,
        model,
        status: AgentStatus::Completed(content),
        started_at_frame: 0,
        completed_at_frame: None,
        target_name: target,
        target_path: None,
        launch_data: None,
        history_path: Some(path.display().to_string()),
    })
}

fn name_parts_count(slug: &str) -> usize {
    slug.split('_').count()
}

fn parts_match_mission(parts: &[&str], slug: &str) -> bool {
    use crate::app::agents::constants::HISTORY_NAME_PARTS_OFFSET;
    let name_parts: Vec<&str> = slug.split('_').collect();
    parts.len() >= HISTORY_NAME_PARTS_OFFSET + name_parts.len()
        && parts[HISTORY_NAME_PARTS_OFFSET..HISTORY_NAME_PARTS_OFFSET + name_parts.len()]
            == name_parts[..]
}

fn extract_target(parts: &[&str], mission_parts_len: usize) -> String {
    use crate::app::agents::constants::HISTORY_NAME_PARTS_OFFSET;
    parts[HISTORY_NAME_PARTS_OFFSET + mission_parts_len..].join("_")
}

fn parse_metadata(raw: &str) -> (AgentProvider, String, String) {
    if let Some((provider_label, model)) = parse_meta_line(raw) {
        let content: String = raw
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join(HISTORY_META_NEWLINE);
        (
            provider_label_from_string(provider_label),
            model.to_string(),
            content,
        )
    } else {
        (AgentProvider::Ollama, String::new(), raw.to_string())
    }
}

fn parse_meta_line(content: &str) -> Option<(&str, &str)> {
    let first_line = content.lines().next()?;
    let rest = first_line.strip_prefix(HISTORY_META_PREFIX)?;
    let (provider_label, model) = rest.split_once(HISTORY_META_SEPARATOR)?;
    Some((provider_label, model))
}
