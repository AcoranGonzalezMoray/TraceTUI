use super::theme::THEME;
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
};
pub fn render_scrollbar(
    f: &mut ratatui::Frame,
    area: Rect,
    content_length: usize,
    position: usize,
) {
    let mut scrollbar_state = ScrollbarState::default()
        .content_length(content_length)
        .position(position);
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .style(Style::default().fg(THEME.text_dim));
    f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
}
