use super::theme::THEME;
use crate::app::App;
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

pub fn render_trends_view(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),      // Summary cards
            Constraint::Percentage(50), // Charts
            Constraint::Min(0),         // Distribution tables
        ])
        .split(area);

    render_summary_cards(f, app, sections[0]);
    render_charts(f, app, sections[1]);
    render_distributions(f, app, sections[2]);
}

fn render_summary_cards(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let active_conns = app.conn_count_history.last().map(|&c| c).unwrap_or(0);
    let peak_conns = app.conn_count_history.iter().max().copied().unwrap_or(0);
    let peak_cpu = app.cpu_history.iter().copied().fold(0.0, f64::max);
    let active_processes = app.processes.len();

    render_summary_card(
        f,
        app,
        cards[0],
        "trends.active_connections",
        format!("{}", active_conns),
        THEME.primary,
    );
    render_summary_card(
        f,
        app,
        cards[1],
        "trends.peak_connections",
        format!("{}", peak_conns),
        THEME.success,
    );
    render_summary_card(
        f,
        app,
        cards[2],
        "trends.peak_cpu",
        format!("{:.1}%", peak_cpu),
        THEME.warning,
    );
    render_summary_card(
        f,
        app,
        cards[3],
        "trends.active_processes",
        format!("{}", active_processes),
        THEME.secondary,
    );
}

fn render_summary_card(
    f: &mut ratatui::Frame,
    app: &App,
    area: Rect,
    label_key: &str,
    value: String,
    color: ratatui::style::Color,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .border_type(BorderType::Rounded);

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let label_line = Line::from(Span::styled(
        app.translator.get(label_key),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ));

    let value_line = Line::from(Span::styled(
        value,
        Style::default()
            .fg(THEME.text_main)
            .add_modifier(Modifier::BOLD),
    ));

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    f.render_widget(
        Paragraph::new(label_line).alignment(Alignment::Center),
        rows[0],
    );
    f.render_widget(
        Paragraph::new(value_line).alignment(Alignment::Center),
        rows[1],
    );
}

fn render_charts(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let charts = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_connections_chart(f, app, charts[0]);
    render_cpu_chart(f, app, charts[1]);
}

fn render_connections_chart(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!(
            " {} ",
            tr!(app.translator, "trends.connections_chart_title")
        ))
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Rounded);

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    if app.conn_count_history.is_empty() {
        f.render_widget(
            Paragraph::new(tr!(app.translator, "trends.empty_data"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            inner,
        );
        return;
    }

    let data: Vec<f64> = app.conn_count_history.iter().map(|&x| x as f64).collect();
    render_simple_chart(f, app, inner, &data, THEME.success);
}

fn render_cpu_chart(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!(
            " {} ",
            tr!(app.translator, "trends.cpu_chart_title")
        ))
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Rounded);

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    if app.cpu_history.is_empty() {
        f.render_widget(
            Paragraph::new(tr!(app.translator, "trends.empty_data"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            inner,
        );
        return;
    }

    render_simple_chart(f, app, inner, &app.cpu_history, THEME.warning);
}

fn render_simple_chart(
    f: &mut ratatui::Frame,
    _app: &App,
    area: Rect,
    data: &[f64],
    color: ratatui::style::Color,
) {
    if area.height < 3 || area.width < 10 {
        return;
    }

    let width = area.width as usize - 2;
    let height = (area.height - 2) as usize;

    let max_val = data.iter().copied().fold(0.0, f64::max).max(1.0);
    let mut chart_data = vec![String::new(); height];

    let step = if data.len() > width {
        data.len() / width
    } else {
        1
    };

    for (_i, chunk) in data.chunks(step).enumerate().take(width) {
        let avg = chunk.iter().sum::<f64>() / chunk.len() as f64;
        let normalized = ((avg / max_val) * height as f64) as usize;

        for row in 0..height {
            if row >= height - normalized.min(height) {
                chart_data[row].push('█');
            } else {
                chart_data[row].push(' ');
            }
        }
    }

    let lines: Vec<Line> = chart_data
        .iter()
        .map(|row| Line::from(Span::styled(row.clone(), Style::default().fg(color))))
        .collect();

    f.render_widget(
        Paragraph::new(lines).alignment(Alignment::Left),
        ratatui::layout::Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        },
    );
}

fn render_distributions(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let tables = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_protocol_distribution(f, app, tables[0]);
    render_country_distribution(f, app, tables[1]);
}

fn render_protocol_distribution(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!(
            " {} ",
            tr!(app.translator, "trends.protocol_distribution")
        ))
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Rounded);

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let mut protocol_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for app_conn in &app.app_connections {
        for conn in &app_conn.connections {
            *protocol_counts.entry(conn.protocol.clone()).or_insert(0) += 1;
        }
    }

    let rows: Vec<Row> = protocol_counts
        .iter()
        .map(|(proto, count)| {
            Row::new(vec![
                Cell::from(Span::styled(
                    proto.clone(),
                    Style::default().fg(THEME.text_main),
                )),
                Cell::from(Span::styled(
                    format!("{}", count),
                    Style::default().fg(THEME.success),
                )),
            ])
        })
        .collect();

    if rows.is_empty() {
        f.render_widget(
            Paragraph::new(tr!(app.translator, "trends.empty_data"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            inner,
        );
        return;
    }

    let header = Row::new(vec!["Protocol", "Count"]).style(
        Style::default()
            .fg(THEME.text_dim)
            .add_modifier(Modifier::BOLD),
    );

    let table = Table::new(
        rows,
        [Constraint::Percentage(60), Constraint::Percentage(40)],
    )
    .header(header);

    f.render_widget(table, inner);
}

fn render_country_distribution(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!(
            " {} ",
            tr!(app.translator, "trends.country_distribution")
        ))
        .title_style(
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Rounded);

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let mut country_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for app_conn in &app.app_connections {
        for conn in &app_conn.connections {
            // Extract country from foreign address if available
            if let Some(addr_parts) = conn.foreign_address.split_whitespace().next() {
                let country = format!("Country: {}", addr_parts);
                *country_counts.entry(country).or_insert(0) += 1;
            }
        }
    }

    let mut sorted: Vec<_> = country_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    let rows: Vec<Row> = sorted
        .iter()
        .take(10)
        .map(|(country, count)| {
            Row::new(vec![
                Cell::from(Span::styled(
                    country.to_string(),
                    Style::default().fg(THEME.text_main),
                )),
                Cell::from(Span::styled(
                    format!("{}", count),
                    Style::default().fg(THEME.warning),
                )),
            ])
        })
        .collect();

    if rows.is_empty() {
        f.render_widget(
            Paragraph::new(tr!(app.translator, "trends.empty_data"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            inner,
        );
        return;
    }

    let header = Row::new(vec!["Country", "Count"]).style(
        Style::default()
            .fg(THEME.text_dim)
            .add_modifier(Modifier::BOLD),
    );

    let table = Table::new(
        rows,
        [Constraint::Percentage(60), Constraint::Percentage(40)],
    )
    .header(header);

    f.render_widget(table, inner);
}
