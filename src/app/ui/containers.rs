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

pub fn render_container_logs_modal(f: &mut ratatui::Frame, app: &App) {
    let area = centered_rect(f.area(), 86, 78);
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " {} ",
            tr!(app.translator, "containers.logs_title")
        ))
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Thick);
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
        .skip(app.container_logs_scroll)
        .take(inner.height as usize)
        .map(|line| {
            Line::from(Span::styled(
                line.as_str(),
                Style::default().fg(THEME.text_dim),
            ))
        })
        .collect();
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

pub fn render_container_console_modal(f: &mut ratatui::Frame, app: &App) {
    let area = centered_rect(f.area(), 86, 78);
    f.render_widget(Clear, area);
    let title = app
        .get_selected_container()
        .map(|container| tr!(app.translator, "containers.console_title", &container.name))
        .unwrap_or_else(|| tr!(app.translator, "containers.docker_action_console"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Thick);

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(inner);

    let visible = chunks[0].height as usize;
    let lines: Vec<Line> = app
        .container_console_output
        .iter()
        .skip(app.container_console_scroll)
        .take(visible)
        .map(|line| {
            Line::from(Span::styled(
                line.as_str(),
                Style::default().fg(THEME.text_dim),
            ))
        })
        .collect();
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), chunks[0]);

    let prompt = if app.container_console_loading {
        tr!(app.translator, "containers.console_running")
    } else {
        app.container_console_input.clone()
    };
    let input = Line::from(vec![
        Span::styled("$ ", Style::default().fg(THEME.success)),
        Span::styled(prompt, Style::default().fg(THEME.text_main)),
        Span::styled(" ", Style::default().fg(THEME.primary)),
    ]);
    let input_block = Block::default()
        .borders(Borders::TOP)
        .title(format!(
            " {} ",
            tr!(app.translator, "containers.console_input_hint")
        ))
        .title_style(Style::default().fg(THEME.text_dim));
    f.render_widget(Paragraph::new(input).block(input_block), chunks[1]);
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
        render_docker_placeholder(f, app, inner, DockerStatus::Starting);
        return;
    }

    if app.containers_error.is_some() {
        render_docker_placeholder(f, app, inner, app.docker_status());
        return;
    }

    if app.containers.is_empty() {
        render_docker_empty(f, app, inner);
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
            Constraint::Length(10),
            Constraint::Percentage(38),
            Constraint::Percentage(44),
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
            Constraint::Length(7),
            Constraint::Min(0),
        ])
        .split(inner);

    render_identity(f, app, container, chunks[0]);
    render_usage(f, app, container, chunks[1]);
    render_runtime(f, app, container, chunks[2]);
    render_log_hint(f, app, chunks[3]);
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

fn render_log_hint(f: &mut ratatui::Frame, app: &App, area: Rect) {
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
    let text = if app.container_logs_loading {
        tr!(app.translator, "containers.logs_loading")
    } else {
        tr!(app.translator, "containers.logs_modal_hint")
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
        .constraints([Constraint::Min(12), Constraint::Length(8)])
        .split(area);
    render_container_actions(f, app, chunks[0]);
    render_docker_actions(f, app, chunks[1]);
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
            "\u{f021}",
            tr!(app.translator, "containers.action_refresh"),
            "R",
            THEME.text_dim,
        ),
        (
            "\u{f6ce}",
            tr!(app.translator, "containers.action_logs"),
            "L",
            THEME.secondary,
        ),
        (
            "\u{e795}",
            tr!(app.translator, "containers.action_console"),
            "C",
            THEME.secondary,
        ),
        (
            "\u{f04b}",
            tr!(app.translator, "containers.action_start"),
            "S",
            THEME.success,
        ),
        (
            "\u{f04d}",
            tr!(app.translator, "containers.action_stop"),
            "T",
            THEME.danger,
        ),
        (
            "\u{f021}",
            tr!(app.translator, "containers.action_restart"),
            "E",
            THEME.warning,
        ),
        ("\u{f04c}", pause_label, "P", THEME.primary),
    ];

    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(i, (icon, label, key, icon_color))| {
            action_item(
                *icon,
                label.clone(),
                *key,
                *icon_color,
                i == app.selected_container_action_index,
            )
        })
        .collect();

    let mut state = ListState::default();
    if app.selected_container_action_index < DOCKER_ACTION_OFFSET {
        state.select(Some(app.selected_container_action_index));
    }
    f.render_stateful_widget(List::new(items), inner, &mut state);
}

fn render_docker_actions(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let focused = app.sidebar_focus == SidebarFocus::Right;
    let selected = app
        .selected_container_action_index
        .saturating_sub(DOCKER_ACTION_OFFSET);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " {} ",
            tr!(app.translator, "containers.docker_actions_title")
        ))
        .title_style(
            Style::default()
                .fg(if focused {
                    THEME.primary
                } else {
                    THEME.secondary
                })
                .add_modifier(Modifier::BOLD),
        )
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

    let actions = vec![
        (
            "\u{f011}",
            tr!(app.translator, "containers.docker_action_start"),
            "N",
            THEME.success,
        ),
        (
            "\u{f011}",
            tr!(app.translator, "containers.docker_action_stop"),
            "O",
            THEME.danger,
        ),
    ];
    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(i, (icon, label, key, color))| {
            action_item(
                *icon,
                label.clone(),
                *key,
                *color,
                app.selected_container_action_index >= DOCKER_ACTION_OFFSET && i == selected,
            )
        })
        .collect();

    let mut state = ListState::default();
    if app.selected_container_action_index >= DOCKER_ACTION_OFFSET {
        state.select(Some(selected));
    }
    f.render_stateful_widget(List::new(items), inner, &mut state);
}

fn action_item<'a>(
    icon: &'a str,
    label: String,
    key: &'a str,
    icon_color: ratatui::style::Color,
    selected: bool,
) -> ListItem<'a> {
    let prefix = if selected { " ▎" } else { "  " };
    let title_style = if selected {
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
            Span::styled(label, title_style),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled(format!("[ {} ]", key), Style::default().fg(THEME.text_dim)),
        ]),
    ])
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

fn render_docker_empty(f: &mut ratatui::Frame, app: &App, area: Rect) {
    render_placeholder_lines(
        f,
        area,
        "\u{f308}",
        tr!(app.translator, "containers.placeholder_no_containers"),
        tr!(app.translator, "containers.placeholder_create_hint"),
        THEME.secondary,
    );
}

fn render_docker_placeholder(f: &mut ratatui::Frame, app: &App, area: Rect, status: DockerStatus) {
    let (icon, title, hint, color) = match status {
        DockerStatus::Missing => (
            "\u{f034}",
            tr!(app.translator, "containers.placeholder_not_installed"),
            tr!(app.translator, "containers.placeholder_install_hint"),
            THEME.danger,
        ),
        DockerStatus::Off => (
            "\u{f011}",
            tr!(app.translator, "containers.placeholder_stopped"),
            tr!(app.translator, "containers.placeholder_start_hint"),
            THEME.danger,
        ),
        DockerStatus::Starting => (
            "\u{f251}",
            tr!(app.translator, "containers.placeholder_starting"),
            tr!(app.translator, "containers.placeholder_wait_hint"),
            THEME.warning,
        ),
        DockerStatus::On => (
            "\u{f308}",
            tr!(app.translator, "containers.placeholder_no_containers"),
            tr!(app.translator, "containers.placeholder_create_hint"),
            THEME.secondary,
        ),
        DockerStatus::Unknown => (
            "\u{f128}",
            tr!(app.translator, "containers.placeholder_unknown"),
            tr!(app.translator, "containers.placeholder_refresh_hint"),
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
    let title_line = Line::from(vec![
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
    ]);
    let hint_line = Line::from(Span::styled(hint, Style::default().fg(THEME.text_dim)));
    f.render_widget(
        Paragraph::new(title_line).alignment(Alignment::Center),
        rows[1],
    );
    f.render_widget(
        Paragraph::new(hint_line)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        rows[2],
    );
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
