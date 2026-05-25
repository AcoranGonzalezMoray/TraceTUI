use super::theme::THEME;
use super::widgets;
use crate::app::{App, AppConnection, InvestigationReport, SidebarFocus};
use crate::config;
use crate::tr;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
pub fn render_left_sidebar(f: &mut ratatui::Frame, app: &App, area: Rect) {
    if let Some(repo) = &app.investigation.investigation_report {
        render_investigation_left_sidebar(f, app, repo, area);
        return;
    }
    let is_focused = app.ui.sidebar_focus == SidebarFocus::Left;
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
    let filtered_apps: Vec<&AppConnection> = app.get_filtered_apps();
    if filtered_apps.is_empty() {
        render_empty_state(f, app, area, border_color, border_type);
        return;
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " 󰲚 {} ",
            tr!(app.ui.translator, "sidebar.processes", filtered_apps.len())
        ))
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(border_color))
        .border_type(border_type);
    f.render_widget(block.clone(), area);
    let inner_area = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(config::SCROLLBAR_WIDTH),
        ])
        .split(inner_area);
    let list_area = chunks[0];
    let scrollbar_area = chunks[1];
    let items: Vec<ListItem> = filtered_apps
        .iter()
        .enumerate()
        .map(|(i, app_conn)| {
            let is_selected = i == app.network.selected_app_index;
            let (risk_icon, risk_color) = match app_conn.risk_level.as_str() {
                "CRITICAL" => ("󰈸", THEME.danger),
                "HIGH" => ("󰀪", THEME.danger),
                "MEDIUM" => ("󰒓", THEME.warning),
                _ => ("󰄬", THEME.success),
            };
            let prefix = if is_selected { " ▎" } else { "  " };
            let prefix_style = if is_selected {
                Style::default().fg(THEME.primary)
            } else {
                Style::default()
            };
            let name_style = if is_selected {
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_main)
            };
            let content = vec![
                Line::from(vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled(&app_conn.process_name, name_style),
                ]),
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(
                        tr!(app.ui.translator, "sidebar.items", app_conn.connections.len()),
                        Style::default().fg(THEME.text_dim),
                    ),
                    Span::styled(" │ ", Style::default().fg(THEME.secondary)),
                    Span::styled(format!(" {} ", risk_icon), Style::default().fg(risk_color)),
                    Span::styled(
                        format!(" {} ", app_conn.risk_level),
                        Style::default()
                            .fg(THEME.background)
                            .bg(risk_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
            ];
            ListItem::new(content)
        })
        .collect();
    let mut list_state = ListState::default();
    list_state.select(Some(app.network.selected_app_index));
    let list = List::new(items).block(Block::default());
    f.render_stateful_widget(list, list_area, &mut list_state);
    widgets::render_scrollbar(
        f,
        scrollbar_area,
        filtered_apps.len(),
        app.network.selected_app_index,
    );
}
fn render_empty_state(
    f: &mut ratatui::Frame,
    app: &App,
    area: Rect,
    border_color: ratatui::style::Color,
    border_type: BorderType,
) {
    let empty_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            " 󱈸 ",
            Style::default().fg(THEME.warning),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            tr!(app.ui.translator, "sidebar.no_conns"),
            Style::default().fg(THEME.text_dim),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                tr!(app.ui.translator, "sidebar.press_r"),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " R ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                tr!(app.ui.translator, "sidebar.to_refresh"),
                Style::default().fg(THEME.text_dim),
            ),
        ]),
    ];
    let empty = Paragraph::new(empty_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " 󰲚 {} ",
                    tr!(app.ui.translator, "sidebar.processes", "")
                ))
                .title_style(
                    Style::default()
                        .fg(border_color)
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(border_color))
                .border_type(border_type),
        )
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(empty, area);
}
fn render_investigation_left_sidebar(
    f: &mut ratatui::Frame,
    app: &App,
    repo: &InvestigationReport,
    area: Rect,
) {
    let is_focused = app.ui.sidebar_focus == SidebarFocus::Left;
    let border_type = if is_focused {
        BorderType::Thick
    } else {
        BorderType::Rounded
    };
    let t = &app.ui.translator;
    let unknown = tr!(t, "investigation.unknown");
    let risk_color = if repo.risk_score < 30 {
        THEME.success
    } else if repo.risk_score < 60 {
        THEME.warning
    } else {
        THEME.danger
    };
    let conn_height = if repo.as_info.is_some() || repo.region.is_some() {
        14
    } else {
        11
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(conn_height), Constraint::Min(0)])
        .split(area);
    let conn_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " 󰩠 {}:{}:{} ",
            tr!(t, "investigation.title"),
            repo.ip,
            repo.port
        ))
        .title_style(Style::default().fg(risk_color).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(risk_color))
        .border_type(border_type);
    f.render_widget(conn_block.clone(), chunks[0]);
    let inner = conn_block.inner(chunks[0]);
    let conn_lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} ", tr!(t, "investigation.domain")),
                Style::default().fg(THEME.secondary),
            ),
            Span::styled(
                repo.domain.as_deref().unwrap_or(&unknown),
                Style::default()
                    .fg(THEME.text_main)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(tr!(t, "sidebar.ip"), Style::default().fg(THEME.secondary)),
            Span::styled(
                &repo.ip,
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(tr!(t, "sidebar.port"), Style::default().fg(THEME.secondary)),
            Span::styled(repo.port.to_string(), Style::default().fg(THEME.text_main)),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {} ", tr!(t, "investigation.org")),
                Style::default().fg(THEME.secondary),
            ),
            Span::styled(
                repo.organization.as_deref().unwrap_or(&unknown),
                Style::default().fg(THEME.text_main),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {} ", tr!(t, "investigation.isp")),
                Style::default().fg(THEME.secondary),
            ),
            Span::styled(
                repo.isp.as_deref().unwrap_or(&unknown),
                Style::default().fg(THEME.text_main),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {} ", tr!(t, "investigation.location")),
                Style::default().fg(THEME.secondary),
            ),
            Span::styled(
                format!(
                    "{} {}, {}",
                    crate::services::geoip_service::GeoIpService::get_flag_emoji(
                        repo.country.as_deref().unwrap_or("")
                    ),
                    repo.city.as_deref().unwrap_or("?"),
                    repo.country.as_deref().unwrap_or(&unknown)
                ),
                Style::default().fg(THEME.text_main),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {} ", tr!(t, "investigation.latency")),
                Style::default().fg(THEME.secondary),
            ),
            Span::styled(
                format!(
                    "{} ms",
                    repo.ping_ms
                        .as_deref()
                        .unwrap_or(&tr!(t, "investigation.timeout"))
                ),
                Style::default().fg(if repo.ping_ms.is_some() {
                    THEME.success
                } else {
                    THEME.danger
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {} ", tr!(t, "investigation.region")),
                Style::default().fg(THEME.secondary),
            ),
            Span::styled(
                repo.region.as_deref().unwrap_or(&unknown),
                Style::default().fg(THEME.text_main),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {} ", tr!(t, "investigation.zip")),
                Style::default().fg(THEME.secondary),
            ),
            Span::styled(
                repo.zip.as_deref().unwrap_or(&unknown),
                Style::default().fg(THEME.text_main),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {} ", tr!(t, "investigation.timezone")),
                Style::default().fg(THEME.secondary),
            ),
            Span::styled(
                repo.timezone.as_deref().unwrap_or(&unknown),
                Style::default().fg(THEME.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" {} ", tr!(t, "investigation.as")),
                Style::default().fg(THEME.secondary),
            ),
            Span::styled(
                repo.as_info.as_deref().unwrap_or(&unknown),
                Style::default().fg(THEME.text_main),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(conn_lines), inner);
    let no_whois = tr!(t, "investigation.no_whois");
    let whois_text = repo.whois_data.as_deref().unwrap_or(&no_whois);
    let whois_p = Paragraph::new(whois_text.to_string())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", tr!(t, "investigation.whois"))),
        )
        .style(Style::default().fg(THEME.text_dim))
        .wrap(Wrap { trim: true });
    f.render_widget(whois_p, chunks[1]);
}
