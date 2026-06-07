use crate::app::ui::agents::constants::{
    ACTION_BTN_PADDING, RUNNING_PHASE_COUNT, RUNNING_PHASE_INTERVAL_FRAMES, SPINNER_FRAMES,
    STATUS_BADGE_PADDING,
};
use crate::app::ui::theme::THEME;
use crate::i18n::Translator;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

pub fn spinner_for_frame(frame_count: u64) -> &'static str {
    let frames = SPINNER_FRAMES;
    frames[(frame_count as usize) % frames.len()]
}

pub fn action_button(
    icon: &str,
    label: String,
    key: &str,
    color: Color,
) -> Vec<Span<'static>> {
    vec![
        Span::raw(ACTION_BTN_PADDING),
        Span::styled(
            format!(" {} {} ", icon, label),
            Style::default()
                .fg(THEME.background)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {} ", key), Style::default().fg(THEME.text_dim)),
    ]
}

pub fn status_badge(text: String, color: Color) -> Span<'static> {
    Span::styled(
        format!("{}{}{}", STATUS_BADGE_PADDING, text, STATUS_BADGE_PADDING),
        Style::default()
            .fg(THEME.background)
            .bg(color)
            .add_modifier(Modifier::BOLD),
    )
}

pub fn phase_for_frame(frame_count: u64, started_at: u64, translator: &Translator) -> String {
    let elapsed = frame_count.saturating_sub(started_at);
    let phase = (elapsed / RUNNING_PHASE_INTERVAL_FRAMES) % RUNNING_PHASE_COUNT;
    let key = match phase {
        0 => "agents.phase_analyzing",
        1 => "agents.phase_writing",
        2 => "agents.phase_deepening",
        _ => "agents.phase_improving",
    };
    translator.get(key).to_string()
}
