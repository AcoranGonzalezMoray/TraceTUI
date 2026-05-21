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

    let container_name = app
        .get_selected_container()
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " 📋 {} - {} ",
            tr!(app.translator, "containers.logs_title"),
            container_name
        ))
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Double);
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

    let total_lines = app.container_logs.len();
    let visible = inner.height as usize;
    let scroll_info = format!(
        " [{}/{}] ",
        (app.container_logs_scroll + visible).min(total_lines),
        total_lines
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let lines: Vec<Line> = app
        .container_logs
        .iter()
        .skip(app.container_logs_scroll)
        .take(visible)
        .enumerate()
        .map(|(idx, line)| {
            let line_num = format!("{:4} │ ", app.container_logs_scroll + idx + 1);
            Line::from(vec![
                Span::styled(line_num, Style::default().fg(THEME.text_dim)),
                Span::styled(line.as_str(), Style::default().fg(THEME.text_main)),
            ])
        })
        .collect();
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), chunks[0]);

    let scroll_bar = Paragraph::new(Span::styled(
        scroll_info,
        Style::default()
            .fg(THEME.secondary)
            .add_modifier(Modifier::DIM),
    ))
    .alignment(Alignment::Right);
    f.render_widget(scroll_bar, chunks[1]);
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
        .title(format!(" 🖥️  {} ", title))
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Double);

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(4)])
        .split(inner);

    let visible = chunks[0].height as usize;
    let lines: Vec<Line> = app
        .container_console_output
        .iter()
        .skip(app.container_console_scroll)
        .take(visible)
        .enumerate()
        .map(|(idx, line)| {
            Line::from(vec![
                Span::styled(
                    format!("{:3} │ ", app.container_console_scroll + idx + 1),
                    Style::default().fg(THEME.text_dim),
                ),
                Span::styled(line.as_str(), Style::default().fg(THEME.text_main)),
            ])
        })
        .collect();
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), chunks[0]);

    let prompt = if app.container_console_loading {
        tr!(app.translator, "containers.console_running")
    } else {
        app.container_console_input.clone()
    };
    let input = Line::from(vec![
        Span::styled(
            "$ ",
            Style::default()
                .fg(THEME.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(prompt, Style::default().fg(THEME.text_main)),
        Span::styled(
            " ▌",
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let input_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(THEME.secondary))
        .title(format!(
            " {} ",
            tr!(app.translator, "containers.console_input_hint")
        ))
        .title_style(Style::default().fg(THEME.secondary));
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
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.secondary))
        .title(format!(
            " {} ",
            tr!(app.translator, "containers.identity_section")
        ))
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

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
            Span::styled(
                container.status.as_str(),
                Style::default().fg(if container.state.eq_ignore_ascii_case("running") {
                    THEME.success
                } else {
                    THEME.warning
                }),
            ),
        ]),
        Line::from(vec![
            label(app, "containers.field_ports"),
            Span::raw(if container.ports.is_empty() {
                "None"
            } else {
                &container.ports
            }),
        ]),
        Line::from(vec![
            label(app, "containers.field_networks"),
            Span::raw(if container.networks.is_empty() {
                "None"
            } else {
                &container.networks
            }),
        ]),
    ];
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn render_usage(f: &mut ratatui::Frame, app: &App, container: &ContainerInfo, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.secondary))
        .title(format!(
            " {} ",
            tr!(app.translator, "containers.usage_section")
        ))
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let gauges = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);
    let cpu = (container.cpu_percent.unwrap_or(0.0) / 100.0).clamp(0.0, 1.0);
    let mem = (container.memory_percent.unwrap_or(0.0) / 100.0).clamp(0.0, 1.0);
    f.render_widget(
        Gauge::default()
            .block(
                Block::default()
                    .title(tr!(app.translator, "containers.cpu"))
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(if cpu > 0.75 {
                THEME.danger
            } else if cpu > 0.5 {
                THEME.warning
            } else {
                THEME.success
            }))
            .ratio(cpu)
            .label(format!("{:.1}%", container.cpu_percent.unwrap_or(0.0))),
        gauges[0],
    );
    f.render_widget(
        Gauge::default()
            .block(
                Block::default()
                    .title(tr!(app.translator, "containers.memory"))
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(if mem > 0.75 {
                THEME.warning
            } else if mem > 0.5 {
                THEME.secondary
            } else {
                THEME.success
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
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.secondary))
        .title(format!(
            " {} ",
            tr!(app.translator, "containers.runtime_section")
        ))
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let rows = vec![
        Row::new(vec![
            Cell::from(tr!(app.translator, "containers.field_created")),
            Cell::from(container.created.clone()),
        ])
        .style(Style::default().fg(THEME.text_dim)),
        Row::new(vec![
            Cell::from(tr!(app.translator, "containers.field_running_for")),
            Cell::from(container.running_for.clone()),
        ])
        .style(Style::default().fg(THEME.text_main)),
        Row::new(vec![
            Cell::from(tr!(app.translator, "containers.field_net_io")),
            Cell::from(container.net_io.clone()),
        ])
        .style(Style::default().fg(THEME.text_dim)),
        Row::new(vec![
            Cell::from(tr!(app.translator, "containers.field_block_io")),
            Cell::from(container.block_io.clone()),
        ])
        .style(Style::default().fg(THEME.text_main)),
        Row::new(vec![
            Cell::from(tr!(app.translator, "containers.field_size")),
            Cell::from(container.size.clone()),
        ])
        .style(Style::default().fg(THEME.text_dim)),
        Row::new(vec![
            Cell::from(tr!(app.translator, "containers.field_pids")),
            Cell::from(container.pids.clone()),
        ])
        .style(Style::default().fg(THEME.text_main)),
    ];
    let table = Table::new(rows, [Constraint::Length(16), Constraint::Min(0)])
        .style(Style::default().fg(THEME.text_dim));
    f.render_widget(table, inner);
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
        (
            "\u{e28b}",
            tr!(app.translator, "containers.docker_hub_search_title"),
            "H",
            THEME.secondary,
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

pub fn render_docker_hub_modal(f: &mut ratatui::Frame, app: &App) {
    let area = centered_rect(f.area(), 90, 75);
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " {} ",
            tr!(app.translator, "containers.docker_hub_search_title")
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

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(inner);

    render_docker_hub_search_input(f, app, chunks[0]);
    render_docker_hub_content(f, app, chunks[1]);
    render_docker_hub_buttons(f, app, chunks[2]);
}

fn render_docker_hub_search_input(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.docker_hub_search.focused_field == 0;
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
        });

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let query_text = if is_focused && app.docker_hub_search.search_query.is_empty() {
        app.translator
            .get("containers.docker_hub_search_placeholder")
            .to_string()
    } else {
        app.docker_hub_search.search_query.clone()
    };

    let cursor = if is_focused {
        format!("{}_", query_text)
    } else {
        query_text
    };

    let line = Line::from(Span::styled(
        cursor,
        Style::default().fg(if is_focused {
            THEME.text_main
        } else {
            THEME.text_dim
        }),
    ));

    f.render_widget(Paragraph::new(line).wrap(Wrap { trim: true }), inner);
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
        .border_type(BorderType::Rounded);

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    if app.docker_hub_search.results.is_empty() {
        let msg = if app.docker_hub_search.search_query.is_empty() {
            "containers.docker_hub_search_placeholder"
        } else {
            "containers.docker_hub_no_results"
        };
        f.render_widget(
            Paragraph::new(app.translator.get(msg).to_string())
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim))
                .wrap(Wrap { trim: true }),
            inner,
        );
        return;
    }

    let items: Vec<ListItem> = app
        .docker_hub_search
        .results
        .iter()
        .enumerate()
        .map(|(i, img)| {
            let is_selected = i == app.docker_hub_search.selected_result_index;
            let prefix = if is_selected { " ▎ " } else { "   " };
            let style = if is_selected {
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_main)
            };

            ListItem::new(Line::from(vec![
                Span::raw(prefix),
                Span::styled(img.name.clone(), style),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.docker_hub_search.selected_result_index));
    f.render_stateful_widget(List::new(items), inner, &mut state);
}

fn render_docker_hub_form(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary))
        .border_type(BorderType::Rounded);

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
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
        &app.docker_hub_search.container_name,
    );

    render_form_field(
        f,
        app,
        rows[1],
        2,
        "containers.docker_hub_ports_field",
        &app.docker_hub_search.ports,
    );

    render_form_field(
        f,
        app,
        rows[2],
        3,
        "containers.docker_hub_env_field",
        &app.docker_hub_search.env_vars,
    );
}

fn render_form_field(
    f: &mut ratatui::Frame,
    app: &App,
    area: Rect,
    field_index: usize,
    label_key: &str,
    value: &str,
) {
    let is_focused = app.docker_hub_search.focused_field == field_index;
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

    let label = Span::styled(
        format!("{} ", app.translator.get(label_key)),
        Style::default()
            .fg(THEME.primary)
            .add_modifier(Modifier::BOLD),
    );

    let cursor = if is_focused {
        format!("{}_", value)
    } else {
        value.to_string()
    };

    let value_span = Span::styled(
        cursor,
        Style::default().fg(if is_focused {
            THEME.text_main
        } else {
            THEME.text_dim
        }),
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![label, value_span])).wrap(Wrap { trim: true }),
        inner,
    );
}

fn render_docker_hub_buttons(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let create_focused = app.docker_hub_search.focused_field == 4;
    let create_style = if create_focused {
        Style::default()
            .fg(THEME.background)
            .bg(THEME.success)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.success)
    };

    f.render_widget(
        Paragraph::new(Line::from(Span::styled("[ CREATE ]", create_style)))
            .alignment(Alignment::Center),
        button_chunks[0],
    );

    let cancel_focused = app.docker_hub_search.focused_field == 5;
    let cancel_style = if cancel_focused {
        Style::default()
            .fg(THEME.background)
            .bg(THEME.danger)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.danger)
    };

    f.render_widget(
        Paragraph::new(Line::from(Span::styled("[ CANCEL ]", cancel_style)))
            .alignment(Alignment::Center),
        button_chunks[1],
    );
}
