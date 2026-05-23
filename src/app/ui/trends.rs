use super::theme::THEME;
use crate::app::App;
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Sparkline, Table},
};

pub fn render_trends_view(f: &mut ratatui::Frame, app: &App, area: Rect) {
    if area.height < 10 || area.width < 30 {
        return;
    }

    if !app.auto_analysis_complete {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let s = spinner[(app.frame_count as usize) % spinner.len()];
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(THEME.secondary));
        f.render_widget(block.clone(), area);
        let inner = block.inner(area);
        let msg = format!(" {} {}...", s, tr!(app.translator, "status.auto_analyzing"));
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                msg,
                Style::default().fg(THEME.warning),
            )))
            .alignment(Alignment::Center),
            inner,
        );
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(8),
            Constraint::Percentage(40),
            Constraint::Min(0),
        ])
        .split(area);

    render_summary_cards(f, app, chunks[0]);
    render_sparkline_row(f, app, chunks[1]);
    render_middle_row(f, app, chunks[2]);
    render_bottom_row(f, app, chunks[3]);
}

fn render_summary_cards(f: &mut ratatui::Frame, app: &App, area: Rect) {
    if area.width < 40 {
        return;
    }

    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(area);

    let active_conns: u64 = app
        .app_connections
        .iter()
        .map(|a| a.connections.len() as u64)
        .sum();

    let peak_conn = app.conn_count_history.iter().max().copied().unwrap_or(0);

    let current_cpu = app.cpu_history.last().copied().unwrap_or(0.0);

    let total_mem_mb: u64 = app
        .app_connections
        .iter()
        .map(|a| a.memory_usage / 1024 / 1024)
        .sum();

    let high_risk_count = app
        .app_connections
        .iter()
        .filter(|a| a.risk_level.contains("HIGH") || a.risk_level.contains("CRITICAL"))
        .count() as u64;

    render_kpi_card(
        f,
        cards[0],
        "CONNECTIONS",
        &fmt_num(active_conns),
        THEME.primary,
        "▲",
    );
    render_kpi_card(
        f,
        cards[1],
        "PEAK CONNS",
        &fmt_num(peak_conn),
        THEME.success,
        "◆",
    );
    render_kpi_card(
        f,
        cards[2],
        "CPU USAGE",
        &format!("{:.1}%", current_cpu),
        cpu_color(current_cpu),
        "◈",
    );
    render_kpi_card(
        f,
        cards[3],
        "MEM USAGE",
        &format!("{} MB", fmt_num(total_mem_mb)),
        THEME.secondary,
        "▣",
    );
    render_kpi_card(
        f,
        cards[4],
        "HIGH RISK",
        &fmt_num(high_risk_count),
        if high_risk_count > 0 {
            THEME.danger
        } else {
            THEME.success
        },
        "⚠",
    );
}

fn render_kpi_card(
    f: &mut ratatui::Frame,
    area: Rect,
    label: &str,
    value: &str,
    color: ratatui::style::Color,
    icon: &str,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .border_type(BorderType::Rounded);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);
    if inner.height < 2 {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{} ", icon),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                label,
                Style::default()
                    .fg(THEME.text_dim)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(Alignment::Center),
        rows[0],
    );

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            value.to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center),
        rows[1],
    );
}

fn render_sparkline_row(f: &mut ratatui::Frame, app: &App, area: Rect) {
    if area.width < 20 || area.height < 3 {
        return;
    }

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_sparkline_panel(
        f,
        cols[0],
        "  CPU HISTORY",
        &cpu_to_u64(&app.cpu_history),
        THEME.warning,
        app.cpu_history
            .last()
            .copied()
            .map(|v| format!("{:.1}%", v)),
    );
    render_sparkline_panel(
        f,
        cols[1],
        "  CONNECTION HISTORY",
        &app.conn_count_history,
        THEME.primary,
        app.conn_count_history
            .last()
            .copied()
            .map(|v| format!("{} active", v)),
    );
}

fn render_sparkline_panel(
    f: &mut ratatui::Frame,
    area: Rect,
    title: &str,
    data: &[u64],
    color: ratatui::style::Color,
    current_label: Option<String>,
) {
    let title_str = if let Some(ref lbl) = current_label {
        format!("{} ─ {}", title, lbl)
    } else {
        title.to_string()
    };

    let block = Block::default()
        .title(Span::styled(
            title_str,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary))
        .border_type(BorderType::Rounded);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);
    if inner.height < 1 || data.is_empty() {
        return;
    }

    let stats_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 1,
    };
    let spark_area = Rect {
        x: inner.x,
        y: inner.y + 1,
        width: inner.width,
        height: inner.height.saturating_sub(1),
    };

    if !data.is_empty() {
        let min = data.iter().min().copied().unwrap_or(0);
        let max = data.iter().max().copied().unwrap_or(0);
        let sum: u64 = data.iter().sum();
        let avg = sum / data.len().max(1) as u64;

        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(" min:", Style::default().fg(THEME.text_dim)),
                Span::styled(format!("{} ", min), Style::default().fg(THEME.success)),
                Span::styled("avg:", Style::default().fg(THEME.text_dim)),
                Span::styled(format!("{} ", avg), Style::default().fg(THEME.warning)),
                Span::styled("max:", Style::default().fg(THEME.text_dim)),
                Span::styled(format!("{}", max), Style::default().fg(THEME.danger)),
            ])),
            stats_area,
        );
    }

    if spark_area.height > 0 {
        f.render_widget(
            Sparkline::default()
                .data(data)
                .style(Style::default().fg(color)),
            spark_area,
        );
    }
}

fn render_middle_row(f: &mut ratatui::Frame, app: &App, area: Rect) {
    if area.height < 4 {
        return;
    }

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    render_risk_distribution(f, app, cols[0]);
    render_top_processes_cpu(f, app, cols[1]);
    render_top_processes_mem(f, app, cols[2]);
}

fn render_risk_distribution(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            "  RISK DISTRIBUTION",
            Style::default()
                .fg(THEME.danger)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary))
        .border_type(BorderType::Rounded);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);
    if inner.height < 2 {
        return;
    }

    let mut critical = 0u64;
    let mut high = 0u64;
    let mut medium = 0u64;
    let mut low = 0u64;
    let mut safe = 0u64;

    for app_conn in &app.app_connections {
        let rl = app_conn.risk_level.to_uppercase();
        if rl.contains("CRITICAL") {
            critical += 1;
        } else if rl.contains("HIGH") {
            high += 1;
        } else if rl.contains("MEDIUM") || rl.contains("MED") {
            medium += 1;
        } else if rl.contains("LOW") {
            low += 1;
        } else {
            safe += 1;
        }
    }

    let total = (critical + high + medium + low + safe).max(1);
    let items = [
        ("CRITICAL", critical, THEME.danger),
        ("HIGH    ", high, THEME.danger),
        ("MEDIUM  ", medium, THEME.warning),
        ("LOW     ", low, THEME.success),
        ("SAFE    ", safe, THEME.primary),
    ];

    let bar_max = inner.width.saturating_sub(16) as usize;

    let mut rows: Vec<Row> = Vec::new();
    for (label, count, color) in &items {
        let pct = *count as f64 / total as f64;
        let bar_len = (pct * bar_max as f64) as usize;
        let bar: String = std::iter::repeat('█').take(bar_len).collect();
        let pct_str = format!("{:>3.0}%", pct * 100.0);

        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                *label,
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            )),
            Cell::from(Line::from(vec![
                Span::styled(bar, Style::default().fg(*color)),
                Span::styled(
                    format!(" {} ({})", count, pct_str),
                    Style::default().fg(THEME.text_dim),
                ),
            ])),
        ]));
    }

    f.render_widget(
        Table::new(rows, [Constraint::Length(9), Constraint::Min(0)]),
        inner,
    );
}

fn render_top_processes_cpu(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            "  TOP PROCESSES — CPU",
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary))
        .border_type(BorderType::Rounded);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);
    if inner.height < 2 {
        return;
    }

    let mut procs: Vec<(&str, f32)> = app
        .app_connections
        .iter()
        .map(|a| (a.process_name.as_str(), a.cpu_usage))
        .collect();
    procs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    procs.truncate(inner.height as usize);

    if procs.is_empty() {
        f.render_widget(
            Paragraph::new("No data")
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            inner,
        );
        return;
    }

    let max_cpu = procs
        .iter()
        .map(|(_, c)| *c)
        .fold(0.0f32, f32::max)
        .max(0.01);
    let name_w = (inner.width as usize / 3).max(8).min(22);
    let bar_max = inner.width.saturating_sub(name_w as u16 + 10) as usize;

    let header = Row::new(vec![
        Cell::from(Span::styled(
            "PROCESS",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Cell::from(Span::styled(
            "CPU %",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
    ]);

    let mut rows: Vec<Row> = vec![header];
    for (name, cpu) in &procs {
        let bar_len = ((*cpu / max_cpu) as f64 * bar_max as f64) as usize;
        let bar: String = std::iter::repeat(bar_char(*cpu / max_cpu))
            .take(bar_len)
            .collect();
        let color = cpu_color(*cpu as f64);
        let short_name = truncate_str(name, name_w);

        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                format!("{:<width$}", short_name, width = name_w),
                Style::default().fg(THEME.text_main),
            )),
            Cell::from(Line::from(vec![
                Span::styled(bar, Style::default().fg(color)),
                Span::styled(
                    format!(" {:.1}%", cpu),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
            ])),
        ]));
    }

    f.render_widget(
        Table::new(
            rows,
            [Constraint::Length(name_w as u16), Constraint::Min(0)],
        ),
        inner,
    );
}

fn render_top_processes_mem(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            "  TOP PROCESSES — MEM",
            Style::default()
                .fg(THEME.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary))
        .border_type(BorderType::Rounded);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);
    if inner.height < 2 {
        return;
    }

    let mut procs: Vec<(&str, u64)> = app
        .app_connections
        .iter()
        .map(|a| (a.process_name.as_str(), a.memory_usage / 1024 / 1024))
        .collect();
    procs.sort_by(|a, b| b.1.cmp(&a.1));
    procs.truncate(inner.height as usize);

    if procs.is_empty() {
        f.render_widget(
            Paragraph::new("No data")
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            inner,
        );
        return;
    }

    let max_mem = procs.iter().map(|(_, m)| *m).max().unwrap_or(1).max(1);
    let name_w = (inner.width as usize / 3).max(8).min(22);
    let bar_max = inner.width.saturating_sub(name_w as u16 + 10) as usize;

    let header = Row::new(vec![
        Cell::from(Span::styled(
            "PROCESS",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Cell::from(Span::styled(
            "MEM MB",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
    ]);

    let mut rows: Vec<Row> = vec![header];
    for (name, mem_mb) in &procs {
        let ratio = *mem_mb as f64 / max_mem as f64;
        let bar_len = (ratio * bar_max as f64) as usize;
        let bar: String = std::iter::repeat(bar_char(ratio as f32))
            .take(bar_len)
            .collect();
        let color = mem_color(*mem_mb);
        let short_name = truncate_str(name, name_w);

        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                format!("{:<width$}", short_name, width = name_w),
                Style::default().fg(THEME.text_main),
            )),
            Cell::from(Line::from(vec![
                Span::styled(bar, Style::default().fg(color)),
                Span::styled(
                    format!(" {} MB", fmt_num(*mem_mb)),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
            ])),
        ]));
    }

    f.render_widget(
        Table::new(
            rows,
            [Constraint::Length(name_w as u16), Constraint::Min(0)],
        ),
        inner,
    );
}

fn render_bottom_row(f: &mut ratatui::Frame, app: &App, area: Rect) {
    if area.height < 4 {
        return;
    }

    let show_containers = !app.containers.is_empty() || app.containers_loaded_once;

    let constraints = if show_containers {
        vec![
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ]
    } else {
        vec![Constraint::Percentage(50), Constraint::Percentage(50)]
    };

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    render_protocol_dist(f, app, cols[0]);
    render_country_dist(f, app, cols[1]);
    if show_containers && cols.len() > 2 {
        render_containers_panel(f, app, cols[2]);
    }
}

fn render_protocol_dist(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let mut counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    for app_conn in &app.app_connections {
        for conn in &app_conn.connections {
            *counts.entry(conn.protocol.clone()).or_insert(0) += 1;
        }
    }

    let mut items: Vec<(String, u64)> = counts.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));

    render_dist_table(
        f,
        area,
        "  PROTOCOLS",
        "PROTO",
        "CONNS",
        &items,
        THEME.success,
    );
}

fn render_country_dist(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let mut counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    for app_conn in &app.app_connections {
        for conn in &app_conn.connections {
            let key = conn
                .location
                .as_deref()
                .map(|s| s.split(',').last().unwrap_or(s).trim().to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            *counts.entry(key).or_insert(0) += 1;
        }
    }

    let mut items: Vec<(String, u64)> = counts.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));
    items.truncate(10);

    render_dist_table(
        f,
        area,
        "  DESTINATIONS",
        "COUNTRY / IP",
        "CONNS",
        &items,
        THEME.warning,
    );
}

fn render_containers_panel(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(Span::styled(
            "  CONTAINERS",
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary))
        .border_type(BorderType::Rounded);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);
    if inner.height < 1 {
        return;
    }

    if app.containers.is_empty() {
        let msg = if app.containers_loading {
            "Loading…"
        } else {
            "No containers"
        };
        f.render_widget(
            Paragraph::new(msg)
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            inner,
        );
        return;
    }

    let running = app
        .containers
        .iter()
        .filter(|c| {
            c.status.to_lowercase().contains("running") || c.status.to_lowercase().contains("up")
        })
        .count();
    let total = app.containers.len();
    let stopped = total - running;

    let summary_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 1,
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("▶ Running: ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                format!("{}", running),
                Style::default()
                    .fg(THEME.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ■ Stopped: ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                format!("{}", stopped),
                Style::default()
                    .fg(THEME.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  / {}", total),
                Style::default().fg(THEME.text_dim),
            ),
        ])),
        summary_area,
    );

    if inner.height < 3 {
        return;
    }

    let list_area = Rect {
        x: inner.x,
        y: inner.y + 1,
        width: inner.width,
        height: inner.height - 1,
    };

    let name_w = (inner.width.saturating_sub(12)) as usize;
    let header = Row::new(vec![
        Cell::from(Span::styled(
            "STATUS",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Cell::from(Span::styled(
            "NAME",
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
    ]);

    let mut rows: Vec<Row> = vec![header];
    for container in app
        .containers
        .iter()
        .take(list_area.height.saturating_sub(1) as usize)
    {
        let is_running = container.status.to_lowercase().contains("running")
            || container.status.to_lowercase().contains("up");
        let (status_icon, color) = if is_running {
            ("▶ UP  ", THEME.success)
        } else {
            ("■ DOWN", THEME.danger)
        };
        let short_name = truncate_str(&container.name, name_w);
        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                status_icon,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                short_name,
                Style::default().fg(THEME.text_main),
            )),
        ]));
    }

    f.render_widget(
        Table::new(rows, [Constraint::Length(6), Constraint::Min(0)]),
        list_area,
    );
}

fn render_dist_table(
    f: &mut ratatui::Frame,
    area: Rect,
    title: &str,
    col1: &str,
    col2: &str,
    items: &[(String, u64)],
    color: ratatui::style::Color,
) {
    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary))
        .border_type(BorderType::Rounded);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);
    if inner.height < 2 {
        return;
    }

    if items.is_empty() {
        f.render_widget(
            Paragraph::new("No data")
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            inner,
        );
        return;
    }

    let max_val = items.iter().map(|(_, v)| *v).max().unwrap_or(1).max(1);
    let label_w = (inner.width as usize / 4).max(col1.len()).min(20);
    let bar_max = inner.width.saturating_sub(label_w as u16 + 8) as usize;

    let header = Row::new(vec![
        Cell::from(Span::styled(
            format!("{:<width$}", col1, width = label_w),
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Cell::from(Span::styled(
            col2,
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
    ]);

    let mut rows: Vec<Row> = vec![header];
    for (label, value) in items.iter().take(inner.height.saturating_sub(1) as usize) {
        let ratio = *value as f64 / max_val as f64;
        let bar_len = (ratio * bar_max as f64) as usize;
        let bar: String = std::iter::repeat(bar_char(ratio as f32))
            .take(bar_len)
            .collect();
        let short_label = truncate_str(label, label_w);

        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                format!("{:<width$}", short_label, width = label_w),
                Style::default()
                    .fg(THEME.text_dim)
                    .add_modifier(Modifier::BOLD),
            )),
            Cell::from(Line::from(vec![
                Span::styled(bar, Style::default().fg(color)),
                Span::styled(
                    format!(" {}", fmt_num(*value)),
                    Style::default().fg(THEME.text_main),
                ),
            ])),
        ]));
    }

    f.render_widget(
        Table::new(
            rows,
            [Constraint::Length(label_w as u16), Constraint::Min(0)],
        ),
        inner,
    );
}

fn fmt_num(n: u64) -> String {
    let s = n.to_string();
    let mut r = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            r.insert(0, ',');
        }
        r.insert(0, c);
    }
    r
}

fn cpu_to_u64(data: &[f64]) -> Vec<u64> {
    data.iter().map(|&x| x as u64).collect()
}

fn bar_char(ratio: f32) -> char {
    if ratio > 0.75 {
        '█'
    } else if ratio > 0.50 {
        '▓'
    } else if ratio > 0.25 {
        '▒'
    } else {
        '░'
    }
}

fn cpu_color(cpu: f64) -> ratatui::style::Color {
    if cpu >= 80.0 {
        THEME.danger
    } else if cpu >= 50.0 {
        THEME.warning
    } else {
        THEME.success
    }
}

fn mem_color(mem_mb: u64) -> ratatui::style::Color {
    if mem_mb >= 2048 {
        THEME.danger
    } else if mem_mb >= 512 {
        THEME.warning
    } else {
        THEME.success
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() > max && max > 1 {
        format!("{}…", &s[..max.saturating_sub(1)])
    } else {
        s.to_string()
    }
}
