use super::theme::THEME;
use crate::app::containers::{ContainerInfo, DockerStatus, DOCKER_ACTION_OFFSET};
use crate::app::{App, SidebarFocus};
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Gauge, List, ListItem, ListState, Paragraph, Row,
        Table, TableState, Wrap,
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
    render_right_panels(f, app, columns[2]);
}

fn render_container_list(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let focused = app.ui.sidebar_focus == SidebarFocus::Left;
    let border_color = focus_color(focused);

    let block = styled_block(
        format!(
            " {} ",
            tr!(
                app.ui.translator,
                "containers.list_title",
                app.containers.containers.len()
            )
        ),
        border_color,
        focused,
    );

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    if app.containers.containers_loading && app.containers.containers.is_empty() {
        render_docker_placeholder(f, app, inner, DockerStatus::Starting);
        return;
    }
    if app.containers.containers_error.is_some() {
        render_docker_placeholder(f, app, inner, app.docker_status());
        return;
    }
    if app.containers.containers.is_empty() {
        render_docker_empty(f, app, inner);
        return;
    }

    let (running, stopped, paused) = container_counts(&app.containers.containers);
    let total = app.containers.containers.len();

    let summary_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 1,
    };
    let table_area = Rect {
        x: inner.x,
        y: inner.y + 1,
        width: inner.width,
        height: inner.height.saturating_sub(1),
    };

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" \u{25b6} ", Style::default().fg(THEME.success)),
            Span::styled(
                format!("{}", running),
                Style::default()
                    .fg(THEME.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  \u{25a0} ", Style::default().fg(THEME.danger)),
            Span::styled(
                format!("{}", stopped),
                Style::default()
                    .fg(THEME.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  \u{23f8} ", Style::default().fg(THEME.warning)),
            Span::styled(
                format!("{}", paused),
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  / {} total", total),
                Style::default().fg(THEME.text_dim),
            ),
        ])),
        summary_area,
    );

    let rows: Vec<Row> = app
        .containers
        .containers
        .iter()
        .enumerate()
        .map(|(i, container)| {
            let selected = i == app.containers.selected_container_index;
            let (badge, badge_color) = state_badge_styled(app, container);
            let name_style = if selected {
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_main)
            };
            let badge_style = if selected {
                Style::default().fg(THEME.background).bg(THEME.primary)
            } else {
                Style::default()
                    .fg(badge_color)
                    .add_modifier(Modifier::BOLD)
            };

            let cpu_dot = match container.cpu_percent {
                Some(c) if c >= 75.0 => {
                    Span::styled("\u{25cf} ", Style::default().fg(THEME.danger))
                }
                Some(c) if c >= 40.0 => {
                    Span::styled("\u{25cf} ", Style::default().fg(THEME.warning))
                }
                Some(_) => Span::styled("\u{25cf} ", Style::default().fg(THEME.success)),
                None => Span::styled("\u{25cb} ", Style::default().fg(THEME.text_dim)),
            };

            Row::new(vec![
                Cell::from(Span::styled(badge, badge_style)),
                Cell::from(Line::from(vec![
                    cpu_dot,
                    Span::styled(container.name.clone(), name_style),
                ])),
                Cell::from(Span::styled(
                    truncate_str(&container.image, 18),
                    if selected {
                        Style::default().fg(THEME.background).bg(THEME.primary)
                    } else {
                        Style::default().fg(THEME.text_dim)
                    },
                )),
            ])
            .height(1)
        })
        .collect();

    let header = Row::new(vec![
        Cell::from(Span::styled(
            tr!(app.ui.translator, "containers.col_state"),
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Cell::from(Span::styled(
            tr!(app.ui.translator, "containers.col_name"),
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Cell::from(Span::styled(
            tr!(app.ui.translator, "containers.col_image"),
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
    ]);

    let mut state = TableState::default();
    state.select(Some(app.containers.selected_container_index));

    f.render_stateful_widget(
        Table::new(
            rows,
            [
                Constraint::Length(9),
                Constraint::Percentage(42),
                Constraint::Percentage(48),
            ],
        )
        .header(header)
        .highlight_symbol(""),
        table_area,
        &mut state,
    );
}

fn render_container_details(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let focused = app.ui.sidebar_focus == SidebarFocus::Center;
    let border_color = focus_color(focused);

    let block = styled_block(
        format!(" {} ", tr!(app.ui.translator, "containers.details_title")),
        border_color,
        focused,
    );
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let Some(container) = app.get_selected_container() else {
        render_centered(f, app, inner, "containers.select_hint");
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Length(5),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(inner);

    render_identity(f, app, container, chunks[0]);
    render_usage(f, app, container, chunks[1]);
    render_runtime(f, app, container, chunks[2]);
    render_log_hint(f, app, chunks[3]);
}

fn render_identity(f: &mut ratatui::Frame, app: &App, container: &ContainerInfo, area: Rect) {
    let is_running = container.state.eq_ignore_ascii_case("running");
    let is_paused = container.state.eq_ignore_ascii_case("paused");
    let state_color = if is_running {
        THEME.success
    } else if is_paused {
        THEME.warning
    } else {
        THEME.danger
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(state_color))
        .title(Span::styled(
            format!(
                " {} ",
                tr!(app.ui.translator, "containers.identity_section")
            ),
            Style::default()
                .fg(state_color)
                .add_modifier(Modifier::BOLD),
        ));
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let state_icon = if is_running {
        format!(
            "\u{25b6} {}",
            tr!(app.ui.translator, "containers.state_running_badge")
        )
    } else if is_paused {
        format!(
            "\u{23f8} {}",
            tr!(app.ui.translator, "containers.state_paused_badge")
        )
    } else {
        format!(
            "\u{25a0} {}",
            tr!(app.ui.translator, "containers.state_stopped_badge")
        )
    };

    let lines = vec![
        Line::from(Span::styled(
            format!("  {}", state_icon),
            Style::default()
                .fg(state_color)
                .add_modifier(Modifier::BOLD),
        )),
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
            Span::styled(
                &container.id[..12.min(container.id.len())],
                Style::default().fg(THEME.text_dim),
            ),
        ]),
        Line::from(vec![
            label(app, "containers.field_image"),
            Span::styled(
                container.image.as_str(),
                Style::default().fg(THEME.secondary),
            ),
        ]),
        Line::from(vec![
            label(app, "containers.field_status"),
            Span::styled(container.status.as_str(), Style::default().fg(state_color)),
        ]),
        Line::from(vec![
            label(app, "containers.field_ports"),
            Span::styled(
                if container.ports.is_empty() || container.ports == "-" {
                    "\u{2014}"
                } else {
                    &container.ports
                },
                Style::default().fg(THEME.primary),
            ),
        ]),
        Line::from(vec![
            label(app, "containers.field_networks"),
            Span::styled(
                if container.networks.is_empty() || container.networks == "-" {
                    "\u{2014}"
                } else {
                    &container.networks
                },
                Style::default().fg(THEME.text_dim),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn render_usage(f: &mut ratatui::Frame, app: &App, container: &ContainerInfo, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.secondary))
        .title(Span::styled(
            format!(" {} ", tr!(app.ui.translator, "containers.usage_section")),
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ));
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let cpu = (container.cpu_percent.unwrap_or(0.0) / 100.0).clamp(0.0, 1.0);
    let mem = (container.memory_percent.unwrap_or(0.0) / 100.0).clamp(0.0, 1.0);

    let cpu_color = gauge_color(cpu);
    let mem_color = gauge_color(mem);

    f.render_widget(
        Gauge::default()
            .block(
                Block::default()
                    .title(Span::styled(
                        format!(" {} ", tr!(app.ui.translator, "containers.cpu")),
                        Style::default().fg(cpu_color),
                    ))
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(cpu_color)),
            )
            .gauge_style(Style::default().fg(cpu_color))
            .ratio(cpu)
            .label(Span::styled(
                format!("{:.1}%", container.cpu_percent.unwrap_or(0.0)),
                Style::default()
                    .fg(THEME.text_main)
                    .add_modifier(Modifier::BOLD),
            )),
        cols[0],
    );
    f.render_widget(
        Gauge::default()
            .block(
                Block::default()
                    .title(Span::styled(
                        format!(" {} ", tr!(app.ui.translator, "containers.memory")),
                        Style::default().fg(mem_color),
                    ))
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(mem_color)),
            )
            .gauge_style(Style::default().fg(mem_color))
            .ratio(mem)
            .label(Span::styled(
                format!(
                    "{:.1}% {}",
                    container.memory_percent.unwrap_or(0.0),
                    container.memory_usage
                ),
                Style::default()
                    .fg(THEME.text_main)
                    .add_modifier(Modifier::BOLD),
            )),
        cols[1],
    );
}

fn render_runtime(f: &mut ratatui::Frame, app: &App, container: &ContainerInfo, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.secondary))
        .title(Span::styled(
            format!(" {} ", tr!(app.ui.translator, "containers.runtime_section")),
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ));
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let key_style = Style::default().fg(THEME.text_dim);
    let val_style = Style::default().fg(THEME.text_main);
    let highlight_style = Style::default()
        .fg(THEME.primary)
        .add_modifier(Modifier::BOLD);

    let rows = vec![
        Row::new(vec![
            Cell::from(Span::styled(
                tr!(app.ui.translator, "containers.field_running_for"),
                key_style,
            )),
            Cell::from(Span::styled(container.running_for.clone(), highlight_style)),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                tr!(app.ui.translator, "containers.field_created"),
                key_style,
            )),
            Cell::from(Span::styled(container.created.clone(), val_style)),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                tr!(app.ui.translator, "containers.field_net_io"),
                key_style,
            )),
            Cell::from(Span::styled(container.net_io.clone(), val_style)),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                tr!(app.ui.translator, "containers.field_block_io"),
                key_style,
            )),
            Cell::from(Span::styled(container.block_io.clone(), val_style)),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                tr!(app.ui.translator, "containers.field_size"),
                key_style,
            )),
            Cell::from(Span::styled(container.size.clone(), val_style)),
        ]),
        Row::new(vec![
            Cell::from(Span::styled(
                tr!(app.ui.translator, "containers.field_pids"),
                key_style,
            )),
            Cell::from(Span::styled(container.pids.clone(), val_style)),
        ]),
    ];

    f.render_widget(
        Table::new(rows, [Constraint::Length(16), Constraint::Min(0)]),
        inner,
    );
}

fn render_log_hint(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(THEME.secondary))
        .title(Span::styled(
            format!(" {} ", tr!(app.ui.translator, "containers.logs_title")),
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        ));

    let text = if app.containers.container_logs_loading {
        tr!(app.ui.translator, "containers.logs_loading")
    } else {
        tr!(app.ui.translator, "containers.logs_modal_hint")
    };

    f.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(THEME.text_dim))
            .block(block),
        area,
    );
}

fn render_right_panels(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(12), Constraint::Length(9)])
        .split(area);

    render_container_actions(f, app, chunks[0]);
    render_docker_actions(f, app, chunks[1]);
}

fn render_container_actions(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let focused = app.ui.sidebar_focus == SidebarFocus::Right;
    let border_color = focus_color(focused);

    let block = styled_block(
        format!(" {} ", tr!(app.ui.translator, "containers.actions_title")),
        border_color,
        focused,
    );
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let pause_label = app
        .get_selected_container()
        .filter(|c| c.state.eq_ignore_ascii_case("paused"))
        .map(|_| tr!(app.ui.translator, "containers.action_unpause"))
        .unwrap_or_else(|| tr!(app.ui.translator, "containers.action_pause"));

    let actions: Vec<(&str, String, &str, ratatui::style::Color)> = vec![
        (
            "\u{f021}",
            tr!(app.ui.translator, "containers.action_refresh"),
            "R",
            THEME.text_dim,
        ),
        (
            "\u{f022}",
            tr!(app.ui.translator, "containers.action_logs"),
            "V",
            THEME.secondary,
        ),
        (
            "\u{e795}",
            tr!(app.ui.translator, "containers.action_console"),
            "C",
            THEME.secondary,
        ),
        (
            "\u{f04b}",
            tr!(app.ui.translator, "containers.action_start"),
            "S",
            THEME.success,
        ),
        (
            "\u{f04d}",
            tr!(app.ui.translator, "containers.action_stop"),
            "T",
            THEME.danger,
        ),
        (
            "\u{f021}",
            tr!(app.ui.translator, "containers.action_restart"),
            "E",
            THEME.warning,
        ),
        ("\u{f04c}", pause_label, "P", THEME.primary),
    ];

    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(i, (icon, lbl, key, color))| {
            action_item(
                icon,
                lbl.clone(),
                key,
                *color,
                i == app.containers.selected_container_action_index,
            )
        })
        .collect();

    let mut state = ListState::default();
    if app.containers.selected_container_action_index < DOCKER_ACTION_OFFSET {
        state.select(Some(app.containers.selected_container_action_index));
    }
    f.render_stateful_widget(List::new(items), inner, &mut state);
}

fn render_docker_actions(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let focused = app.ui.sidebar_focus == SidebarFocus::Right;
    let selected = app
        .containers
        .selected_container_action_index
        .saturating_sub(DOCKER_ACTION_OFFSET);

    let (status_icon, status_color) = match app.docker_status() {
        DockerStatus::On => ("\u{25cf} ON", THEME.success),
        DockerStatus::Starting => ("\u{25cc} STARTING", THEME.warning),
        DockerStatus::Off => ("\u{25cb} OFF", THEME.danger),
        DockerStatus::Missing => ("\u{2717} MISSING", THEME.danger),
        DockerStatus::Unknown => ("? UNKNOWN", THEME.text_dim),
    };

    let title = format!(
        " {}  [{}] ",
        tr!(app.ui.translator, "containers.docker_actions_title"),
        status_icon
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            title,
            Style::default()
                .fg(if focused {
                    status_color
                } else {
                    THEME.secondary
                })
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(if focused {
            THEME.primary
        } else {
            THEME.secondary
        }))
        .border_type(if focused {
            BorderType::Thick
        } else {
            BorderType::Rounded
        });

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let actions = [
        (
            "\u{f011}",
            tr!(app.ui.translator, "containers.docker_action_start"),
            "N",
            THEME.success,
        ),
        (
            "\u{f011}",
            tr!(app.ui.translator, "containers.docker_action_stop"),
            "O",
            THEME.danger,
        ),
        (
            "\u{e28b}",
            tr!(app.ui.translator, "containers.docker_hub_search_title"),
            "H",
            THEME.secondary,
        ),
    ];

    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(i, (icon, lbl, key, color))| {
            action_item(
                icon,
                lbl.clone(),
                key,
                *color,
                app.containers.selected_container_action_index >= DOCKER_ACTION_OFFSET
                    && i == selected,
            )
        })
        .collect();

    let mut state = ListState::default();
    if app.containers.selected_container_action_index >= DOCKER_ACTION_OFFSET {
        state.select(Some(selected));
    }
    f.render_stateful_widget(List::new(items), inner, &mut state);
}

pub fn render_container_logs_modal(f: &mut ratatui::Frame, app: &App) {
    let area = centered_rect(f.area(), 86, 78);
    f.render_widget(Clear, area);

    let container_name = app
        .get_selected_container()
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let total_lines = app.containers.container_logs.len();
    let visible = area.height.saturating_sub(4) as usize;
    let scroll_pct = ((app.containers.container_logs_scroll + visible).min(total_lines) * 100)
        .checked_div(total_lines)
        .unwrap_or(0) as u16;

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!(
                "  {}  {}  ",
                tr!(app.ui.translator, "containers.logs_title"),
                container_name
            ),
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Double);

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    if app.containers.container_logs_loading {
        render_centered(f, app, inner, "containers.logs_loading");
        return;
    }
    if app.containers.container_logs.is_empty() {
        render_centered(f, app, inner, "containers.logs_empty");
        return;
    }

    let log_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width.saturating_sub(1),
        height: inner.height.saturating_sub(1),
    };
    let scrollbar_area = Rect {
        x: inner.x + inner.width.saturating_sub(1),
        y: inner.y,
        width: 1,
        height: inner.height.saturating_sub(1),
    };
    let hint_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(1),
        width: inner.width,
        height: 1,
    };

    let lines: Vec<Line> = app
        .containers
        .container_logs
        .iter()
        .skip(app.containers.container_logs_scroll)
        .take(log_area.height as usize)
        .enumerate()
        .map(|(idx, line)| {
            let lower = line.to_lowercase();
            let line_color = if lower.contains("error") || lower.contains("fatal") {
                THEME.danger
            } else if lower.contains("warn") {
                THEME.warning
            } else {
                THEME.text_main
            };
            Line::from(vec![
                Span::styled(
                    format!(
                        "{:>4} \u{2502} ",
                        app.containers.container_logs_scroll + idx + 1
                    ),
                    Style::default().fg(THEME.text_dim),
                ),
                Span::styled(line.as_str(), Style::default().fg(line_color)),
            ])
        })
        .collect();

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), log_area);

    render_scrollbar(f, scrollbar_area, scroll_pct);

    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            tr!(
                app.ui.translator,
                "containers.logs_hint",
                (app.containers.container_logs_scroll + visible).min(total_lines),
                total_lines
            ),
            Style::default().fg(THEME.text_dim),
        )]))
        .alignment(Alignment::Right),
        hint_area,
    );
}

pub fn render_container_console_modal(f: &mut ratatui::Frame, app: &App) {
    let area = centered_rect(f.area(), 86, 78);
    f.render_widget(Clear, area);

    let title = app
        .get_selected_container()
        .map(|c| tr!(app.ui.translator, "containers.console_title", &c.name))
        .unwrap_or_else(|| tr!(app.ui.translator, "containers.docker_action_console"));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!("  {}  ", title),
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Double);

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(inner);

    let visible = chunks[0].height as usize;
    let lines: Vec<Line> = app
        .containers
        .container_console_output
        .iter()
        .skip(app.containers.container_console_scroll)
        .take(visible)
        .map(|line| {
            if line.starts_with('$') {
                Line::from(Span::styled(
                    line.as_str(),
                    Style::default()
                        .fg(THEME.success)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                Line::from(Span::styled(
                    line.as_str(),
                    Style::default().fg(THEME.text_main),
                ))
            }
        })
        .collect();

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), chunks[0]);

    let cursor = if app.ui.frame_count.is_multiple_of(2) {
        "\u{258c}"
    } else {
        " "
    };
    let prompt = if app.containers.container_console_loading {
        tr!(app.ui.translator, "containers.console_running")
    } else {
        app.containers.container_console_input.clone()
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.success))
        .title(Span::styled(
            format!(
                " {} ",
                tr!(app.ui.translator, "containers.console_input_hint")
            ),
            Style::default().fg(THEME.text_dim),
        ));

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                "$ ",
                Style::default()
                    .fg(THEME.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(prompt, Style::default().fg(THEME.text_main)),
            Span::styled(cursor, Style::default().fg(THEME.primary)),
        ]))
        .block(input_block),
        chunks[1],
    );
}

pub fn render_docker_hub_modal(f: &mut ratatui::Frame, app: &App) {
    let area = centered_rect(f.area(), 90, 75);
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!(
                "  {}  ",
                tr!(app.ui.translator, "containers.docker_hub_search_title")
            ),
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Thick);

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let hint_area = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(1),
        width: inner.width,
        height: 1,
    };
    let content_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(1),
    };

    f.render_widget(
        Paragraph::new(Span::styled(
            tr!(app.ui.translator, "containers.hub_hint"),
            Style::default().fg(THEME.text_dim),
        ))
        .alignment(Alignment::Center),
        hint_area,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(content_area);

    render_docker_hub_search_input(f, app, chunks[0]);
    render_docker_hub_content(f, app, chunks[1]);
    render_docker_hub_buttons(f, app, chunks[2]);
}

fn render_docker_hub_search_input(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.containers.docker_hub_search.focused_field == 0;
    let border_color = if is_focused {
        THEME.primary
    } else {
        THEME.secondary
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(if is_focused {
            BorderType::Thick
        } else {
            BorderType::Rounded
        })
        .title(Span::styled(
            tr!(app.ui.translator, "containers.hub_search_title"),
            Style::default().fg(border_color),
        ));

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let query_text = if is_focused && app.containers.docker_hub_search.search_query.is_empty() {
        app.ui
            .translator
            .get("containers.docker_hub_search_placeholder")
            .to_string()
    } else {
        app.containers.docker_hub_search.search_query.clone()
    };

    let cursor = if is_focused {
        format!("{}\u{258c}", query_text)
    } else {
        query_text
    };

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            cursor,
            Style::default().fg(if is_focused {
                THEME.text_main
            } else {
                THEME.text_dim
            }),
        )))
        .wrap(Wrap { trim: true }),
        inner,
    );
}

fn render_docker_hub_content(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_docker_hub_results(f, app, columns[0]);
    render_docker_hub_form(f, app, columns[1]);
}

fn render_docker_hub_results(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary))
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            tr!(app.ui.translator, "containers.hub_results"),
            Style::default().fg(THEME.secondary),
        ));

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    if app.containers.docker_hub_search.results.is_empty() {
        let msg = if app.containers.docker_hub_search.search_query.is_empty() {
            "containers.docker_hub_search_placeholder"
        } else {
            "containers.docker_hub_no_results"
        };
        f.render_widget(
            Paragraph::new(app.ui.translator.get(msg).to_string())
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim))
                .wrap(Wrap { trim: true }),
            inner,
        );
        return;
    }

    let items: Vec<ListItem> = app
        .containers
        .docker_hub_search
        .results
        .iter()
        .enumerate()
        .map(|(i, img)| {
            let is_selected = i == app.containers.docker_hub_search.selected_result_index;
            let name_style = if is_selected {
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_main)
            };

            let mut badges = Vec::new();
            if img.official {
                badges.push(Span::styled(
                    " \u{2713} official",
                    Style::default().fg(THEME.success),
                ));
            }
            if img.automated {
                badges.push(Span::styled(
                    " \u{2699} auto",
                    Style::default().fg(THEME.secondary),
                ));
            }

            let mut badge_line = vec![Span::raw("   ")];
            badge_line.extend(badges);

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        if is_selected { " \u{258e} " } else { "   " },
                        Style::default().fg(THEME.primary),
                    ),
                    Span::styled(img.name.clone(), name_style),
                ]),
                Line::from(badge_line),
            ])
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.containers.docker_hub_search.selected_result_index));
    f.render_stateful_widget(List::new(items), inner, &mut state);
}

fn render_docker_hub_form(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary))
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            tr!(app.ui.translator, "containers.hub_config"),
            Style::default().fg(THEME.secondary),
        ));

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(inner);

    render_form_field(
        f,
        app,
        rows[0],
        1,
        "containers.docker_hub_name_field",
        &app.containers.docker_hub_search.container_name,
    );
    render_form_field(
        f,
        app,
        rows[1],
        2,
        "containers.docker_hub_ports_field",
        &app.containers.docker_hub_search.ports,
    );
    render_form_field(
        f,
        app,
        rows[2],
        3,
        "containers.docker_hub_env_field",
        &app.containers.docker_hub_search.env_vars,
    );

    if rows[3].height > 3 {
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    tr!(app.ui.translator, "containers.hub_example_ports"),
                    Style::default().fg(THEME.text_dim),
                )),
                Line::from(Span::styled(
                    tr!(app.ui.translator, "containers.hub_example_env"),
                    Style::default().fg(THEME.text_dim),
                )),
            ])
            .wrap(Wrap { trim: true }),
            rows[3],
        );
    }
}

fn render_form_field(
    f: &mut ratatui::Frame,
    app: &App,
    area: Rect,
    field_index: usize,
    label_key: &str,
    value: &str,
) {
    let is_focused = app.containers.docker_hub_search.focused_field == field_index;
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
        .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
        .border_style(Style::default().fg(border_color))
        .border_type(border_type);

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let cursor = if is_focused {
        format!("{}\u{258c}", value)
    } else {
        value.to_string()
    };

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{} ", app.ui.translator.get(label_key)),
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                cursor,
                Style::default().fg(if is_focused {
                    THEME.text_main
                } else {
                    THEME.text_dim
                }),
            ),
        ]))
        .wrap(Wrap { trim: true }),
        inner,
    );
}

fn render_docker_hub_buttons(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let create_focused = app.containers.docker_hub_search.focused_field == 4;
    let cancel_focused = app.containers.docker_hub_search.focused_field == 5;

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            if create_focused {
                format!(
                    "[ \u{2713} {} ]",
                    tr!(app.ui.translator, "containers.button_create")
                )
            } else {
                format!(
                    "[   {} ]",
                    tr!(app.ui.translator, "containers.button_create")
                )
            },
            if create_focused {
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.success)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.success)
            },
        )))
        .alignment(Alignment::Center),
        button_chunks[0],
    );

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            if cancel_focused {
                format!(
                    "[ \u{2717} {} ]",
                    tr!(app.ui.translator, "containers.button_cancel")
                )
            } else {
                format!(
                    "[   {} ]",
                    tr!(app.ui.translator, "containers.button_cancel")
                )
            },
            if cancel_focused {
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.danger)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.danger)
            },
        )))
        .alignment(Alignment::Center),
        button_chunks[1],
    );
}

fn render_docker_empty(f: &mut ratatui::Frame, app: &App, area: Rect) {
    render_placeholder_lines(
        f,
        area,
        "\u{f308}",
        tr!(app.ui.translator, "containers.placeholder_no_containers"),
        tr!(app.ui.translator, "containers.placeholder_create_hint"),
        THEME.secondary,
    );
}

fn render_docker_placeholder(f: &mut ratatui::Frame, app: &App, area: Rect, status: DockerStatus) {
    let (icon, title, hint, color) = match status {
        DockerStatus::Missing => (
            "\u{f034}",
            tr!(app.ui.translator, "containers.placeholder_not_installed"),
            tr!(app.ui.translator, "containers.placeholder_install_hint"),
            THEME.danger,
        ),
        DockerStatus::Off => (
            "\u{f011}",
            tr!(app.ui.translator, "containers.placeholder_stopped"),
            tr!(app.ui.translator, "containers.placeholder_start_hint"),
            THEME.danger,
        ),
        DockerStatus::Starting => (
            "\u{f251}",
            tr!(app.ui.translator, "containers.placeholder_starting"),
            tr!(app.ui.translator, "containers.placeholder_wait_hint"),
            THEME.warning,
        ),
        DockerStatus::On => (
            "\u{f308}",
            tr!(app.ui.translator, "containers.placeholder_no_containers"),
            tr!(app.ui.translator, "containers.placeholder_create_hint"),
            THEME.secondary,
        ),
        DockerStatus::Unknown => (
            "\u{f128}",
            tr!(app.ui.translator, "containers.placeholder_unknown"),
            tr!(app.ui.translator, "containers.placeholder_refresh_hint"),
            THEME.warning,
        ),
    };
    render_placeholder_lines(f, area, icon, title, hint, color);
}

fn render_placeholder_lines(
    f: &mut ratatui::Frame,
    area: Rect,
    icon: &str,
    title: String,
    hint: String,
    color: ratatui::style::Color,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Percentage(30),
        ])
        .split(area);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{}  ", icon),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                title,
                Style::default()
                    .fg(THEME.text_main)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(Alignment::Center),
        rows[1],
    );
    f.render_widget(
        Paragraph::new(Span::styled(hint, Style::default().fg(THEME.text_dim)))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        rows[2],
    );
}

fn action_item<'a>(
    icon: &'a str,
    lbl: String,
    key: &'a str,
    icon_color: ratatui::style::Color,
    selected: bool,
) -> ListItem<'a> {
    let prefix = if selected { " \u{258e}" } else { "  " };
    let name_style = if selected {
        Style::default()
            .fg(THEME.background)
            .bg(THEME.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.text_main)
    };
    ListItem::new(vec![
        Line::from(vec![
            Span::styled(prefix, Style::default().fg(THEME.primary)),
            Span::styled(format!(" {} ", icon), Style::default().fg(icon_color)),
            Span::styled(lbl, name_style),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled(format!("[ {} ]", key), Style::default().fg(THEME.text_dim)),
        ]),
    ])
}

fn render_centered(f: &mut ratatui::Frame, app: &App, area: Rect, key: &str) {
    f.render_widget(
        Paragraph::new(app.ui.translator.get(key).to_string())
            .alignment(Alignment::Center)
            .style(Style::default().fg(THEME.text_dim))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn label<'a>(app: &'a App, key: &'a str) -> Span<'a> {
    Span::styled(
        format!("{} ", app.ui.translator.get(key)),
        Style::default()
            .fg(THEME.primary)
            .add_modifier(Modifier::BOLD),
    )
}

fn state_badge_styled(app: &App, container: &ContainerInfo) -> (String, ratatui::style::Color) {
    if container.state.eq_ignore_ascii_case("running") {
        (
            format!(
                "\u{25b6} {}  ",
                tr!(app.ui.translator, "containers.state_run")
            ),
            THEME.success,
        )
    } else if container.state.eq_ignore_ascii_case("paused") {
        (
            format!(
                "\u{23f8} {}",
                tr!(app.ui.translator, "containers.state_pause")
            ),
            THEME.warning,
        )
    } else if container.state.is_empty() {
        (
            format!("? {}  ", tr!(app.ui.translator, "containers.state_unk")),
            THEME.text_dim,
        )
    } else {
        let s = container.state.to_uppercase();
        let truncated = if s.len() > 5 {
            format!("{}\u{2026}", &s[..5])
        } else {
            format!("{:<5}", s)
        };
        (format!("\u{25a0} {}", truncated), THEME.danger)
    }
}

fn container_counts(containers: &[ContainerInfo]) -> (usize, usize, usize) {
    let running = containers
        .iter()
        .filter(|c| c.state.eq_ignore_ascii_case("running"))
        .count();
    let paused = containers
        .iter()
        .filter(|c| c.state.eq_ignore_ascii_case("paused"))
        .count();
    let stopped = containers.len() - running - paused;
    (running, stopped, paused)
}

fn gauge_color(ratio: f64) -> ratatui::style::Color {
    if ratio >= 0.75 {
        THEME.danger
    } else if ratio >= 0.45 {
        THEME.warning
    } else {
        THEME.success
    }
}

fn focus_color(focused: bool) -> ratatui::style::Color {
    if focused {
        THEME.primary
    } else {
        THEME.secondary
    }
}

fn styled_block<'a>(title: String, color: ratatui::style::Color, focused: bool) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(color))
        .border_type(if focused {
            BorderType::Thick
        } else {
            BorderType::Rounded
        })
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() > max && max > 1 {
        format!("{}\u{2026}", &s[..max.saturating_sub(1)])
    } else {
        s.to_string()
    }
}

fn render_scrollbar(f: &mut ratatui::Frame, area: Rect, pct: u16) {
    if area.height < 3 {
        return;
    }
    let track_h = area.height as usize;
    let thumb_pos = (pct as usize * track_h / 100).min(track_h.saturating_sub(1));
    let lines: Vec<Line> = (0..track_h)
        .map(|i| {
            let ch = if i == thumb_pos {
                "\u{2588}"
            } else {
                "\u{2591}"
            };
            Line::from(Span::styled(ch, Style::default().fg(THEME.secondary)))
        })
        .collect();
    f.render_widget(Paragraph::new(lines), area);
}

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}
