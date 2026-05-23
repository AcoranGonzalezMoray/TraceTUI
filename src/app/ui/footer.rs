use super::theme::THEME;
use crate::app::{App, NavView};
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn os_name(translator: &crate::i18n::Translator) -> String {
    if cfg!(target_os = "windows") {
        tr!(translator, "footer.os_windows")
    } else if cfg!(target_os = "linux") {
        tr!(translator, "footer.os_linux")
    } else if cfg!(target_os = "macos") {
        tr!(translator, "footer.os_macos")
    } else {
        tr!(translator, "footer.os_unknown")
    }
}

pub fn render_footer(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let status_color = if app.status_message.contains("[!]") || app.status_message.contains("[-] ")
    {
        THEME.danger
    } else if app.status_message.contains("[*]") {
        THEME.warning
    } else {
        THEME.success
    };

    let mut status_spans = vec![
        Span::styled(
            format!(" {} ", tr!(app.translator, "footer.status")),
            Style::default()
                .fg(THEME.background)
                .bg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" > {} ", app.status_message),
            Style::default().fg(status_color),
        ),
    ];

    if app.current_nav_view == NavView::Main && app.pending_geo_lookups > 0 {
        status_spans.push(separator());
        status_spans.push(Span::styled(
            format!(
                " \u{f0ac} {} ",
                tr!(app.translator, "status.geo_lookup", app.pending_geo_lookups)
            ),
            Style::default().fg(THEME.warning),
        ));
    }

    let mut right_spans = match app.current_nav_view {
        NavView::Containers => container_footer_spans(app),
        NavView::Storage => storage_footer_spans(app),
        NavView::TrendGraphs => trends_footer_spans(app),
        _ => network_footer_spans(app),
    };
    right_spans.push(separator());
    right_spans.push(Span::styled(
        format!(" {} ", tr!(app.translator, "app.version", APP_VERSION)),
        Style::default().fg(THEME.text_dim),
    ));
    right_spans.push(Span::styled(
        format!(" {} ", tr!(app.translator, "app.os", os_name(&app.translator))),
        Style::default().fg(THEME.secondary),
    ));

    let right_width = right_spans
        .iter()
        .map(|s| s.content.len() as u16)
        .sum::<u16>()
        .min(area.width.saturating_div(2));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary))
        .border_type(BorderType::Rounded);
    let inner = block.inner(area);
    let chunks =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(right_width)]).split(inner);
    f.render_widget(
        Paragraph::new(Line::from(status_spans)).alignment(Alignment::Left),
        chunks[0],
    );
    f.render_widget(
        Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right),
        chunks[1],
    );
    f.render_widget(block, area);
}

fn container_footer_spans(app: &App) -> Vec<Span<'_>> {
    let selected = app
        .get_selected_container()
        .map(|container| container.name.clone())
        .unwrap_or_else(|| tr!(app.translator, "containers.footer_no_container"));
    vec![
        Span::styled(
            format!(
                " {} ",
                tr!(
                    app.translator,
                    "containers.footer_count",
                    app.containers.len()
                )
            ),
            Style::default().fg(THEME.secondary),
        ),
        separator(),
        Span::styled(
            format!(
                " {} ",
                tr!(
                    app.translator,
                    "containers.footer_logs",
                    app.container_logs.len()
                )
            ),
            Style::default().fg(THEME.text_dim),
        ),
        separator(),
        Span::styled(selected, Style::default().fg(THEME.text_dim)),
    ]
}

fn storage_footer_spans(app: &App) -> Vec<Span<'_>> {
    let disk = app.get_selected_disk();
    let mut spans = vec![
        Span::styled(
            format!(" {} ", tr!(app.translator, "storage.col_size").to_lowercase()),
            Style::default().fg(THEME.secondary),
        ),
        separator(),
        Span::styled(
            format!(" {} ", tr!(app.translator, "sidebar.items", app.file_entries.len())),
            Style::default().fg(THEME.text_dim),
        ),
    ];
    if let Some(d) = disk {
        spans.push(separator());
        spans.push(Span::styled(
            format!(" {} {:.0}% ", d.device, d.usage_pct()),
            Style::default().fg(if d.usage_pct() > 85.0 { THEME.danger } else { THEME.success }),
        ));
    }
    spans
}

fn trends_footer_spans(app: &App) -> Vec<Span<'_>> {
    let peak_conn = app.conn_count_history.iter().max().copied().unwrap_or(0);
    let current_cpu = app.cpu_history.last().copied().unwrap_or(0.0);
    vec![
        Span::styled(
            format!(
                " {} ",
                tr!(app.translator, "trends.peak_connections")
            ),
            Style::default().fg(THEME.secondary),
        ),
        separator(),
        Span::styled(
            format!(" {} ", peak_conn),
            Style::default().fg(THEME.text_dim),
        ),
        separator(),
        Span::styled(
            format!(" CPU {:.1}% ", current_cpu),
            Style::default().fg(if current_cpu > 80.0 { THEME.danger } else { THEME.text_dim }),
        ),
    ]
}

fn network_footer_spans(app: &App) -> Vec<Span<'_>> {
    vec![
        Span::styled(
            format!(
                " {} ",
                tr!(app.translator, "app.apps_count", app.app_connections.len())
            ),
            Style::default().fg(THEME.secondary),
        ),
        separator(),
        Span::styled(
            format!(
                " {} ",
                tr!(
                    app.translator,
                    "app.conns_count",
                    app.network_connections.len()
                )
            ),
            Style::default().fg(THEME.text_dim),
        ),
    ]
}

fn separator() -> Span<'static> {
    Span::styled(" | ", Style::default().fg(THEME.secondary))
}
