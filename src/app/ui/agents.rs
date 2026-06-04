use super::theme::THEME;
use super::widgets;
use crate::app::types::{AgentMission, AgentStatus, NavView};
use crate::app::App;
use crate::config;
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
};

fn parse_inline_md(text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut byte_idx = 0;

    while byte_idx < text.len() {
        let remaining = &text[byte_idx..];
        if remaining.starts_with("**") {
            let content_start = byte_idx + 2;
            let end = text[content_start..].find("**");
            if let Some(pos) = end {
                let content = &text[content_start..content_start + pos];
                if !content.is_empty() {
                    spans.push(Span::styled(
                        content.to_string(),
                        Style::default()
                            .fg(THEME.text_main)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                byte_idx = content_start + pos + 2;
            } else {
                spans.push(Span::styled(
                    "**".to_string(),
                    Style::default().fg(THEME.text_main),
                ));
                byte_idx += 2;
            }
        } else if remaining.starts_with('`') {
            let content_start = byte_idx + 1;
            if let Some(pos) = text[content_start..].find('`') {
                let content = &text[content_start..content_start + pos];
                spans.push(Span::styled(
                    content.to_string(),
                    Style::default().fg(THEME.success),
                ));
                byte_idx = content_start + pos + 1;
            } else {
                spans.push(Span::styled(
                    "`".to_string(),
                    Style::default().fg(THEME.text_main),
                ));
                byte_idx += 1;
            }
        } else {
            let next_bold = remaining.find("**");
            let next_code = remaining.find('`');
            let next = match (next_bold, next_code) {
                (Some(a), Some(b)) => a.min(b),
                (Some(a), None) => a,
                (None, Some(b)) => b,
                (None, None) => remaining.len(),
            };
            let end = byte_idx + next;
            spans.push(Span::styled(
                text[byte_idx..end].to_string(),
                Style::default().fg(THEME.text_main),
            ));
            byte_idx = end;
        }
    }
    spans
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let char_len = text.chars().count();
    if char_len <= width || width < 10 {
        return vec![text.to_string()];
    }
    let mut result = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut start = 0;

    while start < chars.len() {
        if start + width >= chars.len() {
            result.push(chars[start..].iter().collect());
            break;
        }
        let mut split = start + width;
        if let Some(space_offset) = chars[start..split].iter().rposition(|c| c.is_whitespace()) {
            split = start + space_offset;
        }
        if split == start {
            split = (start + width).min(chars.len());
        }
        result.push(chars[start..split].iter().collect());
        start = split;
        while start < chars.len() && chars[start].is_whitespace() {
            start += 1;
        }
    }
    result
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    let mut output: String = text.chars().take(max_chars).collect();
    if text.chars().count() > max_chars {
        output.push_str("...");
    }
    output
}

fn action_button(
    icon: &str,
    label: String,
    key: &str,
    color: ratatui::style::Color,
) -> Vec<Span<'static>> {
    vec![
        Span::raw(" "),
        Span::styled(
            format!(" {} {} ", icon, label),
            Style::default()
                .fg(THEME.background)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {} ", key), Style::default().fg(THEME.text_dim)),
    ]
}

fn render_table_row(line: &str, is_header: bool) -> Vec<Line<'static>> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return Vec::new();
    }
    let inner = trimmed.trim_matches('|');
    let cells: Vec<&str> = inner.split('|').map(|c| c.trim()).collect();
    let mut spans = Vec::new();
    for (ci, cell) in cells.iter().enumerate() {
        if ci > 0 {
            spans.push(Span::styled(" ┃ ", Style::default().fg(THEME.text_dim)));
        }
        if is_header {
            let mut cell_spans = parse_inline_md(cell);
            for span in &mut cell_spans {
                span.style = span.style.add_modifier(Modifier::BOLD);
            }
            spans.extend(cell_spans);
        } else {
            spans.extend(parse_inline_md(cell));
        }
    }
    vec![Line::from(spans)]
}

fn is_table_separator(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|')
        && trimmed.ends_with('|')
        && trimmed
            .chars()
            .all(|c| c == '|' || c == '-' || c == ':' || c == ' ' || c == '\t')
}

fn markdown_to_lines(text: &str, width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();
    let mut in_code_block = false;
    let mut pending_header_row: Option<String> = None;

    for raw_line in text.lines() {
        let line = raw_line.trim_end();

        if line.starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block {
                lines.push(Line::from(Span::styled(
                    "┌─ code ───────────────────────────",
                    Style::default().fg(THEME.success),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "└───────────────────────────────────",
                    Style::default().fg(THEME.success),
                )));
            }
            continue;
        }

        if in_code_block {
            for wrapped in wrap_text(line, width.saturating_sub(4).max(10)) {
                lines.push(Line::from(Span::styled(
                    format!("│ {} ", wrapped),
                    Style::default().fg(THEME.success),
                )));
            }
            continue;
        }

        if line.is_empty() {
            lines.push(Line::from(""));
            continue;
        }

        if let Some(content) = line.strip_prefix("### ") {
            lines.push(Line::from(Span::styled(
                format!(" {} ", content),
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            )));
            continue;
        }

        if let Some(content) = line.strip_prefix("## ") {
            lines.push(Line::from(Span::styled(
                format!(" {} ", content),
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            )));
            continue;
        }

        if let Some(content) = line.strip_prefix("# ") {
            lines.push(Line::from(Span::styled(
                format!(" {} ", content),
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )));
            continue;
        }

        if line == "---" || line == "***" || line == "___" {
            lines.push(Line::from(Span::styled(
                "─".repeat(width.saturating_sub(4).min(60)),
                Style::default().fg(THEME.text_dim),
            )));
            continue;
        }

        if is_table_separator(line) {
            if let Some(header) = pending_header_row.take() {
                for row in render_table_row(&header, true) {
                    lines.push(row);
                }
                let sep = "─".repeat(width.saturating_sub(4).min(40));
                lines.push(Line::from(Span::styled(
                    sep,
                    Style::default().fg(THEME.text_dim),
                )));
            }
            continue;
        }

        if line.trim_start().starts_with('|') && line.trim_end().ends_with('|') {
            if pending_header_row.is_none() {
                pending_header_row = Some(line.to_string());
                continue;
            }
            // if we already have a header but the next line isn't a separator, render both as body rows
            if let Some(header) = pending_header_row.take() {
                for row in render_table_row(&header, false) {
                    lines.push(row);
                }
            }
            for row in render_table_row(line, false) {
                lines.push(row);
            }
            continue;
        }

        // flush pending header if we leave table context
        if let Some(header) = pending_header_row.take() {
            for row in render_table_row(&header, false) {
                lines.push(row);
            }
        }

        if line.starts_with("- ") || line.starts_with("* ") {
            let content = &line[2..];
            for wrapped in wrap_text(content, width.saturating_sub(4).max(10)) {
                let mut spans = vec![Span::styled(" • ", Style::default().fg(THEME.primary))];
                spans.extend(parse_inline_md(&wrapped));
                lines.push(Line::from(spans));
            }
            continue;
        }

        if line.starts_with(|c: char| c.is_ascii_digit()) && line.len() > 2 {
            let dot_pos = line.find('.').unwrap_or(1);
            if line.as_bytes().get(dot_pos) == Some(&b'.')
                && line
                    .as_bytes()
                    .get(dot_pos + 1)
                    .is_some_and(|&b| b == b' ' || b == b'\t')
            {
                let num = &line[..dot_pos + 1];
                let content = &line[dot_pos + 1..].trim_start();
                for wrapped in wrap_text(content, width.saturating_sub(6).max(10)) {
                    let mut spans = vec![Span::styled(
                        format!(" {} ", num),
                        Style::default().fg(THEME.primary),
                    )];
                    spans.extend(parse_inline_md(&wrapped));
                    lines.push(Line::from(spans));
                }
                continue;
            }
        }

        for wrapped in wrap_text(line, width.saturating_sub(2).max(10)) {
            lines.push(Line::from(parse_inline_md(&wrapped)));
        }
    }

    lines
}

fn filtered_markdown_lines(app: &App, text: &str, width: usize) -> Vec<Line<'static>> {
    let mut source = Vec::new();
    let mut skip_section = false;
    let query = app.agents.agent_search_query.to_lowercase();

    for line in text.lines() {
        let is_heading = line.starts_with("# ");
        if app.agents.collapse_sections {
            if is_heading {
                source.push(format!("{}  [+]", line));
                skip_section = true;
                continue;
            }
            if line.starts_with("## ") || line.starts_with("### ") {
                source.push(format!("{}  [+]", line));
                skip_section = true;
                continue;
            }
            if skip_section {
                continue;
            }
        }
        if !query.is_empty() && !line.to_lowercase().contains(&query) {
            continue;
        }
        source.push(line.to_string());
    }

    markdown_to_lines(&source.join("\n"), width)
}

fn running_phase_text(
    frame_count: u64,
    started_at: u64,
    translator: &crate::i18n::Translator,
) -> String {
    let elapsed = frame_count.saturating_sub(started_at);
    let phase = (elapsed / 8) % 4;
    match phase {
        0 => tr!(translator, "agents.phase_analyzing"),
        1 => tr!(translator, "agents.phase_writing"),
        2 => tr!(translator, "agents.phase_deepening"),
        _ => tr!(translator, "agents.phase_improving"),
    }
}

fn spinner_for_frame(frame_count: u64) -> &'static str {
    let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    spinners[(frame_count as usize) % spinners.len()]
}

pub fn render_agents_view(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(config::SIDEBAR_LEFT_PCT),
            Constraint::Percentage(config::CENTER_PANEL_PCT),
            Constraint::Percentage(config::SIDEBAR_RIGHT_PCT),
        ])
        .split(area);

    render_agents_left(f, app, columns[0]);
    render_agents_center(f, app, columns[1]);
    render_agents_right(f, app, columns[2]);
}

fn mission_icon(mission: AgentMission) -> &'static str {
    match mission {
        AgentMission::ProcessAnalysis => "󰆧",
        AgentMission::NetworkAnalysis => "󰛳",
        AgentMission::DnsAnalysis => "󰖟",
        AgentMission::FileAnalyzer => "󰈙",
        AgentMission::PortScanner => "󰓾",
        AgentMission::LogAnalyzer => "󰌱",
        AgentMission::MemoryAnalyzer => "󰍛",
        AgentMission::VulnerabilityCheck => "󰒃",
        AgentMission::ThreatIntel => "󰳦",
    }
}

fn mission_label(mission: AgentMission) -> &'static str {
    match mission {
        AgentMission::ProcessAnalysis => "Process",
        AgentMission::NetworkAnalysis => "Network",
        AgentMission::DnsAnalysis => "DNS",
        AgentMission::FileAnalyzer => "Files",
        AgentMission::PortScanner => "Ports",
        AgentMission::LogAnalyzer => "Logs",
        AgentMission::MemoryAnalyzer => "Memory",
        AgentMission::VulnerabilityCheck => "CVE",
        AgentMission::ThreatIntel => "Intel",
    }
}

fn status_icon(status: &AgentStatus, frame_count: u64) -> &'static str {
    match status {
        AgentStatus::Idle => "⏺",
        AgentStatus::Queued => "⏳",
        AgentStatus::Running(_) => spinner_for_frame(frame_count),
        AgentStatus::Completed(_) => "✔",
        AgentStatus::Failed(_) => "✘",
    }
}

fn status_color(status: &AgentStatus) -> ratatui::style::Color {
    match status {
        AgentStatus::Idle => THEME.text_dim,
        AgentStatus::Queued => THEME.text_dim,
        AgentStatus::Running(_) => THEME.warning,
        AgentStatus::Completed(_) => THEME.success,
        AgentStatus::Failed(_) => THEME.danger,
    }
}

fn render_agents_left(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.ui.sidebar_focus == crate::app::SidebarFocus::Left
        && app.ui.current_nav_view == NavView::Agents;
    let border_color = if is_focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let border_type = if is_focused {
        BorderType::Thick
    } else {
        BorderType::Rounded
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(border_type)
        .title(format!(" {} ", tr!(app.ui.translator, "agents.agents")))
        .title_style(
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    if app.agents.history_loading {
        let spinner = spinner_for_frame(app.ui.frame_count);
        let msg = Paragraph::new(Line::from(vec![
            Span::styled(
                spinner,
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                tr!(app.ui.translator, "agents.loading_history"),
                Style::default().fg(THEME.text_dim),
            ),
        ]))
        .alignment(Alignment::Center);
        f.render_widget(msg, inner);
        return;
    }

    if app.agents.agents.is_empty() {
        let msg = Paragraph::new(Line::from(vec![Span::styled(
            tr!(app.ui.translator, "agents.no_history"),
            Style::default().fg(THEME.text_dim),
        )]))
        .alignment(Alignment::Center);
        f.render_widget(msg, inner);
        return;
    }

    let constraints: Vec<Constraint> = app
        .agents
        .agents
        .iter()
        .flat_map(|_| vec![Constraint::Length(3), Constraint::Length(1)])
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, agent) in app.agents.agents.iter().enumerate() {
        if i * 2 >= chunks.len() {
            break;
        }
        let is_selected = i == app.agents.selected_agent_index;
        let area = chunks[i * 2];

        if area.height == 0 {
            continue;
        }

        let style = if is_selected {
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(THEME.text_dim)
        };

        let icon = status_icon(&agent.status, app.ui.frame_count);
        let color = status_color(&agent.status);
        let target = if agent.target_name.is_empty() {
            mission_label(agent.mission)
        } else {
            &agent.target_name
        };
        let title = format!(" {} {}", icon, target);

        let subtitle = format!(
            " {} {} [{}]",
            mission_icon(agent.mission),
            agent.provider.label(),
            agent.model
        );

        let lines = vec![
            Line::from(Span::styled(title, style.fg(color))),
            Line::from(Span::styled(subtitle, Style::default().fg(THEME.text_dim))),
            Line::from(Span::styled(
                match &agent.status {
                    AgentStatus::Idle => tr!(app.ui.translator, "agents.status_idle"),
                    AgentStatus::Queued => tr!(app.ui.translator, "agents.status_queued"),
                    AgentStatus::Running(_) => tr!(app.ui.translator, "agents.status_running"),
                    AgentStatus::Completed(_) => tr!(app.ui.translator, "agents.status_done"),
                    AgentStatus::Failed(msg) => {
                        let first = msg.lines().next().unwrap_or(msg);
                        truncate_chars(first, 45)
                    }
                },
                Style::default().fg(THEME.text_dim),
            )),
        ];

        let agent_block = if is_selected {
            Block::default()
                .borders(Borders::LEFT)
                .border_style(Style::default().fg(THEME.primary))
        } else {
            Block::default()
        };

        f.render_widget(Paragraph::new(lines).block(agent_block), area);
    }
}

fn render_agents_center(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.ui.sidebar_focus == crate::app::SidebarFocus::Center
        && app.ui.current_nav_view == NavView::Agents;
    let border_color = if is_focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let border_type = if is_focused {
        BorderType::Thick
    } else {
        BorderType::Rounded
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(border_type)
        .title(format!(
            " {}  ·  {}={}  ·  +/- {} ",
            tr!(app.ui.translator, "agents.detail"),
            tr!(app.ui.translator, "agents.parallel_short"),
            app.agents.max_parallel_agents,
            tr!(app.ui.translator, "agents.key_parallel")
        ))
        .title_style(
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    if app.agents.history_loading {
        let spinner = spinner_for_frame(app.ui.frame_count);
        let msg = Paragraph::new(Line::from(vec![
            Span::styled(
                spinner,
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                tr!(app.ui.translator, "agents.loading_history"),
                Style::default().fg(THEME.text_dim),
            ),
        ]))
        .alignment(Alignment::Center);
        f.render_widget(msg, inner);
        return;
    }

    let selected = app.agents.selected_agent_index;
    if app.agents.agents.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(Span::styled(
                tr!(app.ui.translator, "agents.no_history"),
                Style::default().fg(THEME.text_dim),
            )),
            Line::from(""),
            Line::from(Span::styled(
                tr!(app.ui.translator, "agents.launch_empty_hint"),
                Style::default().fg(THEME.text_dim),
            )),
        ])
        .alignment(Alignment::Center);
        f.render_widget(msg, inner);
        return;
    }

    if selected >= app.agents.agents.len() {
        let msg = Paragraph::new(vec![
            Line::from(Span::styled(
                tr!(app.ui.translator, "agents.select_hint"),
                Style::default().fg(THEME.text_dim),
            )),
            Line::from(""),
            Line::from(Span::styled(
                tr!(app.ui.translator, "agents.launch_empty_hint"),
                Style::default().fg(THEME.text_dim),
            )),
        ])
        .alignment(Alignment::Center);
        f.render_widget(msg, inner);
        return;
    }

    let agent = &app.agents.agents[selected];
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(vec![
        Span::styled(
            format!(" {} ", mission_icon(agent.mission)),
            Style::default().fg(THEME.warning),
        ),
        Span::styled(
            format!(" {} ", mission_label(agent.mission)),
            Style::default()
                .fg(THEME.text_main)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.agent_status")),
            Style::default()
                .fg(THEME.background)
                .bg(status_color(&agent.status))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            match &agent.status {
                AgentStatus::Idle => tr!(app.ui.translator, "agents.status_idle"),
                AgentStatus::Queued => tr!(app.ui.translator, "agents.status_queued"),
                AgentStatus::Running(_) => tr!(app.ui.translator, "agents.status_running"),
                AgentStatus::Completed(_) => tr!(app.ui.translator, "agents.status_done"),
                AgentStatus::Failed(_) => tr!(app.ui.translator, "agents.status_failed"),
            },
            Style::default()
                .fg(status_color(&agent.status))
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from("")); // top margin

    let mut button_spans = Vec::new();
    match &agent.status {
        AgentStatus::Running(_) | AgentStatus::Queued => {
            for span in action_button(
                "⏹",
                tr!(app.ui.translator, "agents.action_cancel"),
                "S",
                THEME.danger,
            ) {
                button_spans.push(span);
            }
        }
        AgentStatus::Completed(_) => {}
        _ => {
            for span in action_button(
                "󰑐",
                tr!(app.ui.translator, "agents.action_retry"),
                "R",
                THEME.warning,
            ) {
                button_spans.push(span);
            }
        }
    }
    for span in action_button(
        "󰒈",
        tr!(app.ui.translator, "agents.action_export_md"),
        "E",
        THEME.secondary,
    ) {
        button_spans.push(span);
    }
    for span in action_button(
        "󰘦",
        tr!(app.ui.translator, "agents.action_export_json"),
        "J",
        THEME.secondary,
    ) {
        button_spans.push(span);
    }
    for span in action_button(
        "󰍉",
        tr!(app.ui.translator, "agents.action_filter"),
        "F",
        THEME.primary,
    ) {
        button_spans.push(span);
    }
    for span in action_button(
        if app.agents.collapse_sections {
            "󰅀"
        } else {
            "󰅂"
        },
        if app.agents.collapse_sections {
            tr!(app.ui.translator, "agents.action_expand")
        } else {
            tr!(app.ui.translator, "agents.action_collapse")
        },
        "Z",
        THEME.warning,
    ) {
        button_spans.push(span);
    }

    // center buttons on X axis
    let btn_width: usize = button_spans.iter().map(|s| s.content.len()).sum();
    let inner_w = inner.width as usize;
    let pad = if inner_w > btn_width {
        (inner_w - btn_width) / 2
    } else {
        0
    };
    let mut centered = Vec::new();
    centered.push(Span::styled(" ".repeat(pad), Style::default()));
    centered.extend(button_spans);
    centered.push(Span::styled(" ".repeat(pad), Style::default()));
    lines.push(Line::from(centered));
    lines.push(Line::from(""));

    if !agent.target_name.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {}: ", tr!(app.ui.translator, "agents.target")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                &agent.target_name,
                Style::default()
                    .fg(THEME.text_main)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    if let Some(ref path) = agent.target_path {
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {}: ", tr!(app.ui.translator, "agents.path")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(path, Style::default().fg(THEME.text_main)),
        ]));
    }
    lines.push(Line::from(vec![
        Span::styled(
            format!(" {}: ", tr!(app.ui.translator, "agents.agent_model")),
            Style::default().fg(THEME.text_dim),
        ),
        Span::styled(
            format!("{} / {}", agent.provider.label(), agent.model),
            Style::default().fg(THEME.text_main),
        ),
    ]));
    if let Some(path) = &agent.history_path {
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {}: ", tr!(app.ui.translator, "agents.history")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(path, Style::default().fg(THEME.text_main)),
        ]));
    }
    if let Some(end_frame) = agent.completed_at_frame {
        let elapsed_ms = end_frame.saturating_sub(agent.started_at_frame) * 250;
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {}: ", tr!(app.ui.translator, "agents.time")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                format!("{}.{:01}s", elapsed_ms / 1000, (elapsed_ms % 1000) / 100),
                Style::default().fg(THEME.text_main),
            ),
        ]));
    }
    lines.push(Line::from(""));

    match &agent.status {
        AgentStatus::Idle => {
            lines.push(Line::from(Span::styled(
                format!("  {} ", tr!(app.ui.translator, "agents.status_idle")),
                Style::default().fg(THEME.text_dim),
            )));
        }
        AgentStatus::Queued => {
            lines.push(Line::from(Span::styled(
                format!("  ⏳ {} ", tr!(app.ui.translator, "agents.status_queued")),
                Style::default().fg(THEME.text_dim),
            )));
        }
        AgentStatus::Running(msg) => {
            let phase = running_phase_text(
                app.ui.frame_count,
                agent.started_at_frame,
                &app.ui.translator,
            );
            let inner_w = inner.width as usize;
            let elapsed = app.ui.frame_count.saturating_sub(agent.started_at_frame);
            let progress =
                ((elapsed as usize * 2).min(95)).max(if msg.is_empty() { 5 } else { 20 });
            let eta_secs = 180usize.saturating_sub(elapsed as usize / 4);
            let filled = progress / 10;
            let bar = format!(
                "[{}{}] {}% ~{}s",
                "█".repeat(filled),
                "░".repeat(10usize.saturating_sub(filled)),
                progress,
                eta_secs
            );
            let pad_left = inner_w.saturating_sub(phase.len() + 4) / 2;
            let spinner = spinner_for_frame(app.ui.frame_count);
            let line = format!(" {} {} ", spinner, phase);
            let padded = format!("{:pad_left$}{}", "", line, pad_left = pad_left);
            lines.push(Line::from(Span::styled(
                padded,
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " ".repeat(inner_w.saturating_sub(bar.len()) / 2) + &bar,
                Style::default().fg(THEME.text_dim),
            )));
            if !msg.trim().is_empty() {
                lines.push(Line::from(""));
                let scroll = app.agents.agent_detail_scroll;
                let md_lines = filtered_markdown_lines(app, msg, inner.width as usize);
                let available = (inner.height as usize).saturating_sub(12);
                for line in md_lines.iter().skip(scroll).take(available) {
                    lines.push(line.clone());
                }
            }
        }
        AgentStatus::Completed(msg) | AgentStatus::Failed(msg) => {
            let sep = Line::from(Span::styled(
                "─".repeat(inner.width.saturating_sub(2) as usize),
                Style::default().fg(THEME.text_dim),
            ));
            lines.push(sep);
            lines.push(Line::from(""));

            let scroll = app.agents.agent_detail_scroll;
            let md_lines = filtered_markdown_lines(app, msg, inner.width as usize);
            let available = (inner.height as usize).saturating_sub(10);
            let end = (scroll + available).min(md_lines.len());
            for line in md_lines
                .iter()
                .skip(scroll)
                .take(end.saturating_sub(scroll))
            {
                lines.push(line.clone());
            }
            if md_lines.len() > end {
                lines.push(Line::from(Span::styled(
                    format!("▼ (+{} lines)", md_lines.len() - end),
                    Style::default().fg(THEME.text_dim),
                )));
            }
        }
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner);

    if app.agents.agent_search_mode {
        render_agent_search_overlay(f, app, inner);
    }
}

fn render_agent_search_overlay(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let overlay_height = 3;
    let overlay_y = area.y;
    let overlay = Rect {
        x: area.x + 2,
        y: overlay_y,
        width: area.width.saturating_sub(4),
        height: overlay_height,
    };
    f.render_widget(Clear, overlay);

    let cursor = if app.ui.frame_count.is_multiple_of(2) {
        "█"
    } else {
        " "
    };
    let query = &app.agents.agent_search_query;
    let match_count = if query.is_empty() {
        0
    } else {
        app.agents
            .agents
            .get(app.agents.selected_agent_index)
            .map_or(0, |agent| {
                let text = match &agent.status {
                    AgentStatus::Running(t)
                    | AgentStatus::Completed(t)
                    | AgentStatus::Failed(t) => t.as_str(),
                    _ => "",
                };
                text.to_lowercase().matches(&query.to_lowercase()).count()
            })
    };

    let search_line = Line::from(vec![
        Span::styled(
            " 󰍉 ",
            Style::default().fg(THEME.background).bg(THEME.primary),
        ),
        Span::styled(
            " SEARCH ",
            Style::default()
                .fg(THEME.background)
                .bg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            query.clone(),
            Style::default()
                .fg(THEME.text_main)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(cursor, Style::default().fg(THEME.primary)),
        Span::raw(" "),
        Span::styled(
            format!("({})", match_count),
            Style::default().fg(if match_count > 0 {
                THEME.success
            } else {
                THEME.danger
            }),
        ),
        Span::raw(" "),
        Span::styled("[ESC]", Style::default().fg(THEME.text_dim)),
    ]);

    let bg = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(THEME.background));
    f.render_widget(bg, overlay);
    let inner_overlay = Block::default().padding(ratatui::widgets::Padding::new(1, 1, 0, 0));
    f.render_widget(Paragraph::new(search_line).block(inner_overlay), overlay);
}

fn render_agents_right(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.ui.sidebar_focus == crate::app::SidebarFocus::Right
        && app.ui.current_nav_view == NavView::Agents;
    let border_color = if is_focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let border_type = if is_focused {
        BorderType::Thick
    } else {
        BorderType::Rounded
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(border_type)
        .title(format!(" 󰬒 {} ", tr!(app.ui.translator, "actions.title")))
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let t = &app.ui.translator;

    let selected_agent_status = app
        .agents
        .agents
        .get(app.agents.selected_agent_index)
        .map(|a| &a.status);

    let mut action_items: Vec<(&str, String, &str, ratatui::style::Color)> = vec![
        (
            "󰈐",
            format!(
                "{} ({})",
                tr!(t, "agents.action_provider"),
                app.agents.ollama.provider.label()
            ),
            "C",
            THEME.secondary,
        ),
        ("󰚩", tr!(t, "agents.action_launch"), "A", THEME.primary),
    ];

    if !app.agents.history_loading {
        match selected_agent_status {
            Some(AgentStatus::Running(_)) | Some(AgentStatus::Queued) => {
                action_items.push(("⏹", tr!(t, "agents.action_cancel"), "S", THEME.danger));
            }
            _ => {
                action_items.push(("󰑐", tr!(t, "agents.action_retry"), "R", THEME.warning));
            }
        }
    }

    action_items.push((
        "󰓅",
        format!(
            "{}: {} ({})",
            tr!(t, "agents.parallel_label"),
            app.agents.max_parallel_agents,
            "+/-"
        ),
        "+/-",
        THEME.warning,
    ));
    action_items.push(("󰩈", tr!(t, "agents.action_clear"), "X", THEME.danger));

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(config::SCROLLBAR_WIDTH),
        ])
        .split(inner);
    let list_area = chunks[0];
    let scrollbar_area = chunks[1];

    let items: Vec<ListItem> = action_items
        .iter()
        .enumerate()
        .map(|(i, (icon, title, key, color))| {
            let is_selected = i == app.agents.agent_action_index;
            let prefix = if is_selected { " ▎" } else { "  " };
            let prefix_style = if is_selected {
                Style::default().fg(THEME.primary)
            } else {
                Style::default()
            };
            let title_style = if is_selected {
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_main)
            };
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled(format!(" {} ", icon), Style::default().fg(*color)),
                    Span::styled(title.clone(), title_style),
                ]),
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(format!("[ {} ]", key), Style::default().fg(THEME.text_dim)),
                ]),
            ])
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.agents.agent_action_index));
    f.render_stateful_widget(
        List::new(items).block(Block::default()),
        list_area,
        &mut list_state,
    );
    widgets::render_scrollbar(
        f,
        scrollbar_area,
        action_items.len(),
        app.agents.agent_action_index,
    );
}

pub fn render_provider_modal(f: &mut ratatui::Frame, app: &App) {
    let pw = (f.area().width * 3 / 5).max(56).min(f.area().width);
    let ph = (f.area().height * 3 / 7).max(20).min(f.area().height);
    let popup_area = Rect {
        x: (f.area().width.saturating_sub(pw)) / 2,
        y: (f.area().height.saturating_sub(ph)) / 2,
        width: pw,
        height: ph,
    };

    let has_focus = app.agents.provider_modal_focus;
    let cursor = if app.ui.frame_count.is_multiple_of(2) {
        "█"
    } else {
        " "
    };

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  󰈐 ", Style::default().fg(THEME.warning)),
        Span::styled(
            format!(
                "{}: {}",
                tr!(app.ui.translator, "agents.provider_config_title"),
                app.agents.ollama.provider.label()
            ),
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![Span::styled(
        format!("  {}", tr!(app.ui.translator, "agents.provider_hint")),
        Style::default().fg(THEME.text_dim),
    )]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled(
            "  Provider: ",
            if has_focus == 0 {
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_dim)
            },
        ),
        Span::styled(
            app.agents.ollama.provider.label(),
            Style::default().fg(THEME.text_main),
        ),
        Span::styled(
            format!(
                "  ({})",
                tr!(app.ui.translator, "agents.provider_cycle_hint")
            ),
            Style::default().fg(THEME.text_dim),
        ),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled(
            format!("  {}: ", tr!(app.ui.translator, "agents.provider_url")),
            if has_focus == 1 {
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_dim)
            },
        ),
        Span::styled(
            &app.agents.ollama_url_input,
            Style::default().fg(THEME.text_main),
        ),
        Span::styled(
            if has_focus == 1 { cursor } else { " " },
            Style::default().fg(THEME.primary),
        ),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled(
            format!("  {}: ", tr!(app.ui.translator, "agents.models_label")),
            if has_focus == 2 {
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_dim)
            },
        ),
        Span::styled(
            &app.agents.ollama_model_input,
            Style::default().fg(THEME.text_main),
        ),
        Span::styled(
            if has_focus == 2 { cursor } else { " " },
            Style::default().fg(THEME.primary),
        ),
    ]));
    lines.push(Line::from(""));

    let masked_key = if app.agents.agent_api_key_input.is_empty() {
        "".to_string()
    } else {
        "•".repeat(app.agents.agent_api_key_input.len().min(24))
    };
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {}: ", tr!(app.ui.translator, "agents.api_key")),
            if has_focus == 3 {
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_dim)
            },
        ),
        Span::styled(masked_key, Style::default().fg(THEME.text_main)),
        Span::styled(
            if has_focus == 3 { cursor } else { " " },
            Style::default().fg(THEME.primary),
        ),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled(
            format!("  {}: ", tr!(app.ui.translator, "agents.models")),
            Style::default().fg(THEME.text_dim),
        ),
        Span::styled(
            format!("{}", app.agents.ollama_models.len()),
            Style::default().fg(THEME.text_main),
        ),
        Span::styled(
            format!("  · {}", tr!(app.ui.translator, "agents.model_list_hint")),
            Style::default().fg(THEME.text_dim),
        ),
    ]));

    let visible_models: usize = (popup_area.height as usize).saturating_sub(17);
    for (i, model) in app.agents.ollama_models.iter().enumerate() {
        if i >= visible_models {
            break;
        }
        let is_selected = i == app.agents.selected_model_index;
        let is_focused = has_focus == 4 && is_selected;
        let line = if is_focused {
            Line::from(vec![
                Span::styled(" ▎", Style::default().fg(THEME.primary)),
                Span::styled(
                    format!(" {} ", model),
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.primary)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        } else if is_selected {
            Line::from(vec![
                Span::styled(" ▎", Style::default().fg(THEME.primary)),
                Span::styled(
                    format!(" {} ", model),
                    Style::default()
                        .fg(THEME.primary)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        } else {
            Line::from(vec![
                Span::raw("   "),
                Span::styled(format!(" {} ", model), Style::default().fg(THEME.text_main)),
            ])
        };
        lines.push(line);
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.fetch_models")),
            Style::default()
                .fg(THEME.background)
                .bg(if has_focus == 5 {
                    THEME.primary
                } else {
                    THEME.secondary
                })
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.save")),
            Style::default()
                .fg(THEME.background)
                .bg(if has_focus == 6 {
                    THEME.primary
                } else {
                    THEME.success
                })
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.cancel")),
            Style::default()
                .fg(THEME.background)
                .bg(if has_focus == 7 {
                    THEME.primary
                } else {
                    THEME.danger
                })
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![Span::styled(
        format!(
            "  {}: {}",
            tr!(app.ui.translator, "agents.switch_field"),
            tr!(app.ui.translator, "agents.provider_keys_hint")
        ),
        Style::default().fg(THEME.text_dim),
    )]));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(THEME.warning))
        .title(format!(
            " {} ",
            tr!(app.ui.translator, "agents.configure_provider")
        ))
        .title_style(
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph.block(block), popup_area);
}

pub fn render_agent_type_selector(f: &mut ratatui::Frame, app: &App) {
    let popup_height = 18;
    let popup_width = 76;
    let popup_area = Rect {
        x: (f.area().width.saturating_sub(popup_width)) / 2,
        y: (f.area().height.saturating_sub(popup_height)) / 2,
        width: popup_width.min(f.area().width),
        height: popup_height.min(f.area().height),
    };

    let items = [
        (
            "󰆧",
            tr!(app.ui.translator, "agents.launch_process"),
            tr!(app.ui.translator, "agents.agent_process_desc"),
        ),
        (
            "󰛳",
            tr!(app.ui.translator, "agents.launch_network"),
            tr!(app.ui.translator, "agents.agent_network_desc"),
        ),
        (
            "󰖟",
            tr!(app.ui.translator, "agents.agent_dns"),
            tr!(app.ui.translator, "agents.agent_dns_desc"),
        ),
        (
            "󰈙",
            tr!(app.ui.translator, "agents.agent_file"),
            tr!(app.ui.translator, "agents.agent_file_desc"),
        ),
        (
            "󰓾",
            tr!(app.ui.translator, "agents.agent_ports"),
            tr!(app.ui.translator, "agents.agent_ports_desc"),
        ),
        (
            "󰌱",
            tr!(app.ui.translator, "agents.agent_logs"),
            tr!(app.ui.translator, "agents.agent_logs_desc"),
        ),
        (
            "󰍛",
            tr!(app.ui.translator, "agents.agent_memory"),
            tr!(app.ui.translator, "agents.agent_memory_desc"),
        ),
        (
            "󰒃",
            tr!(app.ui.translator, "agents.agent_vuln"),
            tr!(app.ui.translator, "agents.agent_vuln_desc"),
        ),
        (
            "󰳦",
            tr!(app.ui.translator, "agents.agent_intel"),
            tr!(app.ui.translator, "agents.agent_intel_desc"),
        ),
    ];

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        format!("  {} ", tr!(app.ui.translator, "agents.select_type")),
        Style::default()
            .fg(THEME.warning)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    for (i, (icon, label, desc)) in items.iter().enumerate() {
        let is_selected = i == app.agents.agent_type_selector_index;
        if is_selected {
            lines.push(Line::from(vec![
                Span::styled(" ▎", Style::default().fg(THEME.primary)),
                Span::styled(
                    format!(" {} {} ", icon, label),
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("  {}", desc), Style::default().fg(THEME.text_dim)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    format!(" {} {:<22} ", icon, label),
                    Style::default().fg(THEME.text_main),
                ),
                Span::styled(desc.clone(), Style::default().fg(THEME.text_dim)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.confirm")),
            Style::default()
                .fg(THEME.background)
                .bg(THEME.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.cancel")),
            Style::default()
                .fg(THEME.background)
                .bg(THEME.danger)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(THEME.warning))
        .title(format!(
            " {} ",
            tr!(app.ui.translator, "agents.launch_agent")
        ))
        .title_style(
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(Clear, popup_area);
    f.render_widget(Paragraph::new(lines).block(block), popup_area);
}

pub fn render_process_selector(f: &mut ratatui::Frame, app: &App) {
    let processes = &app.network.processes;
    let total = processes.len();
    let popup_height = f.area().height * 3 / 5;
    let popup_width = f.area().width * 3 / 5;
    let popup_area = Rect {
        x: (f.area().width - popup_width) / 2,
        y: (f.area().height - popup_height) / 2,
        width: popup_width,
        height: popup_height,
    };
    let inner_h = popup_area.height as usize;
    let visible = (inner_h.saturating_sub(8)).min(total);

    let cursor_idx = app.agents.selected_model_index.min(total.saturating_sub(1));
    let scroll_offset = if cursor_idx >= visible {
        cursor_idx - visible + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        format!(
            "  {} ({}) ",
            tr!(app.ui.translator, "agents.select_process"),
            total
        ),
        Style::default()
            .fg(THEME.warning)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(Span::styled(
        "  [Space] \u{2713} / \u{2717}   \u{2191}\u{2193} ",
        Style::default().fg(THEME.text_dim),
    )));

    for (i, proc) in processes
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take((scroll_offset + visible).min(total) - scroll_offset)
    {
        let path = proc.path.as_deref().unwrap_or("N/A");
        let label = format!("{} | {} (PID: {})", proc.name, path, proc.pid);
        let checked = if app.agents.selected_pids.contains(&proc.pid) {
            "[X]"
        } else {
            "[ ]"
        };
        let is_cursor = i == cursor_idx;
        if is_cursor {
            lines.push(Line::from(vec![
                Span::styled(" ▎", Style::default().fg(THEME.primary)),
                Span::styled(
                    format!(" {} {} ", checked, label),
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.primary)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        } else if app.agents.selected_pids.contains(&proc.pid) {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    format!(" {} {} ", checked, label),
                    Style::default()
                        .fg(THEME.primary)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    format!(" {} {} ", checked, label),
                    Style::default().fg(THEME.text_main),
                ),
            ]));
        }
    }

    if total > visible {
        let sb = if scroll_offset == 0 {
            "↑"
        } else if scroll_offset + visible >= total {
            "↓"
        } else {
            "↕"
        };
        lines.push(Line::from(Span::styled(
            format!(
                "   {}  {} ",
                sb,
                tr!(app.ui.translator, "agents.scroll_hint")
            ),
            Style::default().fg(THEME.text_dim),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.select_and_continue")),
            Style::default()
                .fg(THEME.background)
                .bg(THEME.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.cancel")),
            Style::default()
                .fg(THEME.background)
                .bg(THEME.danger)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![Span::styled(
        format!(
            "  {} / {}\u{00a0}\u{00a0}Space: toggle\u{00a0}\u{00a0}Enter: launch {} agent(s)",
            app.agents.selected_pids.len(),
            total,
            app.agents.selected_pids.len()
        ),
        Style::default().fg(THEME.text_dim),
    )]));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(THEME.warning))
        .title(format!(
            " {} ",
            tr!(app.ui.translator, "agents.select_process")
        ))
        .title_style(
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph.block(block), popup_area);
}

pub fn render_network_selector(f: &mut ratatui::Frame, app: &App) {
    let conns = &app.network.app_connections;
    let total = conns.len();
    let popup_height = f.area().height * 3 / 5;
    let popup_width = f.area().width * 3 / 5;
    let popup_area = Rect {
        x: (f.area().width - popup_width) / 2,
        y: (f.area().height - popup_height) / 2,
        width: popup_width,
        height: popup_height,
    };
    let inner_h = popup_area.height as usize;
    let visible = (inner_h.saturating_sub(8)).min(total);

    let cursor_idx = app.agents.selected_model_index.min(total.saturating_sub(1));
    let scroll_offset = if cursor_idx >= visible {
        cursor_idx - visible + 1
    } else {
        0
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        format!(
            "  {} ({}) ",
            tr!(app.ui.translator, "agents.select_network"),
            total
        ),
        Style::default()
            .fg(THEME.warning)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(Span::styled(
        "  [Space] \u{2713} / \u{2717}   \u{2191}\u{2193} ",
        Style::default().fg(THEME.text_dim),
    )));

    for (i, conn) in conns
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take((scroll_offset + visible).min(total) - scroll_offset)
    {
        let label = format!(
            "{} | {} conns | {}",
            conn.process_name,
            conn.connections.len(),
            conn.risk_level
        );
        let checked = if app.agents.selected_connection_idxs.contains(&i) {
            "[X]"
        } else {
            "[ ]"
        };
        let is_cursor = i == cursor_idx;
        if is_cursor {
            lines.push(Line::from(vec![
                Span::styled(" ▎", Style::default().fg(THEME.primary)),
                Span::styled(
                    format!(" {} {} ", checked, label),
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.primary)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        } else if app.agents.selected_connection_idxs.contains(&i) {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    format!(" {} {} ", checked, label),
                    Style::default()
                        .fg(THEME.primary)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    format!(" {} {} ", checked, label),
                    Style::default().fg(THEME.text_main),
                ),
            ]));
        }
    }

    if total > visible {
        let sb = if scroll_offset == 0 {
            "↑"
        } else if scroll_offset + visible >= total {
            "↓"
        } else {
            "↕"
        };
        lines.push(Line::from(Span::styled(
            format!(
                "   {}  {} ",
                sb,
                tr!(app.ui.translator, "agents.scroll_hint")
            ),
            Style::default().fg(THEME.text_dim),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.select_and_continue")),
            Style::default()
                .fg(THEME.background)
                .bg(THEME.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.cancel")),
            Style::default()
                .fg(THEME.background)
                .bg(THEME.danger)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![Span::styled(
        format!(
            "  {} / {}\u{00a0}\u{00a0}Space: toggle\u{00a0}\u{00a0}Enter: launch {} agent(s)",
            app.agents.selected_connection_idxs.len(),
            total,
            app.agents.selected_connection_idxs.len()
        ),
        Style::default().fg(THEME.text_dim),
    )]));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(THEME.warning))
        .title(format!(
            " {} ",
            tr!(app.ui.translator, "agents.select_network")
        ))
        .title_style(
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph.block(block), popup_area);
}
