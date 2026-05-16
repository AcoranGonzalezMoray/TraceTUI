use super::theme::THEME;
use crate::app::App;
use crate::tr;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};
pub fn render_header(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let (dot_text, dot_color, live_text) = if app.analysis_paused {
        ("󱐱 ", THEME.danger, tr!(app.translator, "app.paused"))
    } else {
        let pulse = app.frame_count % 4 < 2;
        (
            if pulse { "󱐱 " } else { "  " },
            if pulse { THEME.success } else { THEME.text_dim },
            tr!(app.translator, "app.live"),
        )
    };
    let header_content = Line::from(vec![
        Span::styled(
            format!(" {} ", tr!(app.translator, "app.title")),
            Style::default()
                .fg(THEME.background)
                .bg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {} ", tr!(app.translator, "app.subtitle")),
            Style::default()
                .fg(THEME.primary)
                .bg(THEME.background)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(THEME.secondary)),
        Span::styled(dot_text, Style::default().fg(dot_color)),
        Span::styled(
            format!("{} ", live_text),
            Style::default().fg(THEME.text_main),
        ),
        Span::styled(" │ ", Style::default().fg(THEME.secondary)),
        Span::styled(
            if app.hunter_mode {
                format!("󰒓 {}", tr!(app.translator, "app.mode_hunter"))
            } else {
                format!("󰒓 {}", tr!(app.translator, "app.mode_normal"))
            },
            Style::default().fg(if app.hunter_mode {
                THEME.success
            } else {
                THEME.text_dim
            }),
        ),
        Span::styled("│ ", Style::default().fg(THEME.secondary)),
        Span::styled(
            format!(
                "   {} ",
                tr!(app.translator, "app.apps_count", app.app_connections.len())
            ),
            Style::default().fg(THEME.secondary),
        ),
        Span::styled(
            format!(
                " 󱂇  {} ",
                tr!(
                    app.translator,
                    "app.conns_count",
                    app.network_connections.len()
                )
            ),
            Style::default().fg(THEME.secondary),
        ),
    ]);
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.primary))
        .border_type(BorderType::Rounded);
    let header = Paragraph::new(header_content)
        .block(title_block)
        .alignment(Alignment::Left);
    f.render_widget(header, area);
}
