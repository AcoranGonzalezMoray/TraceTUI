use super::theme::THEME;
use super::widgets;
use crate::app::libraries::{LibraryInfo, LibraryOrigin, SignatureStatus};
use crate::app::{App, NavView, SidebarFocus};
use crate::config;
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table,
    },
};

pub fn render_libraries_view(f: &mut ratatui::Frame, app: &App, area: Rect) {
    if app.libraries.libraries_loading && app.libraries.libraries.is_empty() {
        render_loading(f, app, area);
        return;
    }

    let search_bar_height = if app.libraries.library_search_active {
        3
    } else {
        0
    };

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(search_bar_height),
            Constraint::Min(0),
            Constraint::Length(7),
        ])
        .split(area);

    if app.libraries.library_search_active {
        render_library_search_bar(f, app, rows[0]);
    }

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 5),
            Constraint::Ratio(3, 5),
            Constraint::Ratio(1, 5),
        ])
        .split(rows[1]);

    render_process_list(f, app, cols[0]);
    render_library_table(f, app, cols[1]);
    render_library_actions_panel(f, app, cols[2]);
    render_selected_library_info(f, app, rows[2]);
}

fn render_loading(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let s = spinners[(app.ui.frame_count as usize) % spinners.len()];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            tr!(app.ui.translator, "libraries.title"),
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(THEME.secondary));
    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(
                s,
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}", tr!(app.ui.translator, "libraries.loading")),
                Style::default()
                    .fg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(Alignment::Center),
        inner,
    );
}

fn render_library_search_bar(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let margin = 30;
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(margin),
            Constraint::Min(20),
            Constraint::Percentage(margin),
        ])
        .split(area);
    let search_area = h_chunks[1];

    let count = get_libs_for_selected_process(app).len();
    let cursor = if app.ui.frame_count.is_multiple_of(2) {
        "█"
    } else {
        " "
    };

    let search_line = Line::from(vec![
        Span::styled(
            " 󰅩 SEARCH ",
            Style::default()
                .fg(THEME.background)
                .bg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default()),
        Span::styled(
            &app.libraries.library_search_query,
            Style::default()
                .fg(THEME.text_main)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(cursor, Style::default().fg(THEME.primary)),
        Span::styled(
            format!("  ({})", count),
            Style::default().fg(if count > 0 {
                THEME.success
            } else {
                THEME.danger
            }),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.primary))
        .style(Style::default().bg(ratatui::style::Color::Reset));
    let search_widget = Paragraph::new(search_line).block(block);
    f.render_widget(search_widget, search_area);
}

fn render_process_list(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.ui.sidebar_focus == SidebarFocus::Left
        && app.ui.current_nav_view == NavView::LibraryInspection;
    let border_color = if app.libraries.libraries_loading {
        THEME.warning
    } else if is_focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let border_type = if is_focused {
        BorderType::Thick
    } else {
        BorderType::Rounded
    };

    let title_color = if app.libraries.libraries_loading {
        THEME.warning
    } else {
        THEME.primary
    };
    let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner_char = spinners[(app.ui.frame_count as usize) % spinners.len()];
    let loading_suffix = if app.libraries.libraries_loading {
        format!(
            " ({} {})",
            spinner_char,
            tr!(app.ui.translator, "libraries.loading_libs")
        )
    } else {
        String::new()
    };

    let block = Block::default()
        .title(Span::styled(
            format!(
                " {}{} ",
                tr!(app.ui.translator, "libraries.processes"),
                loading_suffix
            ),
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(border_type);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let groups = group_by_process(app);
    if groups.is_empty() {
        let msg = if app.libraries.libraries_loading {
            Line::from(vec![
                Span::styled(
                    spinner_char,
                    Style::default()
                        .fg(THEME.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {}", tr!(app.ui.translator, "libraries.loading_libs")),
                    Style::default().fg(THEME.text_dim),
                ),
            ])
        } else {
            Line::from(Span::styled(
                tr!(app.ui.translator, "libraries.no_data"),
                Style::default().fg(THEME.text_dim),
            ))
        };
        f.render_widget(Paragraph::new(msg).alignment(Alignment::Center), inner);
        return;
    }

    let header_style = Style::default()
        .fg(THEME.secondary)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);

    let risk_map = build_risk_map(app);

    let mut rows: Vec<Row> = vec![Row::new(vec![
        Cell::from(Span::styled(
            tr!(app.ui.translator, "libraries.column_process"),
            header_style,
        )),
        Cell::from(Span::styled(
            tr!(app.ui.translator, "libraries.column_libs"),
            header_style,
        )),
        Cell::from(Span::styled(
            tr!(app.ui.translator, "libraries.column_risk"),
            header_style,
        )),
    ])];

    for (i, (pname, pcount)) in groups.iter().enumerate() {
        let selected = app.libraries.selected_library_process_index == i;
        let base_style = if selected {
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(THEME.text_main)
        };

        let threat_count = risk_map.get(pname.as_str()).copied().unwrap_or(0);
        let threat_style = if threat_count > 0 {
            Style::default()
                .fg(THEME.danger)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(THEME.text_dim)
        };
        let threat_str = if threat_count > 0 {
            format!("{}", threat_count)
        } else {
            "-".to_string()
        };

        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                truncate_str(
                    &format!(" {}", pname),
                    inner.width.saturating_sub(10) as usize,
                ),
                base_style,
            )),
            Cell::from(Span::styled(
                format!("{}", pcount),
                Style::default().fg(THEME.text_dim),
            )),
            Cell::from(Span::styled(threat_str, threat_style)),
        ]));
    }

    let max_rows = inner.height.saturating_sub(1) as usize;
    let scroll = app.libraries.library_process_scroll;
    let visible: Vec<Row> = rows.iter().skip(scroll).take(max_rows).cloned().collect();

    f.render_widget(
        Table::new(
            visible,
            [
                Constraint::Min(10),
                Constraint::Length(5),
                Constraint::Length(5),
            ],
        ),
        inner,
    );
}

fn render_library_table(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.ui.sidebar_focus == crate::app::SidebarFocus::Center
        && app.ui.current_nav_view == crate::app::NavView::LibraryInspection;
    let border_color = if is_focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let border_type = if is_focused {
        BorderType::Thick
    } else {
        BorderType::Rounded
    };

    let block = Block::default()
        .title(Span::styled(
            format!(" {} ", tr!(app.ui.translator, "libraries.libs")),
            Style::default()
                .fg(THEME.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(border_type);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let libs = get_libs_for_selected_process(app);
    if libs.is_empty() {
        let msg = if !app.libraries.library_search_query.is_empty()
            || app.libraries.library_risk_filter.is_some()
        {
            tr!(app.ui.translator, "libraries.filter_none")
        } else {
            tr!(app.ui.translator, "libraries.filter_none_proc")
        };
        f.render_widget(
            Paragraph::new(msg)
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            inner,
        );
        return;
    }

    let header_style = Style::default()
        .fg(THEME.secondary)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);

    let mut rows: Vec<Row> = vec![Row::new(vec![
        Cell::from(Span::styled(
            tr!(app.ui.translator, "libraries.name"),
            header_style,
        )),
        Cell::from(Span::styled(
            tr!(app.ui.translator, "libraries.size"),
            header_style,
        )),
        Cell::from(Span::styled(
            tr!(app.ui.translator, "libraries.origin"),
            header_style,
        )),
        Cell::from(Span::styled(
            tr!(app.ui.translator, "libraries.sign"),
            header_style,
        )),
        Cell::from(Span::styled(
            tr!(app.ui.translator, "libraries.risk"),
            header_style,
        )),
    ])];

    let search_lc = app.libraries.library_search_query.to_lowercase();

    for (i, lib) in libs.iter().enumerate() {
        let selected = app.libraries.selected_library_index == i;

        let name_display = if !search_lc.is_empty() && lib.name.to_lowercase().contains(&search_lc)
        {
            format!("▶ {}", lib.name)
        } else {
            lib.name.clone()
        };

        let row_style = if selected {
            Style::default()
                .fg(THEME.background)
                .bg(THEME.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(THEME.text_main)
        };

        let _risk_style = risk_color_style(lib);
        let _sign_style = signature_color_style(lib);
        let _origin_style = origin_color_style(lib);

        let size_str = format_size(lib.size);
        let risk_label = risk_display_label(app, lib);
        let sign_label = sign_display_label(lib);
        let origin_label = lib.origin.as_str();

        rows.push(Row::new(vec![
            Cell::from(Span::styled(truncate_str(&name_display, 30), row_style)),
            Cell::from(Span::styled(size_str, row_style)),
            Cell::from(Span::styled(origin_label, row_style)),
            Cell::from(Span::styled(sign_label, row_style)),
            Cell::from(Span::styled(risk_label, row_style)),
        ]));
    }

    let max_rows = inner.height.saturating_sub(1) as usize;
    let scroll = app.libraries.library_lib_scroll;
    let visible: Vec<Row> = rows.iter().skip(scroll).take(max_rows).cloned().collect();

    f.render_widget(
        Table::new(
            visible,
            [
                Constraint::Min(16),
                Constraint::Length(8),
                Constraint::Length(9),
                Constraint::Length(8),
                Constraint::Length(11),
            ],
        ),
        inner,
    );
}

fn render_library_actions_panel(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.ui.sidebar_focus == SidebarFocus::Right
        && app.ui.current_nav_view == NavView::LibraryInspection;
    let border_color = if is_focused {
        THEME.primary
    } else {
        THEME.secondary
    };
    let border_type = if is_focused {
        BorderType::Thick
    } else {
        BorderType::Rounded
    };

    let t = &app.ui.translator;
    let actions: Vec<(&str, String, &str, ratatui::style::Color)> = vec![
        (
            "󰑐",
            tr!(t, "libraries.action_refresh").to_string(),
            "R",
            THEME.primary,
        ),
        (
            "󰒓",
            tr!(t, "libraries.action_filter").to_string(),
            "F",
            THEME.warning,
        ),
        (
            "󰅍",
            tr!(t, "libraries.action_copy").to_string(),
            "Enter",
            THEME.secondary,
        ),
        (
            "󰒈",
            tr!(t, "libraries.action_export_json").to_string(),
            "J",
            THEME.secondary,
        ),
        (
            "󰈸",
            tr!(t, "libraries.action_export_csv").to_string(),
            "C",
            THEME.secondary,
        ),
        (
            "󰄉",
            tr!(t, "libraries.action_hash").to_string(),
            "H",
            THEME.accent,
        ),
        (
            "󰈔",
            tr!(t, "libraries.action_view_binary").to_string(),
            "V",
            THEME.primary,
        ),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" 󰬒 {} ", tr!(&app.ui.translator, "actions.title")))
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(border_color))
        .border_type(border_type);
    f.render_widget(block.clone(), area);

    let inner_area = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(config::SCROLLBAR_WIDTH),
        ])
        .split(inner_area);
    let list_area = chunks[0];
    let scrollbar_area = chunks[1];

    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(i, (icon, title, key, color))| {
            let is_selected = i == app.ui.selected_action_index;
            let prefix = if is_selected { " ▎" } else { "  " };
            let prefix_style = if is_selected {
                Style::default().fg(THEME.primary)
            } else {
                Style::default()
            };
            let title_style = if is_selected {
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_main)
            };
            let content = vec![
                Line::from(vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled(format!(" {} ", icon), Style::default().fg(*color)),
                    Span::styled(title.as_str(), title_style),
                ]),
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(format!("[ {} ]", key), Style::default().fg(THEME.text_dim)),
                ]),
            ];
            ListItem::new(content)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.ui.selected_action_index));
    let list = List::new(items).block(Block::default());
    f.render_stateful_widget(list, list_area, &mut list_state);
    widgets::render_scrollbar(
        f,
        scrollbar_area,
        actions.len(),
        app.ui.selected_action_index,
    );
}

fn render_selected_library_info(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let libs = get_libs_for_selected_process(app);
    let lib = libs.get(app.libraries.selected_library_index);

    let block = Block::default()
        .title(Span::styled(
            tr!(app.ui.translator, "libraries.details"),
            Style::default()
                .fg(THEME.secondary)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.secondary));

    let inner = block.inner(area);

    let text: Vec<Line> = if let Some(l) = lib {
        let risk_style = risk_color_style(l);
        let sign_style = signature_color_style(l);
        let origin_style = origin_color_style(l);

        let hash_display = if l.sha256.is_empty() {
            Span::styled(
                tr!(app.ui.translator, "libraries.hash_not_computed"),
                Style::default().fg(THEME.text_dim),
            )
        } else {
            Span::styled(l.sha256.clone(), Style::default().fg(THEME.text_dim))
        };

        let size_str = format_size(l.size);

        vec![
            Line::from(vec![
                Span::styled(
                    tr!(app.ui.translator, "libraries.detail_name"),
                    Style::default()
                        .fg(THEME.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    l.name.clone(),
                    Style::default()
                        .fg(THEME.text_main)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "),
                Span::styled(
                    format!("[ {} ]", l.risk),
                    risk_style.add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    tr!(app.ui.translator, "libraries.detail_path"),
                    Style::default()
                        .fg(THEME.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    truncate_str(&l.path, (inner.width as usize).saturating_sub(12)),
                    Style::default().fg(THEME.text_dim),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    tr!(app.ui.translator, "libraries.detail_size"),
                    Style::default()
                        .fg(THEME.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(size_str, Style::default().fg(THEME.text_main)),
                Span::raw("   "),
                Span::styled(
                    tr!(app.ui.translator, "libraries.detail_origin"),
                    Style::default()
                        .fg(THEME.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(l.origin.as_str(), origin_style),
                Span::raw("   "),
                Span::styled(
                    tr!(app.ui.translator, "libraries.detail_sign"),
                    Style::default()
                        .fg(THEME.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(l.signature.as_str(), sign_style),
            ]),
            Line::from(vec![
                Span::styled(
                    tr!(app.ui.translator, "libraries.detail_pid"),
                    Style::default()
                        .fg(THEME.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ({})", l.pid, l.process_name),
                    Style::default().fg(THEME.text_main),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    tr!(app.ui.translator, "libraries.detail_hash"),
                    Style::default()
                        .fg(THEME.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                hash_display,
            ]),
            Line::from(vec![Span::styled(
                tr!(app.ui.translator, "libraries.hint_actions"),
                Style::default().fg(THEME.text_dim),
            )]),
        ]
    } else {
        vec![Line::from(Span::styled(
            tr!(app.ui.translator, "libraries.select_lib"),
            Style::default().fg(THEME.text_dim),
        ))]
    };

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn group_by_process(app: &App) -> Vec<(String, usize)> {
    app.group_libs_by_process()
}

pub fn get_libs_for_selected_process(app: &App) -> Vec<&LibraryInfo> {
    let groups = app.group_libs_by_process();
    let pname = groups
        .get(app.libraries.selected_library_process_index)
        .map(|(n, _)| n.as_str())
        .unwrap_or("");

    let search_lc = app.libraries.library_search_query.to_lowercase();
    let risk_filter = app.libraries.library_risk_filter.as_deref();

    app.libraries
        .libraries
        .iter()
        .filter(|l| {
            l.process_name == pname
                && (search_lc.is_empty()
                    || l.name.to_lowercase().contains(&search_lc)
                    || l.path.to_lowercase().contains(&search_lc))
                && match risk_filter {
                    Some(f) => l.risk == f,
                    None => true,
                }
        })
        .collect()
}

fn build_risk_map(app: &App) -> std::collections::HashMap<&str, usize> {
    let mut map = std::collections::HashMap::new();
    for lib in &app.libraries.libraries {
        if lib.risk == "Suspicious" || lib.risk == "Critical" {
            *map.entry(lib.process_name.as_str()).or_insert(0) += 1;
        }
    }
    map
}

fn risk_color_style(lib: &LibraryInfo) -> Style {
    match lib.risk.as_str() {
        "Critical" => Style::default()
            .fg(THEME.danger)
            .add_modifier(Modifier::BOLD),
        "Suspicious" => Style::default()
            .fg(THEME.warning)
            .add_modifier(Modifier::BOLD),
        "Safe" => Style::default().fg(THEME.success),
        _ => Style::default().fg(THEME.text_dim),
    }
}

fn signature_color_style(lib: &LibraryInfo) -> Style {
    match lib.signature {
        SignatureStatus::Signed => Style::default().fg(THEME.success),
        SignatureStatus::Unsigned => Style::default()
            .fg(THEME.warning)
            .add_modifier(Modifier::BOLD),
        SignatureStatus::Invalid => Style::default()
            .fg(THEME.danger)
            .add_modifier(Modifier::BOLD),
        SignatureStatus::Unknown => Style::default().fg(THEME.text_dim),
    }
}

fn origin_color_style(lib: &LibraryInfo) -> Style {
    match lib.origin {
        LibraryOrigin::Temp => Style::default()
            .fg(THEME.danger)
            .add_modifier(Modifier::BOLD),
        LibraryOrigin::UserSpace => Style::default().fg(THEME.warning),
        LibraryOrigin::System => Style::default().fg(THEME.success),
        LibraryOrigin::ProgramFiles => Style::default().fg(THEME.text_main),
        LibraryOrigin::Unknown => Style::default().fg(THEME.text_dim),
    }
}

fn risk_display_label(app: &App, lib: &LibraryInfo) -> String {
    match lib.risk.as_str() {
        "Critical" => tr!(app.ui.translator, "libraries.risk_critical").to_string(),
        "Suspicious" => tr!(app.ui.translator, "libraries.risk_suspicious").to_string(),
        "Safe" => tr!(app.ui.translator, "libraries.risk_safe").to_string(),
        _ => "Unknown".to_string(),
    }
}

fn sign_display_label(lib: &LibraryInfo) -> String {
    match lib.signature {
        SignatureStatus::Signed => "✔ Signed".to_string(),
        SignatureStatus::Unsigned => "✗ Unsign".to_string(),
        SignatureStatus::Invalid => "! Invalid".to_string(),
        SignatureStatus::Unknown => "? Unknwn".to_string(),
    }
}

fn format_size(size: u64) -> String {
    if size > 1024 * 1024 {
        format!("{:.1}MB", size as f64 / (1024.0 * 1024.0))
    } else if size > 1024 {
        format!("{:.1}KB", size as f64 / 1024.0)
    } else {
        format!("{}B", size)
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if max < 2 {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max {
        let end = max.saturating_sub(1);
        chars[..end].iter().collect::<String>() + "…"
    } else {
        s.to_string()
    }
}

pub fn render_library_hash_modal(f: &mut ratatui::Frame, app: &App) {
    let popup_area = Rect {
        x: f.area().width / 5,
        y: f.area().height / 3,
        width: f.area().width * 3 / 5,
        height: 13,
    };
    let t = &app.ui.translator;
    let dialog = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  󰄉  {} ", tr!(t, "libraries.hash_modal_title")),
            Style::default()
                .fg(THEME.warning)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {} ", tr!(t, "libraries.hash_modal_desc1")),
            Style::default().fg(THEME.text_main),
        )]),
        Line::from(vec![Span::styled(
            format!("  {} ", tr!(t, "libraries.hash_modal_desc2")),
            Style::default().fg(THEME.text_main),
        )]),
        Line::from(vec![Span::styled(
            format!("  {} ", tr!(t, "libraries.hash_modal_desc3")),
            Style::default().fg(THEME.text_main),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(
                format!(" {} ", tr!(t, "libraries.hash_modal_continue")),
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  [Enter]  ", Style::default().fg(THEME.text_dim)),
            Span::styled(
                format!(" {} ", tr!(t, "libraries.hash_modal_cancel")),
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  [Esc]", Style::default().fg(THEME.text_dim)),
        ]),
    ];
    let p = Paragraph::new(dialog)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" 󰄉 {} ", tr!(t, "libraries.hash_modal_title")))
                .title_style(
                    Style::default()
                        .fg(THEME.warning)
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(THEME.warning))
                .border_type(BorderType::Thick),
        )
        .style(Style::default())
        .alignment(Alignment::Left);
    f.render_widget(Clear, popup_area);
    f.render_widget(p, popup_area);
}

pub fn render_library_binary_viewer(f: &mut ratatui::Frame, app: &App) {
    let popup_area = Rect {
        x: f.area().width / 10,
        y: f.area().height / 10,
        width: f.area().width * 8 / 10,
        height: f.area().height * 8 / 10,
    };
    f.render_widget(Clear, popup_area);

    let t = &app.ui.translator;
    let tab_labels = [
        ("󰈔", tr!(t, "libraries.binary_viewer_hex")),
        ("󰌠", tr!(t, "libraries.binary_viewer_disasm")),
    ];
    let mut title_spans = vec![Span::raw(" ")];
    for (i, (icon, label)) in tab_labels.iter().enumerate() {
        let active = i == app.libraries.library_binary_tab;
        if i > 0 {
            title_spans.push(Span::raw("  "));
        }
        title_spans.push(Span::styled(
            if active {
                format!("▎[{}] {} ", icon, label)
            } else {
                format!(" [{}] {} ", icon, label)
            },
            if active {
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(THEME.text_dim)
            },
        ));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(THEME.primary))
        .title(Line::from(title_spans))
        .title_style(Style::default().fg(THEME.primary));
    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let lines = if app.libraries.library_binary_tab == 0 {
        &app.libraries.library_binary_hex_lines
    } else {
        &app.libraries.library_binary_disasm_lines
    };

    if lines.is_empty() {
        f.render_widget(
            Paragraph::new("No data")
                .style(Style::default().fg(THEME.text_dim))
                .alignment(Alignment::Center),
            inner,
        );
        return;
    }

    let total = lines.len();
    let visible_height = inner.height.saturating_sub(2) as usize;
    let scroll = app
        .libraries
        .library_binary_scroll
        .min(total.saturating_sub(visible_height));

    let mut rendered = Vec::with_capacity(visible_height + 2);
    rendered.push(Line::from(vec![Span::styled(
        format!(" {} ({}) ", tr!(t, "libraries.binary_viewer_scroll"), total),
        Style::default().fg(THEME.background).bg(THEME.primary),
    )]));

    for i in scroll..(scroll + visible_height).min(total) {
        if let Some(line) = lines.get(i) {
            if app.libraries.library_binary_tab == 0 {
                rendered.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default().fg(THEME.text_main),
                )));
            } else {
                if line.starts_with(';') {
                    rendered.push(Line::from(Span::styled(
                        line.clone(),
                        Style::default()
                            .fg(THEME.text_dim)
                            .add_modifier(Modifier::ITALIC),
                    )));
                } else if line.contains("  ret ")
                    || line.contains("  jmp ")
                    || line.contains("  call ")
                {
                    rendered.push(Line::from(Span::styled(
                        line.clone(),
                        Style::default().fg(THEME.primary),
                    )));
                } else {
                    rendered.push(Line::from(Span::styled(
                        line.clone(),
                        Style::default().fg(THEME.text_main),
                    )));
                }
            }
        }
    }

    f.render_widget(Paragraph::new(rendered).style(Style::default()), inner);

    if total > visible_height {
        let scrollbar = ratatui::widgets::Scrollbar::default()
            .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(THEME.primary));
        let mut scrollbar_state = ratatui::widgets::ScrollbarState::new(total).position(scroll);
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
