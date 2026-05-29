use super::theme::THEME;
use super::widgets;
use crate::app::{App, SidebarFocus};
use crate::config;
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};
pub fn render_right_sidebar(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(14)])
        .split(area);
    render_actions_panel(f, app, chunks[0]);
    render_app_icon(f, app, chunks[1]);
}
fn render_actions_panel(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let border_color = if app.ui.sidebar_focus == SidebarFocus::Right {
        THEME.primary
    } else {
        THEME.secondary
    };
    let border_type = if app.ui.sidebar_focus == SidebarFocus::Right {
        BorderType::Thick
    } else {
        BorderType::Rounded
    };
    let t = &app.ui.translator;
    let actions: Vec<(&str, String, &str, ratatui::style::Color)> = if app.ui.show_map {
        vec![("󰩈", tr!(t, "actions.close_map"), "Esc", THEME.secondary)]
    } else if app.investigation.investigation_report.is_some() {
        vec![
            ("📍", tr!(t, "actions.locatemap"), "Enter", THEME.primary),
            (
                "󰩈",
                tr!(t, "actions.close_report"),
                "Esc/Q",
                THEME.secondary,
            ),
        ]
    } else {
        vec![
            (
                "󰑐",
                if app.ui.analysis_paused {
                    tr!(t, "actions.resume")
                } else {
                    tr!(t, "actions.pause")
                },
                "R",
                THEME.primary,
            ),
            ("󰆐", tr!(t, "actions.kill"), "X", THEME.danger),
            ("󰱝", tr!(t, "actions.kill_conns"), "-", THEME.danger),
            ("󰖟", tr!(t, "actions.search_online"), "G", THEME.secondary),
            ("󰅍", tr!(t, "actions.copy_path"), "C", THEME.secondary),
            ("󰒈", tr!(t, "actions.export"), "S", THEME.secondary),
            ("󰈸", tr!(t, "actions.filter_risk"), "F", THEME.warning),
            (
                "󰒓",
                tr!(t, "actions.filter_unsigned"),
                "H",
                if app.ui.hunter_mode {
                    THEME.success
                } else {
                    THEME.text_dim
                },
            ),
            ("󰒘", tr!(t, "actions.firewall"), "B", THEME.danger),
            ("󰗎", tr!(t, "actions.language"), "L", THEME.secondary),
        ]
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" 󰬒 {} ", tr!(app.ui.translator, "actions.title")))
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
    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(i, (icon, title, key, color))| {
            let is_selected = i == app.ui.selected_action_index;
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
            let content = vec![
                Line::from(vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled(format!(" {} ", icon), Style::default().fg(*color)),
                    Span::styled(title.clone(), title_style),
                ]),
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(format!("[ {} ]", key), Style::default().fg(THEME.text_dim)),
                ]),
            ];
            ListItem::new(content)
        })
        .collect();
    let mut list_state = ListState::default();
    list_state.select(Some(app.ui.selected_action_index));
    let list = List::new(items).block(Block::default());
    f.render_stateful_widget(list, list_area, &mut list_state);
    widgets::render_scrollbar(
        f,
        scrollbar_area,
        actions.len(),
        app.ui.selected_action_index,
    );
}
fn render_app_icon(f: &mut ratatui::Frame, app: &App, area: Rect) {
    if let Some(selected_app) = app.get_selected_app() {
        let icon_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" 󰰍 {} ", tr!(app.ui.translator, "icon.title")))
            .title_style(
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(THEME.secondary))
            .border_type(BorderType::Rounded);
        if selected_app.icon.is_empty() {
            let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let s = spinner[(app.ui.frame_count as usize) % spinner.len()];
            let inner_h = area.height.saturating_sub(2);
            let top = inner_h.saturating_sub(1) / 2;
            let bot = inner_h.saturating_sub(1) - top;
            let mut lines = vec![Line::from(""); top as usize];
            lines.push(Line::from(vec![Span::styled(
                format!("  {}  ", s),
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.extend(vec![Line::from(""); bot as usize]);
            let p = Paragraph::new(lines)
                .block(icon_block)
                .alignment(Alignment::Center);
            f.render_widget(p, area);
        } else {
            use ansi_to_tui::IntoText;
            let icon_widget = match selected_app.icon.as_bytes().into_text() {
                Ok(text) => Paragraph::new(text),
                Err(_) => Paragraph::new(selected_app.icon.as_str()),
            };
            let icon_p = icon_widget.block(icon_block).alignment(Alignment::Center);
            f.render_widget(icon_p, area);
        }
    }
}
