use super::theme::THEME;
use crate::app::containers::ContainerInfo;
use crate::app::{App, SidebarFocus};
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Gauge, List, ListItem, ListState, Paragraph, Row, Table,
        TableState, Wrap,
    },
};

pub fn render_containers_view(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(43),
            Constraint::Percentage(23),
        ])
        .split(area);

    render_container_list(f, app, columns[0]);
    render_container_details(f, app, columns[1]);
    render_container_actions(f, app, columns[2]);
}

fn render_container_list(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let focused = app.sidebar_focus == SidebarFocus::Left;
    let border_color = if focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " {} ",
            tr!(
                app.translator,
                "containers.list_title",
                app.containers.len()
            )
        ))
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(border_color))
        .border_type(if focused {
            BorderType::Thick
        } else {
            BorderType::Rounded
        });

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    if app.containers_loading && app.containers.is_empty() {
        render_centered(f, app, inner, "containers.loading");
        return;
    }

    if let Some(err) = &app.containers_error {
        let text = vec![
            Line::from(tr!(app.translator, "containers.error_title")),
            Line::from(""),
            Line::from(err.as_str()),
            Line::from(""),
            Line::from(tr!(app.translator, "containers.empty_hint")),
        ];
        f.render_widget(
            Paragraph::new(text)
                .style(Style::default().fg(THEME.text_dim))
                .wrap(Wrap { trim: true }),
            inner,
        );
        return;
    }

    if app.containers.is_empty() {
        render_centered(f, app, inner, "containers.empty");
        return;
    }

    let rows: Vec<Row> = app
        .containers
        .iter()
        .enumerate()
        .map(|(i, container)| {
            let selected = i == app.selected_container_index;
            let style = if selected {
                Style::default().fg(THEME.background).bg(THEME.primary)
            } else {
                Style::default().fg(THEME.text_main)
            };
            Row::new(vec![
                Cell::from(state_badge(app, container)),
                Cell::from(container.name.clone()),
                Cell::from(container.image.clone()),
            ])
            .style(style)
            .height(1)
        })
        .collect();

    let header = Row::new(vec![
        Cell::from(tr!(app.translator, "containers.col_state")),
        Cell::from(tr!(app.translator, "containers.col_name")),
        Cell::from(tr!(app.translator, "containers.col_image")),
    ])
    .style(
        Style::default()
            .fg(THEME.text_dim)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    state.select(Some(app.selected_container_index));
    let table = Table::new(
        rows,
        [
            Constraint::Length(9),
            Constraint::Percentage(38),
            Constraint::Percentage(45),
        ],
    )
    .header(header)
    .highlight_symbol("");
    f.render_stateful_widget(table, inner, &mut state);
}

fn render_container_details(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let focused = app.sidebar_focus == SidebarFocus::Center;
    let border_color = if focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " {} ",
            tr!(app.translator, "containers.details_title")
        ))
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(border_color))
        .border_type(if focused {
            BorderType::Thick
        } else {
            BorderType::Rounded
        });

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);
    let Some(container) = app.get_selected_container() else {
        render_centered(f, app, inner, "containers.select_hint");
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(inner);

    render_identity(f, app, container, chunks[0]);
    render_usage(f, app, container, chunks[1]);
    render_runtime(f, app, container, chunks[2]);
    render_logs(f, app, chunks[3]);
}

fn render_identity(f: &mut ratatui::Frame, app: &App, container: &ContainerInfo, area: Rect) {
    let lines = vec![
        Line::from(vec![
            label(app, "containers.field_name"),
            Span::styled(
                container.name.as_str(),
                Style::default()
                    .fg(THEME.text_main)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            label(app, "containers.field_id"),
            Span::raw(container.id.as_str()),
        ]),
        Line::from(vec![
            label(app, "containers.field_image"),
            Span::raw(container.image.as_str()),
        ]),
        Line::from(vec![
            label(app, "containers.field_status"),
            Span::raw(container.status.as_str()),
        ]),
        Line::from(vec![
            label(app, "containers.field_ports"),
            Span::raw(container.ports.as_str()),
        ]),
        Line::from(vec![
            label(app, "containers.field_networks"),
            Span::raw(container.networks.as_str()),
        ]),
    ];
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), area);
}

fn render_usage(f: &mut ratatui::Frame, app: &App, container: &ContainerInfo, area: Rect) {
    let gauges = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    let cpu = (container.cpu_percent.unwrap_or(0.0) / 100.0).clamp(0.0, 1.0);
    let mem = (container.memory_percent.unwrap_or(0.0) / 100.0).clamp(0.0, 1.0);
    f.render_widget(
        Gauge::default()
            .block(Block::default().title(tr!(app.translator, "containers.cpu")))
            .gauge_style(Style::default().fg(if cpu > 0.75 {
                THEME.danger
            } else {
                THEME.success
            }))
            .ratio(cpu)
            .label(format!("{:.1}%", container.cpu_percent.unwrap_or(0.0))),
        gauges[0],
    );
    f.render_widget(
        Gauge::default()
            .block(Block::default().title(tr!(app.translator, "containers.memory")))
            .gauge_style(Style::default().fg(if mem > 0.75 {
                THEME.warning
            } else {
                THEME.primary
            }))
            .ratio(mem)
            .label(format!(
                "{:.1}% {}",
                container.memory_percent.unwrap_or(0.0),
                container.memory_usage
            )),
        gauges[1],
    );
}

fn render_runtime(f: &mut ratatui::Frame, app: &App, container: &ContainerInfo, area: Rect) {
    let rows = vec![
        Row::new(vec![
            Cell::from(tr!(app.translator, "containers.field_created")),
            Cell::from(container.created.clone()),
        ]),
        Row::new(vec![
            Cell::from(tr!(app.translator, "containers.field_running_for")),
            Cell::from(container.running_for.clone()),
        ]),
        Row::new(vec![
            Cell::from(tr!(app.translator, "containers.field_net_io")),
            Cell::from(container.net_io.clone()),
        ]),
        Row::new(vec![
            Cell::from(tr!(app.translator, "containers.field_block_io")),
            Cell::from(container.block_io.clone()),
        ]),
        Row::new(vec![
            Cell::from(tr!(app.translator, "containers.field_size")),
            Cell::from(container.size.clone()),
        ]),
        Row::new(vec![
            Cell::from(tr!(app.translator, "containers.field_pids")),
            Cell::from(container.pids.clone()),
        ]),
    ];
    let table = Table::new(rows, [Constraint::Length(16), Constraint::Min(0)])
        .style(Style::default().fg(THEME.text_dim));
    f.render_widget(table, area);
}

fn render_logs(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .title(format!(
            " {} ",
            tr!(app.translator, "containers.logs_title")
        ))
        .title_style(
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    if app.container_logs_loading {
        render_centered(f, app, inner, "containers.logs_loading");
        return;
    }

    if app.container_logs.is_empty() {
        render_centered(f, app, inner, "containers.logs_empty");
        return;
    }

    let lines: Vec<Line> = app
        .container_logs
        .iter()
        .skip(app.container_detail_scroll)
        .map(|line| {
            Line::from(Span::styled(
                line.as_str(),
                Style::default().fg(THEME.text_dim),
            ))
        })
        .collect();
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn render_container_actions(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let focused = app.sidebar_focus == SidebarFocus::Right;
    let border_color = if focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " {} ",
            tr!(app.translator, "containers.actions_title")
        ))
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(border_color))
        .border_type(if focused {
            BorderType::Thick
        } else {
            BorderType::Rounded
        });
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let pause_label = app
        .get_selected_container()
        .filter(|c| c.state.eq_ignore_ascii_case("paused"))
        .map(|_| tr!(app.translator, "containers.action_unpause"))
        .unwrap_or_else(|| tr!(app.translator, "containers.action_pause"));

    let actions = vec![
        (
            "R",
            tr!(app.translator, "containers.action_refresh"),
            THEME.primary,
        ),
        (
            "L",
            tr!(app.translator, "containers.action_logs"),
            THEME.secondary,
        ),
        (
            "S",
            tr!(app.translator, "containers.action_start"),
            THEME.success,
        ),
        (
            "T",
            tr!(app.translator, "containers.action_stop"),
            THEME.danger,
        ),
        (
            "E",
            tr!(app.translator, "containers.action_restart"),
            THEME.warning,
        ),
        ("P", pause_label, THEME.accent),
    ];

    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(i, (key, label, color))| {
            let selected = i == app.selected_container_action_index;
            let marker = if selected { "> " } else { "  " };
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(marker, Style::default().fg(THEME.primary)),
                    Span::styled(
                        label.clone(),
                        Style::default()
                            .fg(if selected {
                                THEME.background
                            } else {
                                THEME.text_main
                            })
                            .bg(if selected {
                                THEME.primary
                            } else {
                                THEME.background
                            }),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(format!("[{}]", key), Style::default().fg(*color)),
                ]),
            ])
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected_container_action_index));
    f.render_stateful_widget(List::new(items), inner, &mut state);
}

fn render_centered(f: &mut ratatui::Frame, app: &App, area: Rect, key: &str) {
    f.render_widget(
        Paragraph::new(app.translator.get(key).to_string())
            .alignment(Alignment::Center)
            .style(Style::default().fg(THEME.text_dim))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn label<'a>(app: &'a App, key: &'a str) -> Span<'a> {
    Span::styled(
        format!("{} ", app.translator.get(key)),
        Style::default()
            .fg(THEME.primary)
            .add_modifier(Modifier::BOLD),
    )
}

fn state_badge(app: &App, container: &ContainerInfo) -> String {
    if container.state.eq_ignore_ascii_case("running") {
        tr!(app.translator, "containers.state_running")
    } else if container.state.eq_ignore_ascii_case("paused") {
        tr!(app.translator, "containers.state_paused")
    } else if container.state.is_empty() {
        tr!(app.translator, "containers.state_unknown")
    } else {
        container.state.to_uppercase()
    }
}
