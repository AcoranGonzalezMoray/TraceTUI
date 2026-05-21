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
    let header_content = if app.current_nav_view == NavView::Containers {
        docker_header_content(app)
    } else {
        network_header_content(app)
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
