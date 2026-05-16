use super::theme::THEME;
use crate::app::App;
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
fn os_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unknown"
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
            format!(" ⟫ {} ", app.status_message),
            Style::default().fg(status_color),
        ),
    ];
    if app.pending_geo_lookups > 0 {
        status_spans.push(Span::styled(" │ ", Style::default().fg(THEME.secondary)));
        status_spans.push(Span::styled(
            format!(
                " 󰩠 {} ",
                tr!(app.translator, "status.geo_lookup", app.pending_geo_lookups)
            ),
            Style::default().fg(THEME.warning),
        ));
    }
    let right_spans = vec![
        Span::styled(" │ ", Style::default().fg(THEME.secondary)),
        Span::styled(
            format!(" {} ", tr!(app.translator, "app.version", APP_VERSION)),
            Style::default().fg(THEME.text_dim),
        ),
        Span::styled(
            format!(" {} ", tr!(app.translator, "app.os", os_name())),
            Style::default().fg(THEME.secondary),
        ),
    ];
    let right_width = right_spans
        .iter()
        .map(|s| s.content.len() as u16)
        .sum::<u16>();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(THEME.secondary))
        .border_type(BorderType::Rounded);
    let inner = block.inner(area);
    let chunks =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(right_width)]).split(inner);
    let left_line = Line::from(status_spans);
    let left_para = Paragraph::new(left_line).alignment(Alignment::Left);
    f.render_widget(left_para, chunks[0]);
    let right_line = Line::from(right_spans);
    let right_para = Paragraph::new(right_line).alignment(Alignment::Right);
    f.render_widget(right_para, chunks[1]);
    f.render_widget(block, area);
}
