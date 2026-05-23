use super::theme::THEME;
use crate::app::containers::DockerStatus;
use crate::app::{App, NavView};
use crate::tr;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub fn render_header(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let header_content = match app.current_nav_view {
        NavView::Containers => docker_header_content(app),
        NavView::Storage => storage_header_content(app),
        NavView::TrendGraphs => trends_header_content(app),
        NavView::LibraryInspection => libraries_header_content(app),
        _ => network_header_content(app),
    };
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Rounded);
    let header = Paragraph::new(header_content)
        .block(title_block)
        .alignment(Alignment::Left);
    f.render_widget(header, area);
}

fn network_header_content(app: &App) -> Line<'_> {
    let (dot_text, dot_color, live_text) = if app.analysis_paused {
        ("\u{f004} ", THEME.danger, tr!(app.translator, "app.paused"))
    } else {
        let pulse = app.frame_count % 4 < 2;
        (
            if pulse { "\u{f004} " } else { "  " },
            if pulse { THEME.success } else { THEME.text_dim },
            tr!(app.translator, "app.live"),
        )
    };
    Line::from(vec![
        app_title(app),
        app_subtitle(app),
        separator(),
        Span::styled(dot_text, Style::default().fg(dot_color)),
        Span::styled(
            format!("{} ", live_text),
            Style::default().fg(THEME.text_main),
        ),
        separator(),
        Span::styled(
            if app.hunter_mode {
                format!("\u{f493} {}", tr!(app.translator, "app.mode_hunter"))
            } else {
                format!("\u{f493} {}", tr!(app.translator, "app.mode_normal"))
            },
            Style::default().fg(if app.hunter_mode {
                THEME.success
            } else {
                THEME.text_dim
            }),
        ),
        separator(),
        Span::styled(
            format!(
                " \u{f0c0}  {} ",
                tr!(app.translator, "app.apps_count", app.app_connections.len())
            ),
            Style::default().fg(THEME.secondary),
        ),
        Span::styled(
            format!(
                " \u{f0c1}  {} ",
                tr!(
                    app.translator,
                    "app.conns_count",
                    app.network_connections.len()
                )
            ),
            Style::default().fg(THEME.secondary),
        ),
    ])
}

fn docker_header_content(app: &App) -> Line<'_> {
    let (label, color) = match app.docker_status() {
        DockerStatus::On => (tr!(app.translator, "containers.header_on"), THEME.success),
        DockerStatus::Starting => (
            tr!(app.translator, "containers.header_starting"),
            THEME.warning,
        ),
        DockerStatus::Off | DockerStatus::Missing => {
            (tr!(app.translator, "containers.header_off"), THEME.danger)
        }
        DockerStatus::Unknown => (
            tr!(app.translator, "containers.header_unknown"),
            THEME.warning,
        ),
    };

    Line::from(vec![
        app_title(app),
        app_subtitle(app),
        separator(),
        Span::styled("\u{f011} ", Style::default().fg(color)),
        Span::styled(
            format!("{} ", label),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        separator(),
        Span::styled(
            format!(
                "\u{f308} {} ",
                tr!(
                    app.translator,
                    "containers.header_count",
                    app.containers.len()
                )
            ),
            Style::default().fg(THEME.secondary),
        ),
        Span::styled(
            app.get_selected_container()
                .map(|container| format!("{} ", container.name))
                .unwrap_or_else(|| tr!(app.translator, "containers.header_no_selection")),
            Style::default().fg(THEME.text_dim),
        ),
    ])
}

fn storage_header_content(app: &App) -> Line<'_> {
    let disk = app.get_selected_disk();
    let disk_info = disk
        .map(|d| format!("{} {:.0}%", d.device, d.usage_pct()))
        .unwrap_or_default();
    let path = app.current_directory.to_string_lossy().to_string();
    Line::from(vec![
        app_title(app),
        app_subtitle(app),
        separator(),
        Span::styled(" \u{f0a0} ", Style::default().fg(THEME.primary)),
        Span::styled(disk_info, Style::default().fg(THEME.text_main)),
        separator(),
        Span::styled(
            format!(
                " \u{f15b} {} ",
                if path.len() > 40 {
                    format!("...{}", &path[path.len() - 37..])
                } else {
                    path
                }
            ),
            Style::default().fg(THEME.text_dim),
        ),
        Span::styled(
            format!(
                "{} {}",
                tr!(app.translator, "storage.col_size").to_lowercase(),
                app.file_entries.len()
            ),
            Style::default().fg(THEME.secondary),
        ),
    ])
}

fn trends_header_content(app: &App) -> Line<'_> {
    let active_conns: u64 = app
        .app_connections
        .iter()
        .map(|a| a.connections.len() as u64)
        .sum();
    let total_cpu: f64 = app.app_connections.iter().map(|a| a.cpu_usage as f64).sum();
    let total_mem_mb: u64 = app
        .app_connections
        .iter()
        .map(|a| a.memory_usage / 1024 / 1024)
        .sum();
    let high_risk = app
        .app_connections
        .iter()
        .filter(|a| a.risk_level.contains("HIGH") || a.risk_level.contains("CRITICAL"))
        .count();
    Line::from(vec![
        app_title(app),
        app_subtitle(app),
        separator(),
        Span::styled(
            format!(
                " \u{f0c1} {} ",
                tr!(app.translator, "app.conns_count", active_conns)
            ),
            Style::default().fg(THEME.primary),
        ),
        separator(),
        Span::styled(
            format!(" CPU {:.1}% ", total_cpu),
            Style::default().fg(if total_cpu > 80.0 {
                THEME.danger
            } else {
                THEME.text_dim
            }),
        ),
        Span::styled(
            format!(" MEM {} MB ", total_mem_mb),
            Style::default().fg(THEME.text_dim),
        ),
        separator(),
        Span::styled(
            format!(" \u{26a0} {} ", high_risk),
            Style::default().fg(if high_risk > 0 {
                THEME.danger
            } else {
                THEME.success
            }),
        ),
    ])
}

fn libraries_header_content(app: &App) -> Line<'_> {
    let total = app.libraries.len();
    let suspicious = app
        .libraries
        .iter()
        .filter(|l| l.risk == "Suspicious")
        .count();
    let process_count = {
        let mut pids = std::collections::HashSet::new();
        for l in &app.libraries {
            pids.insert(l.pid);
        }
        pids.len()
    };
    Line::from(vec![
        app_title(app),
        app_subtitle(app),
        separator(),
        Span::styled(
            format!(
                " \u{f0e7} {} ",
                tr!(app.translator, "libraries.header_count", total)
            ),
            Style::default().fg(THEME.primary),
        ),
        separator(),
        Span::styled(
            format!(
                " \u{f493} {} ",
                tr!(app.translator, "libraries.header_procs", process_count)
            ),
            Style::default().fg(THEME.secondary),
        ),
        Span::styled(
            format!(" \u{26a0} {} ", suspicious),
            Style::default().fg(if suspicious > 0 {
                THEME.danger
            } else {
                THEME.text_dim
            }),
        ),
    ])
}

fn app_title(app: &App) -> Span<'_> {
    Span::styled(
        format!(" {} ", tr!(app.translator, "app.title")),
        Style::default()
            .fg(THEME.background)
            .bg(THEME.primary)
            .add_modifier(Modifier::BOLD),
    )
}

fn app_subtitle(app: &App) -> Span<'_> {
    Span::styled(
        format!(" {} ", tr!(app.translator, "app.subtitle")),
        Style::default()
            .fg(THEME.primary)
            .bg(THEME.background)
            .add_modifier(Modifier::BOLD),
    )
}

fn separator() -> Span<'static> {
    Span::styled(" | ", Style::default().fg(THEME.secondary))
}
