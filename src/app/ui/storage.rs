use crate::app::storage::{fmt_size, FileEntry, FILE_EXTENSION_FILTERS};
use crate::app::ui::theme::THEME;
use crate::app::App;
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row,
        Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState, Wrap,
    },
};

pub fn render_storage_view(f: &mut ratatui::Frame, app: &App, area: Rect) {
    if area.height < 10 || area.width < 50 {
        return;
    }
    if app.storage.show_file_viewer {
        render_file_viewer_modal(f, app);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30),
            Constraint::Min(20),
            Constraint::Length(32),
        ])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(9)])
        .split(chunks[0]);

    render_disk_list(f, app, left_chunks[0]);
    render_disk_properties(f, app, left_chunks[1]);
    render_file_browser(f, app, chunks[1]);
    render_storage_actions(f, app, chunks[2]);
}

fn render_disk_list(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.storage.storage_focus == 0;

    let border_color = if is_focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let border_style = if is_focused {
        Style::default().fg(border_color)
    } else {
        Style::default().fg(border_color).dim()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(if is_focused {
            BorderType::Thick
        } else {
            BorderType::Rounded
        })
        .border_style(border_style)
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                tr!(app.ui.translator, "storage.disks"),
                Style::default().fg(THEME.primary).bold(),
            ),
            Span::raw(" "),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.storage.disks.is_empty() {
        let msg = if app.storage.disks_loading {
            tr!(app.ui.translator, "storage.loading")
        } else {
            tr!(app.ui.translator, "storage.no_disks")
        };
        f.render_widget(
            Paragraph::new(msg)
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            inner,
        );
        return;
    }

    let items: Vec<ListItem> = app
        .storage
        .disks
        .iter()
        .enumerate()
        .map(|(i, disk)| {
            let is_selected = i == app.storage.selected_disk_index;
            let pct = disk.usage_pct();

            let bar_len = 8;
            let filled = ((pct / 100.0) * bar_len as f64).round() as usize;
            let bar: String = std::iter::repeat_n('■', filled.min(bar_len))
                .chain(std::iter::repeat_n('▱', bar_len.saturating_sub(filled)))
                .collect();

            let label_line = vec![
                Span::styled(
                    format!(" {}: ", disk.device),
                    Style::default()
                        .fg(if is_selected {
                            THEME.background
                        } else {
                            THEME.text_main
                        })
                        .bold(),
                ),
                Span::styled(
                    format!("{} ", bar),
                    Style::default().fg(if is_selected {
                        THEME.background
                    } else {
                        if pct > 85.0 {
                            THEME.danger
                        } else {
                            THEME.success
                        }
                    }),
                ),
                Span::styled(
                    format!("{:.0}%", pct),
                    Style::default().fg(if is_selected {
                        THEME.background
                    } else {
                        THEME.text_dim
                    }),
                ),
            ];

            let style = if is_selected {
                Style::default().bg(THEME.primary)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(label_line)).style(style)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.storage.selected_disk_index));
    f.render_stateful_widget(List::new(items), inner, &mut state);
}

fn render_disk_properties(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.secondary))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                tr!(app.ui.translator, "storage.properties"),
                Style::default().fg(THEME.accent).bold(),
            ),
            Span::raw(" "),
        ]));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(disk) = app.get_selected_disk() else {
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                tr!(app.ui.translator, "storage.no_disk_selected"),
                Style::default().fg(THEME.text_dim),
            ))),
            inner,
        );
        return;
    };

    let total = crate::app::storage::fmt_size(disk.total_bytes);
    let used = crate::app::storage::fmt_size(disk.used_bytes);
    let free = crate::app::storage::fmt_size(disk.free_bytes);
    let pct = disk.usage_pct();

    let p = |k: &str| -> String { format!("{} ", app.ui.translator.get(k)) };
    let mut lines = vec![
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(p("storage.device"), Style::default().fg(THEME.text_dim)),
            Span::styled(&disk.device, Style::default().fg(THEME.text_main)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(p("storage.mount"), Style::default().fg(THEME.text_dim)),
            Span::styled(&disk.mount_point, Style::default().fg(THEME.text_main)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(p("storage.fs"), Style::default().fg(THEME.text_dim)),
            Span::styled(&disk.fs_type, Style::default().fg(THEME.text_main)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                p("storage.total_label"),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(total, Style::default().fg(THEME.text_main)),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(p("storage.used_label"), Style::default().fg(THEME.text_dim)),
            Span::styled(
                used,
                Style::default().fg(if pct > 85.0 {
                    THEME.danger
                } else {
                    THEME.text_main
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(p("storage.free_label"), Style::default().fg(THEME.text_dim)),
            Span::styled(free, Style::default().fg(THEME.success)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                p("storage.usage_label"),
                Style::default().fg(THEME.text_dim),
            ),
        ]),
    ];

    let bar_w = inner.width.saturating_sub(4).max(4) as usize;
    let filled = ((pct / 100.0) * bar_w as f64).round() as usize;
    let bar: String = std::iter::repeat_n('█', filled.min(bar_w))
        .chain(std::iter::repeat_n('░', bar_w.saturating_sub(filled)))
        .collect();
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            bar,
            Style::default().fg(if pct > 85.0 {
                THEME.danger
            } else {
                THEME.primary
            }),
        ),
        Span::styled(format!(" {:.0}%", pct), Style::default().fg(THEME.text_dim)),
    ]));

    f.render_widget(Paragraph::new(lines), inner);
}

fn file_icon(entry: &FileEntry) -> &'static str {
    if entry.is_dir {
        return "";
    }
    match entry.extension.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" => "󰋩",
        "mp3" | "wav" | "flac" | "ogg" | "aac" => "󰎆",
        "mp4" | "avi" | "mkv" | "mov" | "webm" => "󰕧",
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => "󰿺",
        "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "h" | "java" => "󰨲",
        "txt" | "md" | "log" | "csv" | "json" | "toml" | "yaml" => "󰈙",
        "pdf" => "󰈦",
        "doc" | "docx" => "󰈬",
        "xls" | "xlsx" => "󰈛",
        _ => "󰈔",
    }
}

fn file_color(entry: &FileEntry) -> ratatui::style::Color {
    if entry.is_dir {
        return THEME.primary;
    }
    match entry.extension.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => THEME.success,
        "mp3" | "wav" | "flac" | "mp4" | "mkv" | "mov" => THEME.warning,
        "zip" | "tar" | "gz" | "7z" | "rar" => THEME.accent,
        "exe" | "msi" | "sh" | "bat" => THEME.danger,
        _ => THEME.text_main,
    }
}

fn render_file_browser(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.storage.storage_focus == 1;

    let border_color = if is_focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let border_style = if is_focused {
        Style::default().fg(border_color)
    } else {
        Style::default().fg(border_color).dim()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(if is_focused {
            BorderType::Thick
        } else {
            BorderType::Rounded
        })
        .border_style(border_style)
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                tr!(app.ui.translator, "storage.file_browser"),
                Style::default().fg(THEME.primary).bold(),
            ),
            Span::raw(" "),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let sort_label = app.storage.file_sort_mode.label();
    let sort_str = format!(
        " {}",
        tr!(app.ui.translator, "storage.sort_label", sort_label)
    );
    let path_str = format!(
        " 📁 {} {}",
        app.storage.current_directory.to_string_lossy(),
        sort_str
    );
    let header_path = Paragraph::new(path_str)
        .style(Style::default().fg(THEME.background).bg(THEME.primary))
        .alignment(Alignment::Left);

    if inner.height > 2 {
        f.render_widget(header_path, Rect::new(inner.x, inner.y, inner.width, 1));
    }

    let rest = Rect::new(
        inner.x,
        inner.y + 1,
        inner.width,
        inner.height.saturating_sub(1),
    );

    if app.storage.search_progress_running {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let s = spinner[(app.ui.frame_count as usize) % spinner.len()];
        let mut lines = vec![Line::from("")];

        let files_label = tr!(
            app.ui.translator,
            "storage.files_count",
            app.storage.search_progress_found
        );
        let loading_msg = if app.storage.file_search_recursive {
            format!(
                " {} {} {}",
                s,
                tr!(app.ui.translator, "status.search_recursive_on"),
                files_label
            )
        } else {
            format!(
                " {} {} {}",
                s,
                tr!(app.ui.translator, "storage.loading"),
                files_label
            )
        };

        lines.push(Line::from(Span::styled(
            loading_msg,
            Style::default().fg(THEME.warning).bold(),
        )));
        f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), rest);
        return;
    }

    let total_items = app.network.cached_filtered_indices.len();
    if total_items == 0 {
        f.render_widget(
            Paragraph::new(tr!(app.ui.translator, "storage.empty_dir"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            rest,
        );
        return;
    }

    let current_scroll = app.storage.file_scroll.min(total_items.saturating_sub(1));
    let visible_rows = rest.height.saturating_sub(1).max(1) as usize;
    let scroll_start = current_scroll;
    let scroll_end = (scroll_start + visible_rows).min(total_items);

    let widths = [
        Constraint::Fill(1),
        Constraint::Length(11),
        Constraint::Length(18),
    ];

    let header_row = Row::new(vec![
        Cell::from(Line::from(vec![
            Span::raw("  "),
            Span::raw(tr!(app.ui.translator, "storage.col_name")),
        ])),
        Cell::from(Line::from(vec![Span::raw(tr!(
            app.ui.translator,
            "storage.col_size"
        ))])),
        Cell::from(Line::from(vec![Span::raw(tr!(
            app.ui.translator,
            "storage.col_modified"
        ))])),
    ])
    .style(
        Style::default()
            .fg(THEME.primary)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let rows: Vec<Row> = app.network.cached_filtered_indices[scroll_start..scroll_end]
        .iter()
        .map(|&idx| {
            let entry = &app.storage.file_entries[idx];
            let icon = file_icon(entry);
            let fg_color = file_color(entry);
            let size_str = if entry.is_dir {
                format!("  {}", tr!(app.ui.translator, "storage.dir_label"))
            } else {
                fmt_size(entry.size)
            };
            let file_name_style = if entry.is_dir {
                Style::default().fg(fg_color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(fg_color)
            };
            Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::styled(format!(" {} ", icon), Style::default().fg(fg_color)),
                    Span::styled(&entry.name, file_name_style),
                ])),
                Cell::from(Line::from(Span::styled(
                    size_str,
                    Style::default().fg(fg_color),
                ))),
                Cell::from(Line::from(Span::styled(
                    &entry.modified,
                    Style::default().fg(THEME.text_dim),
                ))),
            ])
        })
        .collect();

    let mut table_state = TableState::default();
    table_state.select(Some(0));

    let table = Table::new(rows, widths)
        .header(header_row)
        .highlight_style(
            Style::default()
                .bg(THEME.primary)
                .dim()
                .fg(THEME.background)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ");

    f.render_stateful_widget(table, rest, &mut table_state);

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"))
        .track_symbol(Some("│"))
        .thumb_symbol("█")
        .style(Style::default().fg(THEME.primary));

    let mut scrollbar_state = ScrollbarState::new(total_items).position(current_scroll);

    f.render_stateful_widget(
        scrollbar,
        rest.inner(ratatui::layout::Margin {
            vertical: 2,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

fn render_storage_actions(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let focused = app.storage.storage_focus == 2;
    let border_color = focus_color_storage(focused);

    let block = styled_block_storage(
        format!(" {} ", tr!(app.ui.translator, "storage.actions")),
        border_color,
        focused,
    );
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let sort_mode_label = app.storage.file_sort_mode.label();
    let items: Vec<(&str, String, &str, ratatui::style::Color)> = vec![
        (
            "\u{f021}",
            tr!(app.ui.translator, "storage.refresh"),
            "R",
            THEME.success,
        ),
        (
            "\u{f15b}",
            tr!(app.ui.translator, "storage.open"),
            "Enter",
            THEME.primary,
        ),
        (
            "\u{f0ca}",
            tr!(app.ui.translator, "storage.properties"),
            "P",
            THEME.warning,
        ),
        (
            "\u{f07c}",
            tr!(app.ui.translator, "storage.parent_dir"),
            "Backspace",
            THEME.accent,
        ),
        (
            "\u{f015}",
            tr!(app.ui.translator, "storage.go_home"),
            "H",
            THEME.danger,
        ),
        (
            "\u{f0dc}",
            tr!(app.ui.translator, "storage.sort_label", sort_mode_label),
            "S",
            THEME.accent,
        ),
    ];

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, (icon, lbl, key, color))| {
            let selected = i == app.storage.selected_storage_action_index;
            let prefix = if selected { " \u{258e}" } else { "  " };
            let name_style = if selected {
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_main)
            };
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(prefix, Style::default().fg(THEME.primary)),
                    Span::styled(format!(" {} ", icon), Style::default().fg(*color)),
                    Span::styled(lbl.clone(), name_style),
                ]),
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(format!("[ {} ]", key), Style::default().fg(THEME.text_dim)),
                ]),
            ])
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.storage.selected_storage_action_index));
    f.render_stateful_widget(List::new(list_items), inner, &mut state);
}

fn focus_color_storage(focused: bool) -> ratatui::style::Color {
    if focused {
        THEME.primary
    } else {
        THEME.secondary
    }
}

fn styled_block_storage<'a>(
    title: String,
    color: ratatui::style::Color,
    focused: bool,
) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(color))
        .border_type(if focused {
            BorderType::Thick
        } else {
            BorderType::Rounded
        })
}

pub fn render_file_viewer_modal(f: &mut ratatui::Frame, app: &App) {
    let area = centered_rect(f.area(), 70, 60);
    f.render_widget(Clear, area);

    let path = app.storage.current_directory.join(
        app.storage
            .file_entries
            .first()
            .map(|e| e.name.as_str())
            .unwrap_or(""),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.primary))
        .title(Line::from(vec![
            Span::raw(" 📄 "),
            Span::styled(
                path.file_name().unwrap_or_default().to_string_lossy(),
                Style::default().fg(THEME.primary).bold(),
            ),
            Span::raw(" "),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.storage.file_viewer_content.is_empty() {
        f.render_widget(
            Paragraph::new(tr!(app.ui.translator, "storage.viewer_empty"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            inner,
        );
        return;
    }

    let total_lines = app.storage.file_viewer_content.len();
    let visible_height = inner.height.saturating_sub(2) as usize;
    let scroll_pos = app
        .storage
        .file_viewer_scroll
        .min(total_lines.saturating_sub(visible_height));

    let mut lines: Vec<Line> = Vec::with_capacity(visible_height + 1);

    lines.push(
        Line::from(vec![
            Span::styled(
                format!(
                    " {} ",
                    tr!(
                        app.ui.translator,
                        "storage.viewer_lines",
                        (scroll_pos + visible_height).min(total_lines),
                        total_lines
                    )
                ),
                Style::default().fg(THEME.background),
            ),
            Span::styled(
                format!(
                    " ({})",
                    tr!(app.ui.translator, "storage.viewer_scroll_hint")
                ),
                Style::default().fg(THEME.background),
            ),
        ])
        .bg(THEME.primary),
    );
    lines.push(Line::from(""));

    if app.storage.file_viewer_is_ansi {
        let raw: Vec<u8> = app.storage.file_viewer_content.join("\n").into_bytes();
        match ansi_to_tui::IntoText::into_text(&raw) {
            Ok(text) => {
                f.render_widget(Paragraph::new(text).alignment(Alignment::Center), inner);
            }
            Err(_) => {
                f.render_widget(
                    Paragraph::new(Line::from(Span::styled(
                        tr!(app.ui.translator, "storage.viewer_empty"),
                        Style::default().fg(THEME.text_dim),
                    )))
                    .alignment(Alignment::Center),
                    inner,
                );
            }
        }
        return;
    }

    for i in scroll_pos..(scroll_pos + visible_height).min(total_lines) {
        if let Some(line) = app.storage.file_viewer_content.get(i) {
            let line_num = i + 1;
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>4} │ ", line_num),
                    Style::default().fg(THEME.secondary),
                ),
                Span::styled(line.clone(), Style::default().fg(THEME.text_main)),
            ]));
        }
    }

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);

    if total_lines > visible_height {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(THEME.primary));

        let mut scrollbar_state = ScrollbarState::new(total_lines).position(scroll_pos);
        f.render_stateful_widget(
            scrollbar,
            inner.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

pub fn render_file_search_modal(f: &mut ratatui::Frame, app: &App) {
    let area = centered_rect(f.area(), 55, 45);
    f.render_widget(Clear, area);

    let state = &app.ui.file_search_state;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.primary))
        .title(Line::from(vec![
            Span::raw(" 🔍 "),
            Span::styled(
                tr!(app.ui.translator, "storage.search_title"),
                Style::default().fg(THEME.warning).bold(),
            ),
            Span::raw(" "),
        ]));
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(inner);

    let q_focused = state.focused_field == 0;
    let q_text = if state.query.is_empty() {
        tr!(app.ui.translator, "storage.type_to_filter")
    } else {
        let cursor = if q_focused && app.ui.frame_count.is_multiple_of(2) {
            "█"
        } else {
            ""
        };
        format!("{}{}", state.query, cursor)
    };

    let border_color_q = if q_focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let border_style_q = if q_focused {
        Style::default().fg(border_color_q)
    } else {
        Style::default().fg(border_color_q).dim()
    };

    let q_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style_q)
        .title(Span::styled(
            tr!(app.ui.translator, "storage.search_query"),
            Style::default().fg(if q_focused {
                THEME.primary
            } else {
                THEME.text_dim
            }),
        ));

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            q_text,
            Style::default().fg(if state.query.is_empty() {
                THEME.text_dim
            } else {
                THEME.text_main
            }),
        )))
        .block(q_block),
        chunks[0],
    );

    let r_focused = state.focused_field == 1;
    let r_icon = if state.recursive { " ✅ " } else { " ❌ " };
    let r_label = if state.recursive {
        tr!(app.ui.translator, "status.search_recursive_on")
    } else {
        tr!(app.ui.translator, "status.search_recursive_off")
    };

    let border_color_r = if r_focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let border_style_r = if r_focused {
        Style::default().fg(border_color_r)
    } else {
        Style::default().fg(border_color_r).dim()
    };

    let r_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style_r)
        .title(Span::styled(
            tr!(app.ui.translator, "storage.search_recursive"),
            Style::default().fg(if r_focused {
                THEME.primary
            } else {
                THEME.text_dim
            }),
        ));

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(r_icon),
            Span::styled(r_label, Style::default().fg(THEME.text_main)),
        ]))
        .block(r_block),
        chunks[1],
    );

    let e_focused = state.focused_field == 2;
    let ext_idx = state
        .extension_idx
        .min(FILE_EXTENSION_FILTERS.len().saturating_sub(1));
    let (ext_icon, _) = FILE_EXTENSION_FILTERS[ext_idx];
    let ext_name = app
        .ui
        .translator
        .get(crate::app::storage::extension_filter_label(ext_idx));

    let border_color_e = if e_focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let border_style_e = if e_focused {
        Style::default().fg(border_color_e)
    } else {
        Style::default().fg(border_color_e).dim()
    };

    let e_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style_e)
        .title(Span::styled(
            tr!(app.ui.translator, "storage.search_filetype"),
            Style::default().fg(if e_focused {
                THEME.primary
            } else {
                THEME.text_dim
            }),
        ));

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!(" {} ", ext_icon), Style::default().fg(THEME.accent)),
            Span::styled(
                ext_name,
                Style::default()
                    .fg(THEME.text_main)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ◀ ▶", Style::default().fg(THEME.text_dim)),
        ]))
        .block(e_block),
        chunks[2],
    );

    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Fill(1)])
        .split(chunks[3]);

    let cont_focused = state.focused_field == 3;
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " ✔ ",
                Style::default().fg(if cont_focused {
                    THEME.background
                } else {
                    THEME.success
                }),
            ),
            Span::styled(
                tr!(app.ui.translator, "dialog.continue"),
                Style::default().fg(if cont_focused {
                    THEME.background
                } else {
                    THEME.success
                }),
            ),
        ]))
        .alignment(Alignment::Center)
        .style(if cont_focused {
            Style::default()
                .bg(THEME.success)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }),
        btn_chunks[0],
    );

    let cancel_focused = state.focused_field == 4;
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " ✘ ",
                Style::default().fg(if cancel_focused {
                    THEME.background
                } else {
                    THEME.danger
                }),
            ),
            Span::styled(
                tr!(app.ui.translator, "dialog.cancel"),
                Style::default().fg(if cancel_focused {
                    THEME.background
                } else {
                    THEME.danger
                }),
            ),
        ]))
        .alignment(Alignment::Center)
        .style(if cancel_focused {
            Style::default()
                .bg(THEME.danger)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }),
        btn_chunks[1],
    );
}

fn centered_rect(r: Rect, pct_x: u16, pct_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
