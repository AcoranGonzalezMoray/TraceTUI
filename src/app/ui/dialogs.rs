use super::theme::THEME;
use crate::app::App;
use crate::config;
use crate::i18n::Translator;
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph},
};
pub fn render_install_dialog(f: &mut ratatui::Frame, app: &App) {
    let popup_area = Rect {
        x: (f.area().width / 5),
        y: (f.area().height / 3),
        width: f.area().width * 3 / 5,
        height: 12,
    };
    if app.install.installing && !app.install.done {
        let spinner = match app.ui.frame_count % 4 {
            0 => "/",
            1 => "-",
            2 => "\\",
            _ => "|",
        };
        let dialog_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!(
                    "  {}  {} ",
                    spinner,
                    tr!(app.ui.translator, "dialog.net_tools_installing")
                ),
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {} ", app.install.message),
                Style::default().fg(THEME.text_main),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("   Press ", Style::default().fg(THEME.text_dim)),
                Span::styled(
                    " Esc ",
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.danger)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}", tr!(app.ui.translator, "dialog.to_cancel")),
                    Style::default().fg(THEME.text_main),
                ),
            ]),
        ];
        let dialog = Paragraph::new(dialog_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", tr!(app.ui.translator, "dialog.net_tools")))
                    .title_style(
                        Style::default()
                            .fg(THEME.warning)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(THEME.warning))
                    .border_type(BorderType::Thick),
            )
            .style(Style::default().bg(THEME.background))
            .alignment(Alignment::Left);
        f.render_widget(Clear, popup_area);
        f.render_widget(dialog, popup_area);
    } else if app.install.done {
        let (icon, border_color, title) = if app.install.success {
            (
                "[OK]",
                THEME.success,
                format!(" {} ", tr!(app.ui.translator, "dialog.net_tools_complete")),
            )
        } else {
            (
                "[FAIL]",
                THEME.danger,
                format!(" {} ", tr!(app.ui.translator, "dialog.net_tools_failed")),
            )
        };
        let lines: Vec<Line> = app
            .install.message
            .lines()
            .map(|l| {
                Line::from(Span::styled(
                    format!("  {} ", l),
                    Style::default().fg(THEME.text_main),
                ))
            })
            .collect();
        let mut dialog_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {} ", icon),
                Style::default()
                    .fg(THEME.background)
                    .bg(border_color)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];
        dialog_text.extend(lines);
        dialog_text.push(Line::from(""));
        dialog_text.push(Line::from(vec![
            Span::styled("   Press ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                " Enter ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {} ", tr!(app.ui.translator, "dialog.or")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " Esc ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}", tr!(app.ui.translator, "dialog.to_dismiss")),
                Style::default().fg(THEME.text_main),
            ),
        ]));
        let dialog = Paragraph::new(dialog_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .title_style(
                        Style::default()
                            .fg(border_color)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(border_color))
                    .border_type(BorderType::Thick),
            )
            .style(Style::default().bg(THEME.background))
            .alignment(Alignment::Left);
        f.render_widget(Clear, popup_area);
        f.render_widget(dialog, popup_area);
    } else {
        let dialog_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {} ", tr!(app.ui.translator, "dialog.net_tools_title")),
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                &app.install.message,
                Style::default().fg(THEME.text_main),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("   Press ", Style::default().fg(THEME.text_dim)),
                Span::styled(
                    " Enter ",
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {} ", tr!(app.ui.translator, "dialog.or")),
                    Style::default().fg(THEME.text_dim),
                ),
                Span::styled(
                    " Y ",
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}   ", tr!(app.ui.translator, "dialog.install")),
                    Style::default().fg(THEME.text_main),
                ),
                Span::styled(
                    " N ",
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.danger)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}", tr!(app.ui.translator, "dialog.cancel")),
                    Style::default().fg(THEME.text_main),
                ),
            ]),
        ];
        let dialog = Paragraph::new(dialog_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", tr!(app.ui.translator, "dialog.net_tools")))
                    .title_style(
                        Style::default()
                            .fg(THEME.warning)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(THEME.warning))
                    .border_type(BorderType::Thick),
            )
            .style(Style::default().bg(THEME.background))
            .alignment(Alignment::Left);
        f.render_widget(Clear, popup_area);
        f.render_widget(dialog, popup_area);
    }
}
pub fn render_password_modal(f: &mut ratatui::Frame, app: &App) {
    let popup_area = Rect {
        x: (f.area().width / 4),
        y: (f.area().height / 3),
        width: f.area().width / 2,
        height: 10,
    };
    let masked: String = app.install.password.chars().map(|_| '*').collect();
    let cursor = if app.ui.frame_count.is_multiple_of(2) {
        "█"
    } else {
        " "
    };
    let dialog_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {} ", tr!(app.ui.translator, "dialog.password_required")),
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {} ", tr!(app.ui.translator, "dialog.password_label")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(masked, Style::default().fg(THEME.text_main)),
            Span::styled(cursor, Style::default().fg(THEME.primary)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("   Press ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                " Enter ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}   ", tr!(app.ui.translator, "dialog.confirm")),
                Style::default().fg(THEME.text_main),
            ),
            Span::styled(
                " Esc ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}", tr!(app.ui.translator, "dialog.cancel")),
                Style::default().fg(THEME.text_main),
            ),
        ]),
    ];
    let dialog = Paragraph::new(dialog_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " 󰰍 {} ",
                    tr!(app.ui.translator, "dialog.password_title")
                ))
                .title_style(
                    Style::default()
                        .fg(THEME.warning)
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(THEME.warning))
                .border_type(BorderType::Thick),
        )
        .style(Style::default().bg(THEME.background))
        .alignment(Alignment::Left);
    f.render_widget(Clear, popup_area);
    f.render_widget(dialog, popup_area);
}
pub fn render_nerdfont_dialog(f: &mut ratatui::Frame, app: &App) {
    let popup_area = Rect {
        x: (f.area().width / 6),
        y: (f.area().height / 4),
        width: f.area().width * 2 / 3,
        height: 14,
    };
    if app.nerdfont.installing && !app.nerdfont.install_done {
        let spinner = match app.ui.frame_count % 4 {
            0 => "/",
            1 => "-",
            2 => "\\",
            _ => "|",
        };
        let dialog_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!(
                    "  {}  {} ",
                    spinner,
                    tr!(app.ui.translator, "dialog.nerdfont_installing")
                ),
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {} ", app.nerdfont.install_message),
                Style::default().fg(THEME.text_main),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {}", tr!(app.ui.translator, "dialog.nerdfont_wait1")),
                Style::default().fg(THEME.text_dim),
            )]),
            Line::from(vec![Span::styled(
                format!("  {}", tr!(app.ui.translator, "dialog.nerdfont_wait2")),
                Style::default().fg(THEME.text_dim),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("   Press ", Style::default().fg(THEME.text_dim)),
                Span::styled(
                    " Esc ",
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.danger)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}", tr!(app.ui.translator, "dialog.to_cancel")),
                    Style::default().fg(THEME.text_main),
                ),
            ]),
        ];
        let dialog = Paragraph::new(dialog_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        " {} ",
                        tr!(app.ui.translator, "dialog.nerdfont_complete")
                    ))
                    .title_style(
                        Style::default()
                            .fg(THEME.warning)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(THEME.warning))
                    .border_type(BorderType::Thick),
            )
            .style(Style::default().bg(THEME.background))
            .alignment(Alignment::Left);
        f.render_widget(Clear, popup_area);
        f.render_widget(dialog, popup_area);
    } else if app.nerdfont.install_done {
        let (icon, border_color) = if app.nerdfont.install_success {
            ("[OK]", THEME.success)
        } else {
            ("[FAIL]", THEME.danger)
        };
        let lines: Vec<Line> = app
            .nerdfont.install_message
            .lines()
            .map(|l| {
                Line::from(Span::styled(
                    format!("  {} ", l),
                    Style::default().fg(THEME.text_main),
                ))
            })
            .collect();
        let mut dialog_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {} ", icon),
                Style::default()
                    .fg(THEME.background)
                    .bg(border_color)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
        ];
        dialog_text.extend(lines);
        dialog_text.push(Line::from(""));
        dialog_text.push(Line::from(vec![
            Span::styled("   Press ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                " Enter ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {} ", tr!(app.ui.translator, "dialog.or")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " Esc ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}", tr!(app.ui.translator, "dialog.to_dismiss")),
                Style::default().fg(THEME.text_main),
            ),
        ]));
        let dialog = Paragraph::new(dialog_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        " {} ",
                        tr!(app.ui.translator, "dialog.nerdfont_complete")
                    ))
                    .title_style(
                        Style::default()
                            .fg(border_color)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(border_color))
                    .border_type(BorderType::Thick),
            )
            .style(Style::default().bg(THEME.background))
            .alignment(Alignment::Left);
        f.render_widget(Clear, popup_area);
        f.render_widget(dialog, popup_area);
    } else {
        let dialog_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {} ", tr!(app.ui.translator, "dialog.nerdfont_required")),
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {}", tr!(app.ui.translator, "dialog.nerdfont_intro1")),
                Style::default().fg(THEME.text_main),
            )]),
            Line::from(vec![Span::styled(
                format!("  {}", tr!(app.ui.translator, "dialog.nerdfont_intro2")),
                Style::default().fg(THEME.text_main),
            )]),
            Line::from(vec![Span::styled(
                format!("  {}", tr!(app.ui.translator, "dialog.nerdfont_intro3")),
                Style::default().fg(THEME.text_main),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {}", tr!(app.ui.translator, "dialog.nerdfont_prompt1")),
                Style::default().fg(THEME.text_dim),
            )]),
            Line::from(vec![Span::styled(
                format!("  {}", tr!(app.ui.translator, "dialog.nerdfont_prompt2")),
                Style::default().fg(THEME.text_dim),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("   Press ", Style::default().fg(THEME.text_dim)),
                Span::styled(
                    " Enter ",
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}   ", tr!(app.ui.translator, "dialog.install")),
                    Style::default().fg(THEME.text_main),
                ),
                Span::styled(
                    " N ",
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.danger)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}", tr!(app.ui.translator, "dialog.nerdfont_skip")),
                    Style::default().fg(THEME.text_main),
                ),
            ]),
        ];
        let dialog = Paragraph::new(dialog_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        " {} ",
                        tr!(app.ui.translator, "dialog.nerdfont_title")
                    ))
                    .title_style(
                        Style::default()
                            .fg(THEME.warning)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(THEME.warning))
                    .border_type(BorderType::Thick),
            )
            .style(Style::default().bg(THEME.background))
            .alignment(Alignment::Left);
        f.render_widget(Clear, popup_area);
        f.render_widget(dialog, popup_area);
    }
}
pub fn render_language_modal(f: &mut ratatui::Frame, app: &App) {
    let locales = Translator::available_locales();
    let total = locales.len();
    let visible = config::LANGUAGE_VISIBLE_ITEMS;
    let list_rows = total.min(visible) as u16;
    let line_count = 3 + list_rows;
    let popup_height = line_count + 2;
    let popup_width = f.area().width * 2 / 5;
    let popup_area = Rect {
        x: (f.area().width - popup_width) / 2,
        y: (f.area().height - popup_height) / 2,
        width: popup_width,
        height: popup_height,
    };

    let locale_key = format!("locale.{}", app.ui.translator.locale);
    let locale_name = app.ui.translator.get(&locale_key).to_string();

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!(" {} ", locale_name),
            Style::default()
                .fg(THEME.background)
                .bg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            tr!(app.ui.translator, "language.prompt"),
            Style::default().fg(THEME.text_dim),
        ),
    ]));

    let offset = app.ui.language_scroll_offset;
    let scrollbar_chars = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    for rel_i in 0..visible {
        let abs_i = offset + rel_i;
        if abs_i >= total {
            break;
        }
        let (_code, name) = &locales[abs_i];
        let is_selected = abs_i == app.ui.language_selection_index;
        let item = if is_selected {
            Line::from(vec![
                Span::styled(" ▎", Style::default().fg(THEME.primary)),
                Span::styled(
                    format!(" {} ", name),
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.primary)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        } else {
            Line::from(vec![
                Span::raw("   "),
                Span::styled(format!(" {} ", name), Style::default().fg(THEME.text_main)),
            ])
        };
        lines.push(item);
    }

    if total > visible {
        let thumb_pos = (app.ui.language_selection_index * (scrollbar_chars.len() - 1)) / total.max(1);
        let thumb = scrollbar_chars[thumb_pos.min(scrollbar_chars.len() - 1)];
        let pad_w = popup_width.saturating_sub(6);
        let pad = " ".repeat(pad_w as usize);
        if let Some(last_line) = lines.last_mut() {
            last_line.spans.push(Span::styled(pad, Style::default()));
            last_line.spans.push(Span::styled(
                format!(" {} ", thumb),
                Style::default().fg(THEME.text_dim),
            ));
        }
    }

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(tr!(app.ui.translator, "language.title"))
        .title_style(
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(THEME.warning))
        .border_type(BorderType::Thick);
    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph.block(block), popup_area);
}

pub fn render_update_dialog(f: &mut ratatui::Frame, app: &App) {
    let popup_height = 10;
    let popup_area = Rect {
        x: (f.area().width / 6),
        y: (f.area().height.saturating_sub(popup_height)) / 2,
        width: f.area().width * 2 / 3,
        height: popup_height.min(f.area().height),
    };

    if app.update.is_updating && !app.update.update_done {
        let spinner = match app.ui.frame_count % 4 {
            0 => "/",
            1 => "-",
            2 => "\\",
            _ => "|",
        };

        let progress_chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(popup_area);

        let dialog_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!(
                    "  {}  {} ",
                    spinner,
                    tr!(app.ui.translator, "dialog.update_downloading")
                ),
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {} ", tr!(app.ui.translator, "dialog.nerdfont_wait1")),
                Style::default().fg(THEME.text_dim),
            )]),
        ];

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(THEME.primary)),
            )
            .gauge_style(
                Style::default()
                    .fg(THEME.success)
                    .bg(THEME.background)
                    .add_modifier(Modifier::BOLD),
            )
            .percent(app.update.update_progress as u16)
            .label(format!("{:.1}%", app.update.update_progress));

        let dialog = Paragraph::new(dialog_text)
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                    .title(format!(" {} ", tr!(app.ui.translator, "dialog.update_title")))
                    .title_style(
                        Style::default()
                            .fg(THEME.warning)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(THEME.warning))
                    .border_type(BorderType::Thick),
            )
            .style(Style::default().bg(THEME.background));

        f.render_widget(Clear, popup_area);
        f.render_widget(dialog, progress_chunks[0]);

        let gauge_area = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Percentage(5),
                Constraint::Percentage(90),
                Constraint::Percentage(5),
            ])
            .split(progress_chunks[1])[1];

        f.render_widget(gauge, gauge_area);

        let footer_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("   Press ", Style::default().fg(THEME.text_dim)),
                Span::styled(
                    " Esc ",
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.danger)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}", tr!(app.ui.translator, "dialog.to_cancel")),
                    Style::default().fg(THEME.text_main),
                ),
            ]),
        ];
        let footer = Paragraph::new(footer_text).block(
            Block::default()
                .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(THEME.warning))
                .border_type(BorderType::Thick),
        );
        f.render_widget(footer, progress_chunks[2]);
    } else if app.update.update_done {
        let (icon, border_color, title) = if app.update.update_success {
            (
                "\u{2705}",
                THEME.success,
                tr!(app.ui.translator, "dialog.update_success"),
            )
        } else {
            (
                "\u{274C}",
                THEME.danger,
                tr!(app.ui.translator, "dialog.update_failed"),
            )
        };
        let dialog_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {} ", icon),
                Style::default()
                    .fg(THEME.background)
                    .bg(border_color)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {} ", app.update.update_message),
                Style::default().fg(THEME.text_main),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("   Press ", Style::default().fg(THEME.text_dim)),
                Span::styled(
                    " Enter ",
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}", tr!(app.ui.translator, "dialog.to_dismiss")),
                    Style::default().fg(THEME.text_main),
                ),
            ]),
        ];
        let dialog = Paragraph::new(dialog_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", title))
                    .title_style(
                        Style::default()
                            .fg(border_color)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(border_color))
                    .border_type(BorderType::Thick),
            )
            .style(Style::default().bg(THEME.background))
            .alignment(Alignment::Left);
        f.render_widget(Clear, popup_area);
        f.render_widget(dialog, popup_area);
    } else {
        let dialog_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("  {} ", tr!(app.ui.translator, "dialog.update_available")),
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!(
                        "v{} → v{}",
                        env!("CARGO_PKG_VERSION"),
                        app.update.latest_remote_version,
                    ),
                    Style::default().fg(THEME.text_main),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    tr!(app.ui.translator, "dialog.update_prompt"),
                    Style::default().fg(THEME.text_dim),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("   Press ", Style::default().fg(THEME.text_dim)),
                Span::styled(
                    " Enter ",
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}   ", tr!(app.ui.translator, "dialog.download")),
                    Style::default().fg(THEME.text_main),
                ),
                Span::styled(
                    " Esc ",
                    Style::default()
                        .fg(THEME.background)
                        .bg(THEME.danger)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}", tr!(app.ui.translator, "dialog.to_dismiss")),
                    Style::default().fg(THEME.text_main),
                ),
            ]),
        ];
        let dialog = Paragraph::new(dialog_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} ", tr!(app.ui.translator, "dialog.update_title")))
                    .title_style(
                        Style::default()
                            .fg(THEME.warning)
                            .add_modifier(Modifier::BOLD),
                    )
                    .border_style(Style::default().fg(THEME.warning))
                    .border_type(BorderType::Thick),
            )
            .style(Style::default().bg(THEME.background))
            .alignment(Alignment::Left);
        f.render_widget(Clear, popup_area);
        f.render_widget(dialog, popup_area);
    }
}

pub fn render_confirmation_dialog(f: &mut ratatui::Frame, app: &App) {
    let popup_area = Rect {
        x: (f.area().width / 4),
        y: (f.area().height / 3),
        width: f.area().width / 2,
        height: 11,
    };

    let dialog_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  󰆐  {} ", tr!(app.ui.translator, "dialog.confirm_attention")),
            Style::default()
                .fg(THEME.danger)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {} ", app.ui.confirmation_message),
            Style::default().fg(THEME.text_main),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {} ", tr!(app.ui.translator, "dialog.admin_required")),
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::ITALIC),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("   Press ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                " Y ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}   ", tr!(app.ui.translator, "dialog.confirm")),
                Style::default().fg(THEME.text_main),
            ),
            Span::styled(
                " N ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}", tr!(app.ui.translator, "dialog.cancel")),
                Style::default().fg(THEME.text_main),
            ),
        ]),
    ];

    let dialog = Paragraph::new(dialog_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " 󰰍 {} ",
                    tr!(app.ui.translator, "dialog.confirm_title")
                ))
                .title_style(
                    Style::default()
                        .fg(THEME.danger)
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(THEME.danger))
                .border_type(BorderType::Thick),
        )
        .style(Style::default().bg(THEME.background))
        .alignment(ratatui::layout::Alignment::Left);
    f.render_widget(Clear, popup_area);
    f.render_widget(dialog, popup_area);
}

pub fn render_welcome_dialog(f: &mut ratatui::Frame, app: &App) {
    let popup_height = 10;
    let popup_width = 70;
    let area = Rect {
        x: (f.area().width.saturating_sub(popup_width)) / 2,
        y: (f.area().height.saturating_sub(popup_height)) / 2,
        width: popup_width.min(f.area().width),
        height: popup_height.min(f.area().height),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(THEME.success))
        .title(format!(
            " 🎉 {} 🎉 ",
            tr!(app.ui.translator, "dialog.welcome_title")
        ))
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(THEME.background));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .margin(1)
        .split(area);

    let current_version = env!("CARGO_PKG_VERSION");
    let content = vec![Line::from(vec![
        Span::styled(
            format!("{} ", tr!(app.ui.translator, "dialog.welcome_message")),
            Style::default().fg(THEME.text_main),
        ),
        Span::styled(
            format!("v{}", current_version),
            Style::default()
                .fg(THEME.success)
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .alignment(Alignment::Center)];

    let content_para = Paragraph::new(content)
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });

    let button_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(35),
            Constraint::Percentage(10),
            Constraint::Percentage(35),
            Constraint::Percentage(10),
        ])
        .split(inner_chunks[1]);

    let btn_continue_style = if app.ui.welcome_index == crate::config::WELCOME_PAGE_COUNT - 2 {
        Style::default()
            .fg(THEME.background)
            .bg(THEME.success)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.text_dim).bg(THEME.background)
    };

    let btn_changes_style = if app.ui.welcome_index == crate::config::WELCOME_PAGE_COUNT - 1 {
        Style::default()
            .fg(THEME.background)
            .bg(THEME.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(THEME.text_dim).bg(THEME.background)
    };

    let btn_continue = Paragraph::new(tr!(app.ui.translator, "dialog.confirm"))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if app.ui.welcome_index == crate::config::WELCOME_PAGE_COUNT - 2 {
                    THEME.success
                } else {
                    THEME.text_dim
                })),
        );

    let btn_changes = Paragraph::new(tr!(app.ui.translator, "dialog.view_changes"))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if app.ui.welcome_index == crate::config::WELCOME_PAGE_COUNT - 1 {
                    THEME.primary
                } else {
                    THEME.text_dim
                })),
        );

    f.render_widget(Clear, area);
    f.render_widget(block, area);
    f.render_widget(content_para, inner_chunks[0]);

    if app.ui.welcome_index == crate::config::WELCOME_PAGE_COUNT - 2 {
        f.render_widget(Block::default().style(btn_continue_style), button_area[1]);
    } else {
        f.render_widget(Block::default().style(btn_changes_style), button_area[3]);
    }

    f.render_widget(btn_continue, button_area[1]);
    f.render_widget(btn_changes, button_area[3]);
}
