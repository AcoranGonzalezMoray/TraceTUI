use super::theme::THEME;
use crate::app::types::{AgentMission, AgentStatus, NavView};
use crate::app::App;
use crate::config;
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

fn parse_inline_md(text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            let end = text[i + 2..].find("**");
            if let Some(pos) = end {
                let content = &text[i + 2..i + 2 + pos];
                if !content.is_empty() {
                    spans.push(Span::styled(
                        content.to_string(),
                        Style::default()
                            .fg(THEME.text_main)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                i += 2 + pos + 2;
            } else {
                let end2 = text[i + 2..].find(|c: char| !c.is_whitespace());
                if end2.map_or(true, |p| text.as_bytes().get(i + 2 + p).map_or(true, |&b| b as char == '*'))
                {
                    spans.push(Span::styled(
                        chars[i].to_string(),
                        Style::default().fg(THEME.text_main),
                    ));
                    i += 1;
                } else {
                    break;
                }
            }
        } else if chars[i] == '`' {
            let remaining = &text[i + 1..];
            if let Some(pos) = remaining.find('`') {
                let content = &remaining[..pos];
                spans.push(Span::styled(
                    content.to_string(),
                    Style::default().fg(THEME.success),
                ));
                i += 1 + pos + 1;
            } else {
                spans.push(Span::styled(
                    chars[i].to_string(),
                    Style::default().fg(THEME.text_main),
                ));
                i += 1;
            }
        } else {
            let mut end = i + 1;
            while end < len {
                if (chars[end] == '*' && end + 1 < len && chars[end + 1] == '*') || chars[end] == '`'
                {
                    break;
                }
                end += 1;
            }
            spans.push(Span::styled(
                text[i..end].to_string(),
                Style::default().fg(THEME.text_main),
            ));
            i = end;
        }
    }
    spans
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if text.len() <= width || width < 10 {
        return vec![text.to_string()];
    }
    let mut result = Vec::new();
    let mut start = 0;
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    while start < len {
        if start + width >= len {
            result.push(text[start..].to_string());
            break;
        }
        let end = start + width;
        if let Some(space) = text[start..end].rfind(' ') {
            let split = start + space;
            result.push(text[start..split].to_string());
            start = split + 1;
        } else {
            result.push(text[start..end].to_string());
            start = end;
        }
    }
    result
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
            spans.push(Span::styled(
                " ┃ ",
                Style::default().fg(THEME.text_dim),
            ));
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
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.chars().all(|c| c == '|' || c == '-' || c == ':' || c == ' ' || c == '\t')
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

        if line.starts_with("### ") {
            let content = &line[4..];
            lines.push(Line::from(Span::styled(
                format!(" {} ", content),
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            )));
            continue;
        }

        if line.starts_with("## ") {
            let content = &line[3..];
            lines.push(Line::from(Span::styled(
                format!(" {} ", content),
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            )));
            continue;
        }

        if line.starts_with("# ") {
            let content = &line[2..];
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
                lines.push(Line::from(Span::styled(sep, Style::default().fg(THEME.text_dim))));
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
                let mut spans = vec![Span::styled(
                    " • ",
                    Style::default().fg(THEME.primary),
                )];
                spans.extend(parse_inline_md(&wrapped));
                lines.push(Line::from(spans));
            }
            continue;
        }

        if line.starts_with(|c: char| c.is_ascii_digit()) && line.len() > 2 {
            let dot_pos = line.find('.').unwrap_or(1);
            if line.as_bytes().get(dot_pos) == Some(&b'.') && line.as_bytes().get(dot_pos + 1).map_or(false, |&b| b == b' ' || b == b'\t') {
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

fn running_phase_text(frame_count: u64, started_at: u64, translator: &crate::i18n::Translator) -> String {
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
    }
}

fn mission_label(mission: AgentMission) -> &'static str {
    match mission {
        AgentMission::ProcessAnalysis => "Process",
        AgentMission::NetworkAnalysis => "Network",
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

    if app.agents.agents.is_empty() {
        let msg = Paragraph::new(Line::from(vec![Span::styled(
            tr!(app.ui.translator, "agents.no_agents"),
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
        .flat_map(|_| vec![Constraint::Length(4), Constraint::Length(1)])
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
        let title = format!(
            " {} {}",
            icon,
            target
        );

        let subtitle = format!(
            " {} [{}]",
            mission_icon(agent.mission),
            agent.model
        );

        let lines = vec![
            Line::from(Span::styled(title, style.fg(color))),
            Line::from(Span::styled(
                subtitle,
                Style::default().fg(THEME.text_dim),
            )),
            Line::from(Span::styled(
                match &agent.status {
                    AgentStatus::Idle => tr!(app.ui.translator, "agents.status_idle"),
                    AgentStatus::Queued => tr!(app.ui.translator, "agents.status_queued"),
                    AgentStatus::Running(_) => tr!(app.ui.translator, "agents.status_running"),
                    AgentStatus::Completed(_) => tr!(app.ui.translator, "agents.status_done"),
                    AgentStatus::Failed(msg) => {
                        let first = msg.lines().next().unwrap_or(msg);
                        if first.len() > 45 {
                            format!("{}...", &first[..45])
                        } else {
                            first.to_string()
                        }
                    }
                },
                Style::default().fg(THEME.text_dim),
            )),
        ];


        let agent_block = if is_selected {
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(THEME.primary))
        } else {
            Block::default().padding(ratatui::widgets::Padding::new(1, 1, 0, 0))
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
        .title(format!(" {} ", tr!(app.ui.translator, "agents.detail")))
        .title_style(
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let selected = app.agents.selected_agent_index;
    if selected >= app.agents.agents.len() {
        let msg = Paragraph::new(Line::from(vec![Span::styled(
            tr!(app.ui.translator, "agents.select_hint"),
            Style::default().fg(THEME.text_dim),
        )]))
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
    lines.push(Line::from(""));

    if !agent.target_name.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {}: ", tr!(app.ui.translator, "agents.target")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                &agent.target_name,
                Style::default().fg(THEME.text_main).add_modifier(Modifier::BOLD),
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
        Span::styled(&agent.model, Style::default().fg(THEME.text_main)),
    ]));
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
        AgentStatus::Running(_) => {
            let phase = running_phase_text(app.ui.frame_count, agent.started_at_frame, &app.ui.translator);
            let inner_w = inner.width as usize;
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
                " ".repeat(inner_w.saturating_sub(20) / 2) + &tr!(app.ui.translator, "agents.please_wait"),
                Style::default().fg(THEME.text_dim),
            )));
        }
        AgentStatus::Completed(msg) | AgentStatus::Failed(msg) => {
            let sep = Line::from(Span::styled(
                "─".repeat(inner.width.saturating_sub(2) as usize),
                Style::default().fg(THEME.text_dim),
            ));
            lines.push(sep);
            lines.push(Line::from(""));

            let scroll = app.agents.agent_detail_scroll;
            let md_lines = markdown_to_lines(msg, inner.width as usize);
            let available = (inner.height as usize).saturating_sub(10);
            let end = (scroll + available).min(md_lines.len());
            for line in md_lines.iter().skip(scroll).take(end.saturating_sub(scroll)) {
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
        .title(format!(" {} ", tr!(app.ui.translator, "actions.title")))
        .title_style(
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let action_items = vec![
        ("󰈐", tr!(app.ui.translator, "agents.configure_provider")),
        ("󰚩", tr!(app.ui.translator, "agents.launch_agent")),
        ("✕", tr!(app.ui.translator, "agents.clear_results")),
    ];

    let constraints: Vec<Constraint> = action_items
        .iter()
        .flat_map(|_| vec![Constraint::Length(3), Constraint::Length(1)])
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, (icon, label)) in action_items.iter().enumerate() {
        if i * 2 >= chunks.len() {
            break;
        }
        let is_selected = i == app.agents.agent_action_index;
        let area = chunks[i * 2];

        let style = if is_selected {
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(THEME.text_dim)
        };

        let action_block = if is_selected {
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(THEME.primary))
        } else {
            Block::default().padding(ratatui::widgets::Padding::new(1, 1, 1, 1))
        };

        let content = Paragraph::new(Line::from(vec![
            Span::styled(format!(" {} ", icon), style),
            Span::styled(label, style),
        ]))
        .block(action_block);

        f.render_widget(content, area);
    }
}

pub fn render_provider_modal(f: &mut ratatui::Frame, app: &App) {
    let popup_area = Rect {
        x: (f.area().width / 5),
        y: (f.area().height / 5),
        width: f.area().width * 3 / 5,
        height: f.area().height * 3 / 5,
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
            tr!(app.ui.translator, "agents.ollama_config"),
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled(
            format!("  {}: ", tr!(app.ui.translator, "agents.provider_url")),
            if has_focus == 0 {
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
            if has_focus == 0 { cursor } else { " " },
            Style::default().fg(THEME.primary),
        ),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled(
            format!("  {}: ", tr!(app.ui.translator, "agents.models_label")),
            if has_focus == 1 {
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
            if has_focus == 1 { cursor } else { " " },
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
    ]));

    let visible_models: usize = (popup_area.height as usize).saturating_sub(14);
    for (i, model) in app.agents.ollama_models.iter().enumerate() {
        if i >= visible_models {
            break;
        }
        let is_selected = i == app.agents.selected_model_index;
        let is_focused = has_focus == 2 && is_selected;
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
                    Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD),
                ),
            ])
        } else {
            Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    format!(" {} ", model),
                    Style::default().fg(THEME.text_main),
                ),
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
                .bg(if has_focus == 3 { THEME.primary } else { THEME.secondary })
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.save")),
            Style::default()
                .fg(THEME.background)
                .bg(if has_focus == 4 { THEME.primary } else { THEME.success })
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.cancel")),
            Style::default()
                .fg(THEME.background)
                .bg(if has_focus == 5 { THEME.primary } else { THEME.danger })
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![Span::styled(
        format!("  {}: Tab/Shift+Tab", tr!(app.ui.translator, "agents.switch_field")),
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
    let popup_height = 10;
    let popup_width = 50;
    let popup_area = Rect {
        x: (f.area().width.saturating_sub(popup_width)) / 2,
        y: (f.area().height.saturating_sub(popup_height)) / 2,
        width: popup_width.min(f.area().width),
        height: popup_height.min(f.area().height),
    };

    let items = [
        ("󰆧", tr!(app.ui.translator, "agents.launch_process")),
        ("󰛳", tr!(app.ui.translator, "agents.launch_network")),
    ];

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {} ", tr!(app.ui.translator, "agents.select_type")),
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    for (i, (icon, label)) in items.iter().enumerate() {
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
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    format!(" {} {} ", icon, label),
                    Style::default().fg(THEME.text_main),
                ),
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
    let scroll_offset = if cursor_idx >= visible { cursor_idx - visible + 1 } else { 0 };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {} ({}) ", tr!(app.ui.translator, "agents.select_process"), total),
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(Span::styled(
        "  [Space] \u{2713} / \u{2717}   \u{2191}\u{2193} ",
        Style::default().fg(THEME.text_dim),
    )));

    for i in scroll_offset..(scroll_offset + visible).min(total) {
        let proc = &processes[i];
        let path = proc.path.as_deref().unwrap_or("N/A");
        let label = format!("{} | {} (PID: {})", proc.name, path, proc.pid);
        let checked = if app.agents.selected_pids.contains(&proc.pid) { "[X]" } else { "[ ]" };
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
                    Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD),
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
        let sb = if scroll_offset == 0 { "↑" } else if scroll_offset + visible >= total { "↓" } else { "↕" };
        lines.push(Line::from(Span::styled(
            format!("   {}  {} ", sb, tr!(app.ui.translator, "agents.scroll_hint")),
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
        format!("  {} / {}\u{00a0}\u{00a0}Space: toggle\u{00a0}\u{00a0}Enter: launch {} agent(s)",
            app.agents.selected_pids.len(), total, app.agents.selected_pids.len()),
        Style::default().fg(THEME.text_dim),
    )]));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(THEME.warning))
        .title(format!(" {} ", tr!(app.ui.translator, "agents.select_process")))
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
    let scroll_offset = if cursor_idx >= visible { cursor_idx - visible + 1 } else { 0 };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {} ({}) ", tr!(app.ui.translator, "agents.select_network"), total),
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(Span::styled(
        "  [Space] \u{2713} / \u{2717}   \u{2191}\u{2193} ",
        Style::default().fg(THEME.text_dim),
    )));

    for i in scroll_offset..(scroll_offset + visible).min(total) {
        let conn = &conns[i];
        let label = format!("{} | {} conns | {}", conn.process_name, conn.connections.len(), conn.risk_level);
        let checked = if app.agents.selected_connection_idxs.contains(&i) { "[X]" } else { "[ ]" };
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
                    Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD),
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
        let sb = if scroll_offset == 0 { "↑" } else if scroll_offset + visible >= total { "↓" } else { "↕" };
        lines.push(Line::from(Span::styled(
            format!("   {}  {} ", sb, tr!(app.ui.translator, "agents.scroll_hint")),
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
        format!("  {} / {}\u{00a0}\u{00a0}Space: toggle\u{00a0}\u{00a0}Enter: launch {} agent(s)",
            app.agents.selected_connection_idxs.len(), total, app.agents.selected_connection_idxs.len()),
        Style::default().fg(THEME.text_dim),
    )]));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(THEME.warning))
        .title(format!(" {} ", tr!(app.ui.translator, "agents.select_network")))
        .title_style(
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph.block(block), popup_area);
}
