use super::theme::THEME;
use super::widgets;
use crate::app::services::input_service::{any_blocked_checked, any_conn_checked};
use crate::app::{App, FirewallPanel};
use crate::config;
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};
pub fn render_firewall_mode(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let right_col = 100 - 2 * config::FIREWALL_COL_PCT;
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(config::FIREWALL_COL_PCT),
            Constraint::Percentage(config::FIREWALL_COL_PCT),
            Constraint::Percentage(right_col),
        ])
        .split(area);
    render_connections_panel(f, app, columns[0]);
    render_blocked_panel(f, app, columns[1]);
    render_actions_panel(f, app, columns[2]);
}
fn checkbox(enabled: bool, checked: bool) -> &'static str {
    if enabled {
        if checked {
            "[x]"
        } else {
            "[ ]"
        }
    } else {
        "   "
    }
}
fn render_connections_panel(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.firewall.firewall_focus == FirewallPanel::Connections;
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
    if app.firewall.firewall_connections.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!(" 󱂇 {}", tr!(app.ui.translator, "firewall.no_conns")),
                Style::default().fg(THEME.text_dim),
            )]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " 󱂇 {} ",
                    tr!(
                        app.ui.translator,
                        "firewall.conns",
                        &app.firewall.firewall_process_name,
                        ""
                    )
                ))
                .title_style(
                    Style::default()
                        .fg(border_color)
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(border_color))
                .border_type(border_type),
        )
        .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " 󱂇 {} ",
            tr!(
                app.ui.translator,
                "firewall.conns",
                &app.firewall.firewall_process_name,
                app.firewall.firewall_connections.len()
            )
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
    let items: Vec<ListItem> = app
        .firewall
        .firewall_connections
        .iter()
        .enumerate()
        .map(|(i, conn)| {
            let is_selected = i == app.firewall.firewall_conn_index && is_focused;
            let boxed = app
                .firewall
                .firewall_conn_checked
                .get(i)
                .copied()
                .unwrap_or(false);
            let chk = checkbox(true, boxed);
            let conn_style = if is_selected {
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_main)
            };
            let indicator = if is_selected { " ▎" } else { "  " };
            let content = vec![
                Line::from(vec![
                    Span::styled(indicator, Style::default().fg(THEME.primary)),
                    Span::styled(
                        format!(" {} ", chk),
                        if boxed {
                            Style::default()
                                .fg(THEME.success)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(THEME.text_dim)
                        },
                    ),
                    Span::styled(
                        format!("{}:{}", conn.foreign_address, conn.foreign_port),
                        conn_style,
                    ),
                ]),
                Line::from(vec![
                    Span::raw("       "),
                    Span::styled(
                        conn.protocol.to_string(),
                        Style::default().fg(THEME.secondary),
                    ),
                    Span::styled(" │ ", Style::default().fg(THEME.text_dim)),
                    Span::styled(&conn.state, Style::default().fg(THEME.text_dim)),
                ]),
            ];
            ListItem::new(content)
        })
        .collect();
    let mut list_state = ListState::default();
    list_state.select(Some(app.firewall.firewall_conn_index));
    let list = List::new(items).block(Block::default());
    f.render_stateful_widget(list, chunks[0], &mut list_state);
    widgets::render_scrollbar(
        f,
        chunks[1],
        app.firewall.firewall_connections.len(),
        app.firewall.firewall_conn_index,
    );
    let count = app
        .firewall
        .firewall_conn_checked
        .iter()
        .filter(|&&c| c)
        .count();
    let hint = Paragraph::new(Line::from(vec![
        Span::styled(
            tr!(app.ui.translator, "firewall.checked", count),
            Style::default().fg(if count > 0 {
                THEME.success
            } else {
                THEME.text_dim
            }),
        ),
        Span::styled(
            format!("  {}", tr!(app.ui.translator, "firewall.hint_enter")),
            Style::default().fg(THEME.text_dim),
        ),
    ]))
    .alignment(Alignment::Right);
    let hint_area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    f.render_widget(hint, hint_area);
}
fn render_blocked_panel(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.firewall.firewall_focus == FirewallPanel::BlockedList;
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
    if app.firewall.blocked_ips.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!(" 󰒘 {}", tr!(app.ui.translator, "firewall.no_blocked")),
                Style::default().fg(THEME.text_dim),
            )]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " 󰒘 {} ",
                    tr!(app.ui.translator, "firewall.blocked", "")
                ))
                .title_style(
                    Style::default()
                        .fg(border_color)
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(border_color))
                .border_type(border_type),
        )
        .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " 󰒘 {} ",
            tr!(
                app.ui.translator,
                "firewall.blocked",
                app.firewall.blocked_ips.len()
            )
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
    let items: Vec<ListItem> = app
        .firewall
        .blocked_ips
        .iter()
        .enumerate()
        .map(|(i, (ip, pname, _))| {
            let is_selected = i == app.firewall.firewall_blocked_index && is_focused;
            let boxed = app
                .firewall
                .firewall_blocked_checked
                .get(i)
                .copied()
                .unwrap_or(false);
            let chk = checkbox(true, boxed);
            let ip_style = if is_selected {
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.danger)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.danger)
            };
            let indicator = if is_selected { " ▎" } else { "  " };
            let content = vec![
                Line::from(vec![
                    Span::styled(indicator, Style::default().fg(THEME.danger)),
                    Span::styled(
                        format!(" {} ", chk),
                        if boxed {
                            Style::default()
                                .fg(THEME.success)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(THEME.text_dim)
                        },
                    ),
                    Span::styled(ip, ip_style),
                ]),
                Line::from(vec![
                    Span::raw("       "),
                    Span::styled(pname, Style::default().fg(THEME.text_dim)),
                ]),
            ];
            ListItem::new(content)
        })
        .collect();
    let mut list_state = ListState::default();
    list_state.select(Some(app.firewall.firewall_blocked_index));
    let list = List::new(items).block(Block::default());
    f.render_stateful_widget(list, chunks[0], &mut list_state);
    widgets::render_scrollbar(
        f,
        chunks[1],
        app.firewall.blocked_ips.len(),
        app.firewall.firewall_blocked_index,
    );
    let count = app
        .firewall
        .firewall_blocked_checked
        .iter()
        .filter(|&&c| c)
        .count();
    let hint = Paragraph::new(Line::from(vec![
        Span::styled(
            tr!(app.ui.translator, "firewall.checked", count),
            Style::default().fg(if count > 0 {
                THEME.success
            } else {
                THEME.text_dim
            }),
        ),
        Span::styled(
            format!("  {}", tr!(app.ui.translator, "firewall.hint_enter")),
            Style::default().fg(THEME.text_dim),
        ),
    ]))
    .alignment(Alignment::Right);
    let hint_area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    f.render_widget(hint, hint_area);
}
fn render_actions_panel(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.firewall.firewall_focus == FirewallPanel::Actions;
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
    let any_conn = any_conn_checked(app);
    let any_blocked = any_blocked_checked(app);
    struct ActionDef {
        icon: &'static str,
        title: String,
        key: &'static str,
        color: Color,
        enabled: bool,
    }
    let actions = [
        ActionDef {
            icon: "󰄭",
            title: tr!(app.ui.translator, "firewall.action_select"),
            key: "Space",
            color: THEME.primary,
            enabled: true,
        },
        ActionDef {
            icon: "󰒘",
            title: tr!(app.ui.translator, "firewall.action_block"),
            key: "B",
            color: THEME.danger,
            enabled: any_conn,
        },
        ActionDef {
            icon: "󰅁",
            title: tr!(app.ui.translator, "firewall.action_unblock"),
            key: "U",
            color: THEME.success,
            enabled: any_blocked,
        },
        ActionDef {
            icon: "󰩈",
            title: tr!(app.ui.translator, "firewall.action_exit"),
            key: "Esc",
            color: THEME.secondary,
            enabled: true,
        },
    ];
    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let is_selected = i == app.firewall.firewall_action_index && is_focused;
            let icon_style = if a.enabled {
                Style::default().fg(a.color)
            } else {
                Style::default().fg(THEME.text_dim)
            };
            let title_style = if is_selected {
                if a.enabled {
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(THEME.background).bg(THEME.text_dim)
                }
            } else if a.enabled {
                Style::default().fg(THEME.text_main)
            } else {
                Style::default().fg(THEME.text_dim)
            };
            let prefix = if is_selected { " ▎" } else { "  " };
            let title_str = a.title.clone();
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(prefix, Style::default().fg(THEME.primary)),
                    Span::styled(format!(" {} ", a.icon), icon_style),
                    Span::styled(title_str, title_style),
                ]),
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(
                        format!("[ {} ]", a.key),
                        Style::default().fg(THEME.text_dim),
                    ),
                ]),
            ])
        })
        .collect();
    let mut list_state = ListState::default();
    list_state.select(Some(app.firewall.firewall_action_index));
    let list = List::new(items).block(Block::default());
    f.render_stateful_widget(list, inner_area, &mut list_state);
}
