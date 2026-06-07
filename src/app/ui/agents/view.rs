use super::constants::{
    AGENT_TYPE_SELECTOR_HEIGHT, AGENT_TYPE_SELECTOR_WIDTH, ELAPSED_TIME_DECIMAL_DIVISOR,
    FAILED_STATUS_MAX_CHARS, FRAME_COUNT_MS, MD_SCROLL_DOWN_INDICATOR_PREFIX,
    MD_SCROLL_DOWN_INDICATOR_SUFFIX, NETWORK_SELECTOR_CHECKED, NETWORK_SELECTOR_HEIGHT_DIVISOR,
    NETWORK_SELECTOR_RESERVED_ROWS, NETWORK_SELECTOR_UNCHECKED, NETWORK_SELECTOR_WIDTH_DIVISOR,
    PROCESS_SELECTOR_CHECKED, PROCESS_SELECTOR_HEIGHT_DIVISOR, PROCESS_SELECTOR_RESERVED_ROWS,
    PROCESS_SELECTOR_UNCHECKED, PROCESS_SELECTOR_WIDTH_DIVISOR, PROGRESS_BAR_EMPTY,
    PROGRESS_BAR_FULL, PROGRESS_BAR_TOTAL, RUNNING_ETA_BASE_SECS, RUNNING_PROGRESS_DIVISOR,
    RUNNING_PROGRESS_MAX, RUNNING_PROGRESS_MIN_INITIAL, RUNNING_PROGRESS_MIN_WITH_MSG,
    RUNNING_PROGRESS_TIME_DIVISOR, SCROLL_INDICATOR_BOTH, SCROLL_INDICATOR_DOWN,
    SCROLL_INDICATOR_UP, SELECTED_ROW_INDICATOR, SELECTOR_HINT_TEXT, SELECTOR_TOGGLE_HINT,
    UNSELECTED_ROW_INDICATOR,
};
use super::icons::{
    action_cancel_icon, cursor_for_frame, key_hint, ICON_ACTIONS_TITLE, ICON_ACTION_COLLAPSE,
    ICON_ACTION_EXPAND, ICON_ACTION_EXPORT_JSON, ICON_ACTION_EXPORT_MD, ICON_ACTION_FILTER,
    ICON_ACTION_RETRY, ICON_AGENTS_TITLE, ICON_CLEAR, ICON_DETAIL_TITLE, ICON_LAUNCH,
    ICON_PARALLEL, ICON_PROVIDER, STATUS_DONE, STATUS_FAILED, STATUS_IDLE, STATUS_QUEUED,
};
use super::markdown;
use super::widgets::{action_button, phase_for_frame, spinner_for_frame, status_badge};
use crate::app::agents::constants::NO_PROCESS_NAME_PLACEHOLDER;
use crate::app::agents::mission::all_missions;
use crate::app::types::{AgentMission, AgentStatus, NavView};
use crate::app::ui::theme::THEME;
use crate::app::{App, SidebarFocus};
use crate::config;
use crate::tr;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::Frame;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
};

pub fn render_agents_view(f: &mut Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(config::SIDEBAR_LEFT_PCT),
            Constraint::Percentage(config::CENTER_PANEL_PCT),
            Constraint::Percentage(config::SIDEBAR_RIGHT_PCT),
        ])
        .split(area);

    render_left(f, app, columns[0]);
    render_center(f, app, columns[1]);
    render_right(f, app, columns[2]);
}

pub fn matching_agent_indices(app: &App) -> Vec<usize> {
    if !app.ui.search_mode || app.ui.search_query.is_empty() {
        return (0..app.agents.agents.len()).collect();
    }
    let q = app.ui.search_query.to_lowercase();
    app.agents
        .agents
        .iter()
        .enumerate()
        .filter_map(|(i, agent)| {
            let target_match = agent.target_name.to_lowercase().contains(&q);
            let provider_match = agent.provider.label().to_lowercase().contains(&q);
            let model_match = agent.model.to_lowercase().contains(&q);
            let report_match = match &agent.status {
                AgentStatus::Running(t) | AgentStatus::Completed(t) | AgentStatus::Failed(t) => {
                    t.to_lowercase().contains(&q)
                }
                _ => false,
            };
            if target_match || provider_match || model_match || report_match {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

fn render_left(f: &mut Frame, app: &App, area: Rect) {
    let is_focused =
        app.ui.sidebar_focus == SidebarFocus::Left && app.ui.current_nav_view == NavView::Agents;
    let block = focused_block(
        is_focused,
        format!(
            " {} {} ",
            ICON_AGENTS_TITLE,
            tr!(app.ui.translator, "agents.report_history")
        ),
    );

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    if app.agents.history_loading {
        render_loading_placeholder(f, app, inner);
        return;
    }

    let matching = matching_agent_indices(app);
    if matching.is_empty() {
        render_empty_history(f, app, inner);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(config::SCROLLBAR_WIDTH),
        ])
        .split(inner);
    let list_area = chunks[0];
    let scrollbar_area = chunks[1];

    let agent_h = 5usize;
    let visible_max = (list_area.height as usize).saturating_sub(1) / agent_h;
    let total = matching.len();
    let sel_pos = matching
        .iter()
        .position(|&i| i == app.agents.selected_agent_index)
        .unwrap_or(0);
    let scroll = sel_pos.saturating_sub(visible_max.saturating_sub(1));
    let visible_count = visible_max.min(total.saturating_sub(scroll));

    let rows = build_agent_rows(list_area, visible_count);
    for offset in 0..visible_count {
        let i = matching[scroll + offset];
        let agent = &app.agents.agents[i];
        let is_selected = i == app.agents.selected_agent_index;
        let row_area = rows[offset * 2];
        if row_area.height == 0 {
            continue;
        }
        f.render_widget(
            Paragraph::new(agent_card_lines(app, agent, is_selected))
                .block(agent_row_block(is_selected)),
            row_area,
        );
    }

    if total > visible_max {
        crate::app::ui::widgets::render_scrollbar(f, scrollbar_area, total, scroll);
    }
}

fn render_center(f: &mut Frame, app: &App, area: Rect) {
    let is_focused =
        app.ui.sidebar_focus == SidebarFocus::Center && app.ui.current_nav_view == NavView::Agents;
    let title = format!(
        " {} {}  ·  {}  ·  {} {} ",
        ICON_DETAIL_TITLE,
        tr!(app.ui.translator, "agents.report_detail"),
        app.agents.agents.len(),
        app.agents.running_agent_count,
        tr!(app.ui.translator, "agents.running_short"),
    );
    let block = focused_block(is_focused, title);
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    if app.agents.history_loading {
        render_loading_placeholder(f, app, inner);
        return;
    }
    if app.agents.agents.is_empty() {
        render_empty_history_with_hint(f, app, inner);
        return;
    }
    let selected = app.agents.selected_agent_index;
    if selected >= app.agents.agents.len() {
        render_select_hint(f, app, inner);
        return;
    }
    if app.ui.search_mode && !app.ui.search_query.is_empty() {
        let matching = matching_agent_indices(app);
        if !matching.contains(&selected) {
            render_no_match_hint(f, app, inner);
            return;
        }
    }

    let agent = &app.agents.agents[selected];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(config::SCROLLBAR_WIDTH),
        ])
        .split(inner);
    let content_area = chunks[0];
    let scrollbar_area = chunks[1];

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(content_area);
    let header_area = v_chunks[0];
    let actions_area = v_chunks[1];
    let report_area = v_chunks[2];

    let mut header_lines: Vec<Line> = Vec::new();
    header_lines.push(header_line(agent));
    header_lines.push(status_line(app, agent));
    append_metadata_lines(&mut header_lines, app, agent);

    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.secondary));
    f.render_widget(
        Paragraph::new(header_lines).block(header_block),
        header_area,
    );

    let actions_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.secondary));
    let actions_content = vec![actions_line(
        app,
        agent,
        actions_area.width.saturating_sub(2),
    )];
    f.render_widget(
        Paragraph::new(actions_content).block(actions_block),
        actions_area,
    );

    let report_total = match &agent.status {
        AgentStatus::Running(msg) | AgentStatus::Completed(msg) | AgentStatus::Failed(msg) => {
            if msg.trim().is_empty() {
                None
            } else {
                let md_lines =
                    filtered_md_lines(app, msg, report_area.width.saturating_sub(2) as usize);
                Some(md_lines.len())
            }
        }
        _ => None,
    };

    let (report_total, report_visible) = match report_total {
        Some(total) => {
            let reserved = match &agent.status {
                AgentStatus::Running(_) => 6,
                _ => 2,
            };
            let visible = (report_area.height as usize).saturating_sub(reserved);
            (total, visible)
        }
        None => (0, 0),
    };

    let report_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.secondary))
        .title(format!(
            " {} ",
            tr!(app.ui.translator, "agents.report_detail")
        ))
        .title_style(Style::default().fg(THEME.text_dim));

    let mut report_lines: Vec<Line> = Vec::new();
    append_report_lines(
        &mut report_lines,
        app,
        agent,
        report_block.inner(report_area),
    );
    f.render_widget(
        Paragraph::new(report_lines).block(report_block),
        report_area,
    );

    if report_total > report_visible {
        let mut scrollbar_rect = scrollbar_area;
        scrollbar_rect.y = report_area.y;
        scrollbar_rect.height = report_area.height;
        crate::app::ui::widgets::render_scrollbar(
            f,
            scrollbar_rect,
            report_total,
            app.agents.agent_detail_scroll,
        );
    }

    if app.agents.agent_search_mode {
        render_search_overlay(f, app, inner);
    }
}

fn render_right(f: &mut Frame, app: &App, area: Rect) {
    let is_focused =
        app.ui.sidebar_focus == SidebarFocus::Right && app.ui.current_nav_view == NavView::Agents;
    let title = format!(
        " {} {} ",
        ICON_ACTIONS_TITLE,
        tr!(app.ui.translator, "actions.title")
    );
    let block = focused_block(is_focused, title);
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let items = build_action_items(app);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(config::SCROLLBAR_WIDTH),
        ])
        .split(inner);
    let list_area = chunks[0];
    let scrollbar_area = chunks[1];

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| action_item_view(item, i == app.agents.agent_action_index))
        .collect();
    let mut list_state = ListState::default();
    list_state.select(Some(app.agents.agent_action_index));
    f.render_stateful_widget(
        List::new(list_items).block(Block::default()),
        list_area,
        &mut list_state,
    );
    crate::app::ui::widgets::render_scrollbar(
        f,
        scrollbar_area,
        items.len(),
        app.agents.agent_action_index,
    );
}

struct ActionItem {
    icon: &'static str,
    title: String,
    key: String,
    color: Color,
}

fn build_action_items(app: &App) -> Vec<ActionItem> {
    let t = &app.ui.translator;
    let mut items = vec![
        ActionItem {
            icon: ICON_PROVIDER,
            title: format!(
                "{} ({})",
                tr!(t, "agents.action_provider"),
                app.agents.ollama.provider.label()
            ),
            key: super::constants::KEY_PROVIDER_CYCLE.to_string(),
            color: THEME.secondary,
        },
        ActionItem {
            icon: ICON_LAUNCH,
            title: tr!(t, "agents.action_launch").to_string(),
            key: super::constants::KEY_LAUNCH.to_string(),
            color: THEME.primary,
        },
    ];

    if !app.agents.history_loading {
        let selected = app
            .agents
            .agents
            .get(app.agents.selected_agent_index)
            .map(|a| &a.status);
        match selected {
            Some(AgentStatus::Running(_)) | Some(AgentStatus::Queued) => items.push(ActionItem {
                icon: action_cancel_icon(),
                title: tr!(t, "agents.action_cancel").to_string(),
                key: super::constants::KEY_CANCEL.to_string(),
                color: THEME.danger,
            }),
            _ => items.push(ActionItem {
                icon: ICON_ACTION_RETRY,
                title: tr!(t, "agents.action_retry").to_string(),
                key: super::constants::KEY_RETRY.to_string(),
                color: THEME.warning,
            }),
        }
    }

    items.push(ActionItem {
        icon: ICON_PARALLEL,
        title: format!(
            "{}: {} ({})",
            tr!(t, "agents.parallel_label"),
            app.agents.max_parallel_agents,
            super::constants::KEY_PARALLEL
        ),
        key: super::constants::KEY_PARALLEL.to_string(),
        color: THEME.warning,
    });
    items.push(ActionItem {
        icon: ICON_CLEAR,
        title: tr!(t, "agents.action_clear").to_string(),
        key: super::constants::KEY_CLEAR.to_string(),
        color: THEME.danger,
    });
    items
}

fn action_item_view(item: &ActionItem, is_selected: bool) -> ListItem<'static> {
    let prefix = if is_selected {
        SELECTED_ROW_INDICATOR
    } else {
        UNSELECTED_ROW_INDICATOR
    };
    let indicator_style = if is_selected {
        Style::default().fg(THEME.primary)
    } else {
        Style::default()
    };
    let title_style = if is_selected {
        Style::default()
            .fg(THEME.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.text_main)
    };
    ListItem::new(vec![
        Line::from(vec![
            Span::styled(prefix, indicator_style),
            Span::styled(format!(" {} ", item.icon), Style::default().fg(item.color)),
            Span::styled(item.title.clone(), title_style),
        ]),
        Line::from(vec![
            Span::raw("   "),
            Span::styled(key_hint(&item.key), Style::default().fg(THEME.text_dim)),
        ]),
    ])
}

fn header_line(agent: &crate::app::types::AgentInstance) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!(" {} ", agent.mission.icon()),
            Style::default().fg(THEME.warning),
        ),
        Span::styled(
            format!(" {} ", agent.mission.short_label()),
            Style::default()
                .fg(THEME.text_main)
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

fn status_line(app: &App, agent: &crate::app::types::AgentInstance) -> Line<'static> {
    let color = status_color_for(&agent.status);
    let label = match &agent.status {
        AgentStatus::Idle => tr!(app.ui.translator, "agents.status_idle").to_string(),
        AgentStatus::Queued => tr!(app.ui.translator, "agents.status_queued").to_string(),
        AgentStatus::Running(_) => tr!(app.ui.translator, "agents.status_running").to_string(),
        AgentStatus::Completed(_) => tr!(app.ui.translator, "agents.status_done").to_string(),
        AgentStatus::Failed(_) => tr!(app.ui.translator, "agents.status_failed").to_string(),
    };
    Line::from(vec![
        status_badge(
            tr!(app.ui.translator, "agents.agent_status").to_string(),
            color,
        ),
        Span::raw(" "),
        Span::styled(
            label,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}

fn actions_line(
    app: &App,
    agent: &crate::app::types::AgentInstance,
    inner_width: u16,
) -> Line<'static> {
    let mut button_spans = Vec::new();
    match &agent.status {
        AgentStatus::Running(_) | AgentStatus::Queued => {
            button_spans.extend(action_button(
                action_cancel_icon(),
                tr!(app.ui.translator, "agents.action_cancel").to_string(),
                super::constants::KEY_CANCEL,
                THEME.danger,
            ));
        }
        AgentStatus::Completed(_) => {}
        _ => {
            button_spans.extend(action_button(
                ICON_ACTION_RETRY,
                tr!(app.ui.translator, "agents.action_retry").to_string(),
                super::constants::KEY_RETRY,
                THEME.warning,
            ));
        }
    }
    button_spans.extend(action_button(
        ICON_ACTION_EXPORT_MD,
        tr!(app.ui.translator, "agents.action_export_md").to_string(),
        super::constants::KEY_EXPORT_MD,
        THEME.secondary,
    ));
    button_spans.extend(action_button(
        ICON_ACTION_EXPORT_JSON,
        tr!(app.ui.translator, "agents.action_export_json").to_string(),
        super::constants::KEY_EXPORT_JSON,
        THEME.secondary,
    ));
    button_spans.extend(action_button(
        ICON_ACTION_FILTER,
        tr!(app.ui.translator, "agents.action_filter").to_string(),
        super::constants::KEY_FILTER,
        THEME.primary,
    ));
    let (icon, label) = if app.agents.collapse_sections {
        (
            ICON_ACTION_EXPAND,
            tr!(app.ui.translator, "agents.action_expand").to_string(),
        )
    } else {
        (
            ICON_ACTION_COLLAPSE,
            tr!(app.ui.translator, "agents.action_collapse").to_string(),
        )
    };
    button_spans.extend(action_button(
        icon,
        label,
        super::constants::KEY_COLLAPSE,
        THEME.warning,
    ));

    let btn_width: usize = button_spans.iter().map(|s| s.content.len()).sum();
    let inner_w = inner_width as usize;
    let pad = if inner_w > btn_width {
        (inner_w - btn_width) / 2
    } else {
        0
    };
    let mut centered = Vec::new();
    centered.push(Span::styled(" ".repeat(pad), Style::default()));
    centered.extend(button_spans);
    centered.push(Span::styled(" ".repeat(pad), Style::default()));
    Line::from(centered)
}

fn append_metadata_lines(
    lines: &mut Vec<Line<'static>>,
    app: &App,
    agent: &crate::app::types::AgentInstance,
) {
    if !agent.target_name.is_empty() {
        lines.push(label_value_line(
            tr!(app.ui.translator, "agents.target").to_string(),
            agent.target_name.clone(),
        ));
    }
    if let Some(ref path) = agent.target_path {
        lines.push(label_value_line(
            tr!(app.ui.translator, "agents.path").to_string(),
            path.clone(),
        ));
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
        lines.push(label_value_line(
            tr!(app.ui.translator, "agents.history").to_string(),
            path.clone(),
        ));
    }
    if let Some(end_frame) = agent.completed_at_frame {
        let elapsed_ms = end_frame.saturating_sub(agent.started_at_frame) * FRAME_COUNT_MS;
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {}: ", tr!(app.ui.translator, "agents.time")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                format!(
                    "{}.{:01}s",
                    elapsed_ms / 1000,
                    (elapsed_ms % 1000) / ELAPSED_TIME_DECIMAL_DIVISOR
                ),
                Style::default().fg(THEME.text_main),
            ),
        ]));
    }
}

fn label_value_line(label: String, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!(" {}: ", label), Style::default().fg(THEME.text_dim)),
        Span::styled(value, Style::default().fg(THEME.text_main)),
    ])
}

fn append_report_lines(
    lines: &mut Vec<Line<'static>>,
    app: &App,
    agent: &crate::app::types::AgentInstance,
    inner: Rect,
) {
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
            push_running_state(lines, app, agent, msg, inner);
        }
        AgentStatus::Completed(msg) | AgentStatus::Failed(msg) => {
            push_completed_state(lines, app, msg, inner);
        }
    }
}

fn push_running_state(
    lines: &mut Vec<Line<'static>>,
    app: &App,
    agent: &crate::app::types::AgentInstance,
    msg: &str,
    inner: Rect,
) {
    let phase = phase_for_frame(
        app.ui.frame_count,
        agent.started_at_frame,
        &app.ui.translator,
    );
    let inner_w = inner.width as usize;
    let elapsed = app.ui.frame_count.saturating_sub(agent.started_at_frame);
    let progress = ((elapsed as usize * 2).min(RUNNING_PROGRESS_MAX)).max(if msg.is_empty() {
        RUNNING_PROGRESS_MIN_INITIAL
    } else {
        RUNNING_PROGRESS_MIN_WITH_MSG
    });
    let eta_secs = RUNNING_ETA_BASE_SECS
        .saturating_sub(elapsed as usize / RUNNING_PROGRESS_TIME_DIVISOR as usize);
    let filled = progress / RUNNING_PROGRESS_DIVISOR;
    let bar = format!(
        "[{}{}] {}% ~{}s",
        PROGRESS_BAR_FULL.repeat(filled),
        PROGRESS_BAR_EMPTY.repeat(PROGRESS_BAR_TOTAL.saturating_sub(filled)),
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
        let md_lines = filtered_md_lines(app, msg, inner.width as usize);
        let available = (inner.height as usize).saturating_sub(4);
        for line in md_lines.iter().skip(scroll).take(available) {
            lines.push(line.clone());
        }
    }
}

fn push_completed_state(lines: &mut Vec<Line<'static>>, app: &App, msg: &str, inner: Rect) {
    let scroll = app.agents.agent_detail_scroll;
    let md_lines = filtered_md_lines(app, msg, inner.width as usize);
    let available = inner.height as usize;
    let end = (scroll + available).min(md_lines.len());
    for line in md_lines
        .iter()
        .skip(scroll)
        .take(end.saturating_sub(scroll))
    {
        lines.push(line.clone());
    }
    if md_lines.len() > end {
        let extra = md_lines.len() - end;
        lines.push(Line::from(Span::styled(
            format!(
                "{}{}{}",
                MD_SCROLL_DOWN_INDICATOR_PREFIX, extra, MD_SCROLL_DOWN_INDICATOR_SUFFIX
            ),
            Style::default().fg(THEME.text_dim),
        )));
    }
}

fn filtered_md_lines(app: &App, text: &str, width: usize) -> Vec<Line<'static>> {
    let source = markdown::collapse_lines(
        text,
        app.agents.collapse_sections,
        &app.agents.agent_search_query,
    );
    markdown::render_to_lines(&source.join("\n"), width)
}

fn render_empty_history(f: &mut Frame, app: &App, inner: Rect) {
    let msg = Paragraph::new(Line::from(vec![Span::styled(
        tr!(app.ui.translator, "agents.no_history"),
        Style::default().fg(THEME.text_dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(msg, inner);
}

fn render_empty_history_with_hint(f: &mut Frame, app: &App, inner: Rect) {
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
}

fn render_select_hint(f: &mut Frame, app: &App, inner: Rect) {
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
}

fn render_no_match_hint(f: &mut Frame, app: &App, inner: Rect) {
    let msg = Paragraph::new(Line::from(Span::styled(
        tr!(app.ui.translator, "agents.search_no_match"),
        Style::default().fg(THEME.text_dim),
    )))
    .alignment(Alignment::Center);
    f.render_widget(msg, inner);
}

fn render_loading_placeholder(f: &mut Frame, app: &App, inner: Rect) {
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
}

fn agent_card_lines(
    app: &App,
    agent: &crate::app::types::AgentInstance,
    is_selected: bool,
) -> Vec<Line<'static>> {
    let style = if is_selected {
        Style::default()
            .fg(THEME.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.text_dim)
    };
    let icon = status_icon_for(&agent.status, app.ui.frame_count);
    let color = status_color_for(&agent.status);
    let target = if agent.target_name.is_empty() {
        agent.mission.short_label()
    } else {
        &agent.target_name
    };
    let title = format!(" {} {}", icon, target);

    let subtitle = format!(
        " {} {} [{}]",
        agent.mission.icon(),
        agent.provider.label(),
        agent.model
    );

    vec![
        Line::from(Span::styled(title, style.fg(color))),
        Line::from(Span::styled(subtitle, Style::default().fg(THEME.text_dim))),
        Line::from(Span::styled(
            format!("  {}", agent_status_text(app, agent)),
            Style::default().fg(THEME.text_dim),
        )),
    ]
}

fn agent_status_text(app: &App, agent: &crate::app::types::AgentInstance) -> String {
    match &agent.status {
        AgentStatus::Idle => tr!(app.ui.translator, "agents.status_idle").to_string(),
        AgentStatus::Queued => tr!(app.ui.translator, "agents.status_queued").to_string(),
        AgentStatus::Running(_) => tr!(app.ui.translator, "agents.status_running").to_string(),
        AgentStatus::Completed(_) => tr!(app.ui.translator, "agents.status_done").to_string(),
        AgentStatus::Failed(msg) => {
            let first = msg.lines().next().unwrap_or(msg);
            markdown::truncate_chars(first, FAILED_STATUS_MAX_CHARS)
        }
    }
}

fn agent_row_block(is_selected: bool) -> Block<'static> {
    if is_selected {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.primary))
    } else {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.background))
    }
}

fn build_agent_rows(inner: Rect, count: usize) -> Vec<Rect> {
    let constraints: Vec<Constraint> = (0..count)
        .flat_map(|_| [Constraint::Length(5), Constraint::Length(0)])
        .collect();
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner)
        .to_vec()
}

fn render_search_overlay(f: &mut Frame, app: &App, area: Rect) {
    let overlay_height = 3;
    let overlay = Rect {
        x: area.x + 2,
        y: area.y,
        width: area.width.saturating_sub(4),
        height: overlay_height,
    };
    f.render_widget(Clear, overlay);

    let cursor = cursor_for_frame(app.ui.frame_count);
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
            "  ",
            Style::default().fg(THEME.background).bg(THEME.primary),
        ),
        Span::styled(
            format!(" {} ", ICON_ACTION_FILTER),
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

fn focused_block(is_focused: bool, title: String) -> Block<'static> {
    let border_color = border_color_for(is_focused);
    let border_type = if is_focused {
        BorderType::Thick
    } else {
        BorderType::Rounded
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(border_type)
        .title(title)
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
}

fn border_color_for(is_focused: bool) -> Color {
    if is_focused {
        THEME.primary
    } else {
        THEME.secondary
    }
}

pub fn status_icon_for(status: &AgentStatus, frame_count: u64) -> &'static str {
    match status {
        AgentStatus::Idle => STATUS_IDLE,
        AgentStatus::Queued => STATUS_QUEUED,
        AgentStatus::Running(_) => spinner_for_frame(frame_count),
        AgentStatus::Completed(_) => STATUS_DONE,
        AgentStatus::Failed(_) => STATUS_FAILED,
    }
}

pub fn status_color_for(status: &AgentStatus) -> Color {
    match status {
        AgentStatus::Idle | AgentStatus::Queued => THEME.text_dim,
        AgentStatus::Running(_) => THEME.warning,
        AgentStatus::Completed(_) => THEME.success,
        AgentStatus::Failed(_) => THEME.danger,
    }
}

pub fn render_provider_modal(f: &mut Frame, app: &App) {
    use super::constants::{
        PROVIDER_MODAL_API_KEY_MASK_CHAR, PROVIDER_MODAL_API_KEY_MAX_MASK,
        PROVIDER_MODAL_FOCUS_API_KEY, PROVIDER_MODAL_FOCUS_CANCEL, PROVIDER_MODAL_FOCUS_FETCH,
        PROVIDER_MODAL_FOCUS_MODELS, PROVIDER_MODAL_FOCUS_MODEL_INPUT,
        PROVIDER_MODAL_FOCUS_PROVIDER, PROVIDER_MODAL_FOCUS_SAVE, PROVIDER_MODAL_FOCUS_URL,
        PROVIDER_MODAL_HEIGHT_DIVISOR, PROVIDER_MODAL_HEIGHT_MIN, PROVIDER_MODAL_HEIGHT_NUMERATOR,
        PROVIDER_MODAL_MODEL_LIST_RESERVED_ROWS, PROVIDER_MODAL_WIDTH_DIVISOR,
        PROVIDER_MODAL_WIDTH_MIN, PROVIDER_MODAL_WIDTH_NUMERATOR,
    };

    let pw = (f.area().width * PROVIDER_MODAL_WIDTH_NUMERATOR / PROVIDER_MODAL_WIDTH_DIVISOR)
        .max(PROVIDER_MODAL_WIDTH_MIN)
        .min(f.area().width);
    let ph = (f.area().height * PROVIDER_MODAL_HEIGHT_NUMERATOR / PROVIDER_MODAL_HEIGHT_DIVISOR)
        .max(PROVIDER_MODAL_HEIGHT_MIN)
        .min(f.area().height);
    let popup_area = Rect {
        x: (f.area().width.saturating_sub(pw)) / 2,
        y: (f.area().height.saturating_sub(ph)) / 2,
        width: pw,
        height: ph,
    };

    let has_focus = app.agents.provider_modal_focus;
    let cursor = cursor_for_frame(app.ui.frame_count);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default().fg(THEME.warning)),
        Span::styled(ICON_PROVIDER, Style::default().fg(THEME.warning)),
        Span::styled(
            format!(
                " {}: {}",
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
            focus_style(has_focus == PROVIDER_MODAL_FOCUS_PROVIDER),
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
            focus_style(has_focus == PROVIDER_MODAL_FOCUS_URL),
        ),
        Span::styled(
            &app.agents.ollama_url_input,
            Style::default().fg(THEME.text_main),
        ),
        Span::styled(
            cursor_indicator(has_focus == PROVIDER_MODAL_FOCUS_URL, cursor),
            Style::default().fg(THEME.primary),
        ),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled(
            format!("  {}: ", tr!(app.ui.translator, "agents.models_label")),
            focus_style(has_focus == PROVIDER_MODAL_FOCUS_MODEL_INPUT),
        ),
        Span::styled(
            &app.agents.ollama_model_input,
            Style::default().fg(THEME.text_main),
        ),
        Span::styled(
            cursor_indicator(has_focus == PROVIDER_MODAL_FOCUS_MODEL_INPUT, cursor),
            Style::default().fg(THEME.primary),
        ),
    ]));
    lines.push(Line::from(""));

    let masked_key = if app.agents.agent_api_key_input.is_empty() {
        String::new()
    } else {
        PROVIDER_MODAL_API_KEY_MASK_CHAR.repeat(
            app.agents
                .agent_api_key_input
                .len()
                .min(PROVIDER_MODAL_API_KEY_MAX_MASK),
        )
    };
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {}: ", tr!(app.ui.translator, "agents.api_key")),
            focus_style(has_focus == PROVIDER_MODAL_FOCUS_API_KEY),
        ),
        Span::styled(masked_key, Style::default().fg(THEME.text_main)),
        Span::styled(
            cursor_indicator(has_focus == PROVIDER_MODAL_FOCUS_API_KEY, cursor),
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

    let visible_models: usize =
        (popup_area.height as usize).saturating_sub(PROVIDER_MODAL_MODEL_LIST_RESERVED_ROWS);
    for (i, model) in app.agents.ollama_models.iter().enumerate() {
        if i >= visible_models {
            break;
        }
        let is_selected = i == app.agents.selected_model_index;
        let is_focused = has_focus == PROVIDER_MODAL_FOCUS_MODELS && is_selected;
        lines.push(model_line(model, is_selected, is_focused));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.fetch_models")),
            Style::default()
                .fg(THEME.background)
                .bg(focus_color(
                    has_focus == PROVIDER_MODAL_FOCUS_FETCH,
                    THEME.primary,
                    THEME.secondary,
                ))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.save")),
            Style::default()
                .fg(THEME.background)
                .bg(focus_color(
                    has_focus == PROVIDER_MODAL_FOCUS_SAVE,
                    THEME.primary,
                    THEME.success,
                ))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!(" {} ", tr!(app.ui.translator, "agents.cancel")),
            Style::default()
                .fg(THEME.background)
                .bg(focus_color(
                    has_focus == PROVIDER_MODAL_FOCUS_CANCEL,
                    THEME.primary,
                    THEME.danger,
                ))
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

fn focus_style(active: bool) -> Style {
    if active {
        Style::default()
            .fg(THEME.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.text_dim)
    }
}

fn focus_color(active: bool, focused: Color, blurred: Color) -> Color {
    if active {
        focused
    } else {
        blurred
    }
}

fn cursor_indicator(active: bool, cursor: &'static str) -> &'static str {
    if active {
        cursor
    } else {
        " "
    }
}

fn model_line(model: &str, is_selected: bool, is_focused: bool) -> Line<'static> {
    if is_focused {
        Line::from(vec![
            Span::styled(SELECTED_ROW_INDICATOR, Style::default().fg(THEME.primary)),
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
            Span::styled(SELECTED_ROW_INDICATOR, Style::default().fg(THEME.primary)),
            Span::styled(
                format!(" {} ", model),
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![
            Span::raw(UNSELECTED_ROW_INDICATOR),
            Span::styled(format!(" {} ", model), Style::default().fg(THEME.text_main)),
        ])
    }
}

pub fn render_agent_type_selector(f: &mut Frame, app: &App) {
    let popup_area = centered_rect(
        f.area(),
        AGENT_TYPE_SELECTOR_WIDTH,
        AGENT_TYPE_SELECTOR_HEIGHT,
    );

    let items = build_type_selector_items(app);
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
        if i == app.agents.agent_type_selector_index {
            lines.push(Line::from(vec![
                Span::styled(SELECTED_ROW_INDICATOR, Style::default().fg(THEME.primary)),
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
                Span::raw(UNSELECTED_ROW_INDICATOR),
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

fn build_type_selector_items(app: &App) -> Vec<(&'static str, String, String)> {
    let t = &app.ui.translator;
    all_missions()
        .iter()
        .map(|m| {
            let label = match m {
                AgentMission::ProcessAnalysis => tr!(t, "agents.launch_process").to_string(),
                AgentMission::NetworkAnalysis => tr!(t, "agents.launch_network").to_string(),
                AgentMission::DnsAnalysis => tr!(t, "agents.agent_dns").to_string(),
                AgentMission::FileAnalyzer => tr!(t, "agents.agent_file").to_string(),
                AgentMission::PortScanner => tr!(t, "agents.agent_ports").to_string(),
                AgentMission::LogAnalyzer => tr!(t, "agents.agent_logs").to_string(),
                AgentMission::MemoryAnalyzer => tr!(t, "agents.agent_memory").to_string(),
                AgentMission::VulnerabilityCheck => tr!(t, "agents.agent_vuln").to_string(),
                AgentMission::ThreatIntel => tr!(t, "agents.agent_intel").to_string(),
            };
            let desc = match m {
                AgentMission::ProcessAnalysis => tr!(t, "agents.agent_process_desc").to_string(),
                AgentMission::NetworkAnalysis => tr!(t, "agents.agent_network_desc").to_string(),
                AgentMission::DnsAnalysis => tr!(t, "agents.agent_dns_desc").to_string(),
                AgentMission::FileAnalyzer => tr!(t, "agents.agent_file_desc").to_string(),
                AgentMission::PortScanner => tr!(t, "agents.agent_ports_desc").to_string(),
                AgentMission::LogAnalyzer => tr!(t, "agents.agent_logs_desc").to_string(),
                AgentMission::MemoryAnalyzer => tr!(t, "agents.agent_memory_desc").to_string(),
                AgentMission::VulnerabilityCheck => tr!(t, "agents.agent_vuln_desc").to_string(),
                AgentMission::ThreatIntel => tr!(t, "agents.agent_intel_desc").to_string(),
            };
            (m.icon(), label, desc)
        })
        .collect()
}

pub fn render_process_selector(f: &mut Frame, app: &App) {
    let processes = &app.network.processes;
    let total = processes.len();
    let popup_area = centered_rect_with_ratios(
        f.area(),
        PROCESS_SELECTOR_WIDTH_DIVISOR,
        PROCESS_SELECTOR_HEIGHT_DIVISOR,
    );
    let visible = (popup_area.height as usize)
        .saturating_sub(PROCESS_SELECTOR_RESERVED_ROWS)
        .min(total);
    let cursor_idx = app.agents.selected_model_index.min(total.saturating_sub(1));
    let scroll_offset = scroll_offset_for(cursor_idx, visible);

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
        SELECTOR_HINT_TEXT,
        Style::default().fg(THEME.text_dim),
    )));

    for (i, proc) in processes
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take((scroll_offset + visible).min(total) - scroll_offset)
    {
        let path = proc.path.as_deref().unwrap_or(NO_PROCESS_NAME_PLACEHOLDER);
        let label = format!("{} | {} (PID: {})", proc.name, path, proc.pid);
        let checked = if app.agents.selected_pids.contains(&proc.pid) {
            PROCESS_SELECTOR_CHECKED
        } else {
            PROCESS_SELECTOR_UNCHECKED
        };
        let is_cursor = i == cursor_idx;
        lines.push(selector_row(
            &label,
            checked,
            is_cursor,
            app.agents.selected_pids.contains(&proc.pid),
        ));
    }

    if total > visible {
        lines.push(Line::from(Span::styled(
            format!(
                "   {}  {} ",
                scroll_indicator(scroll_offset, total, visible),
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
            "  {} / {}\u{00a0}\u{00a0}{}\u{00a0}\u{00a0}Enter: launch {} agent(s)",
            app.agents.selected_pids.len(),
            total,
            SELECTOR_TOGGLE_HINT,
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

pub fn render_network_selector(f: &mut Frame, app: &App) {
    let conns = &app.network.app_connections;
    let total = conns.len();
    let popup_area = centered_rect_with_ratios(
        f.area(),
        NETWORK_SELECTOR_WIDTH_DIVISOR,
        NETWORK_SELECTOR_HEIGHT_DIVISOR,
    );
    let visible = (popup_area.height as usize)
        .saturating_sub(NETWORK_SELECTOR_RESERVED_ROWS)
        .min(total);
    let cursor_idx = app.agents.selected_model_index.min(total.saturating_sub(1));
    let scroll_offset = scroll_offset_for(cursor_idx, visible);

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
        SELECTOR_HINT_TEXT,
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
            NETWORK_SELECTOR_CHECKED
        } else {
            NETWORK_SELECTOR_UNCHECKED
        };
        let is_cursor = i == cursor_idx;
        lines.push(selector_row(
            &label,
            checked,
            is_cursor,
            app.agents.selected_connection_idxs.contains(&i),
        ));
    }

    if total > visible {
        lines.push(Line::from(Span::styled(
            format!(
                "   {}  {} ",
                scroll_indicator(scroll_offset, total, visible),
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
            "  {} / {}\u{00a0}\u{00a0}{}\u{00a0}\u{00a0}Enter: launch {} agent(s)",
            app.agents.selected_connection_idxs.len(),
            total,
            SELECTOR_TOGGLE_HINT,
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

fn selector_row(label: &str, checked: &str, is_cursor: bool, is_checked: bool) -> Line<'static> {
    if is_cursor {
        Line::from(vec![
            Span::styled(SELECTED_ROW_INDICATOR, Style::default().fg(THEME.primary)),
            Span::styled(
                format!(" {} {} ", checked, label),
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else if is_checked {
        Line::from(vec![
            Span::raw(UNSELECTED_ROW_INDICATOR),
            Span::styled(
                format!(" {} {} ", checked, label),
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![
            Span::raw(UNSELECTED_ROW_INDICATOR),
            Span::styled(
                format!(" {} {} ", checked, label),
                Style::default().fg(THEME.text_main),
            ),
        ])
    }
}

fn scroll_offset_for(cursor_idx: usize, visible: usize) -> usize {
    if cursor_idx >= visible {
        cursor_idx - visible + 1
    } else {
        0
    }
}

fn scroll_indicator(scroll_offset: usize, total: usize, visible: usize) -> &'static str {
    if scroll_offset == 0 {
        SCROLL_INDICATOR_UP
    } else if scroll_offset + visible >= total {
        SCROLL_INDICATOR_DOWN
    } else {
        SCROLL_INDICATOR_BOTH
    }
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    Rect {
        x: (area.width.saturating_sub(width)) / 2,
        y: (area.height.saturating_sub(height)) / 2,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

fn centered_rect_with_ratios(area: Rect, width_div: u16, height_div: u16) -> Rect {
    let w = area.width * 3 / width_div;
    let h = area.height * 3 / height_div;
    Rect {
        x: (area.width.saturating_sub(w)) / 2,
        y: (area.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
    }
}
