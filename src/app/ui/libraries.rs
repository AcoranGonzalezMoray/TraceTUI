use super::theme::THEME;
use crate::app::libraries::{LibraryInfo, LibraryOrigin, SignatureStatus};
use crate::app::App;
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

pub fn render_libraries_view(f: &mut ratatui::Frame, app: &App, area: Rect) {
    if app.libraries_loading {
        render_loading(f, app, area);
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    render_search_bar(f, app, rows[0]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
        .split(rows[1]);

    render_process_list(f, app, cols[0]);
    render_library_right_panel(f, app, cols[1]);
}

fn render_loading(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            tr!(app.translator, "libraries.title"),
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(THEME.secondary));
    let inner = block.inner(area);
    f.render_widget(block, area);
    f.render_widget(
        Paragraph::new(tr!(app.translator, "libraries.loading"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(THEME.text_dim)),
        inner,
    );
}

fn render_search_bar(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_searching = app.library_search_active;
    let border_color = if is_searching {
        THEME.primary
    } else {
        THEME.secondary
    };
    let border_type = if is_searching {
        BorderType::Thick
    } else {
        BorderType::Rounded
    };

    let filter_tag = match app.library_risk_filter.as_deref() {
        Some("Critical") => "  [Filter: CRITICAL]",
        Some("Suspicious") => "  [Filter: SUSPICIOUS]",
        _ => "",
    };

    let total = app.libraries.len();
    let shown = get_libs_for_selected_process(app).len();
    let stats = format!("  {}/{} libs{}", shown, total, filter_tag);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(
                " {} | [/] Search  [F] Filter Risk  [J/C] Export JSON/CSV  [H] Hash{}",
                tr!(app.translator, "libraries.title"),
                stats
            ),
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let search_display = if app.library_search_query.is_empty() {
        if is_searching {
            "Type to search libraries...".to_string()
        } else {
            "Press [/] to search".to_string()
        }
    } else {
        format!("Search: {}", app.library_search_query)
    };

    let style = if is_searching {
        Style::default().fg(THEME.text_main)
    } else {
        Style::default().fg(THEME.text_dim)
    };

    f.render_widget(Paragraph::new(search_display).style(style), inner);
}

fn render_process_list(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.sidebar_focus == crate::app::SidebarFocus::Left
        && app.current_nav_view == crate::app::NavView::LibraryInspection;
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
            format!(" {} ", tr!(app.translator, "libraries.processes")),
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(border_type);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let groups = group_by_process(app);
    if groups.is_empty() {
        f.render_widget(
            Paragraph::new(tr!(app.translator, "libraries.no_data"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            inner,
        );
        return;
    }

    let header_style = Style::default()
        .fg(THEME.secondary)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);

    let risk_map = build_risk_map(app);

    let mut rows: Vec<Row> = vec![Row::new(vec![
        Cell::from(Span::styled("Process", header_style)),
        Cell::from(Span::styled("Libs", header_style)),
        Cell::from(Span::styled("!", header_style)),
    ])];

    for (i, (pname, pcount)) in groups.iter().enumerate() {
        let selected = app.selected_library_process_index == i;
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
    let scroll = app.library_process_scroll;
    let visible: Vec<Row> = rows.iter().skip(scroll).take(max_rows).cloned().collect();

    f.render_widget(
        Table::new(
            visible,
            [
                Constraint::Min(10),
                Constraint::Length(5),
                Constraint::Length(3),
            ],
        ),
        inner,
    );
}

fn render_library_right_panel(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(10)])
        .split(area);

    render_library_table(f, app, chunks[0]);
    render_selected_library_info(f, app, chunks[1]);
}

fn render_library_table(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.sidebar_focus == crate::app::SidebarFocus::Center
        && app.current_nav_view == crate::app::NavView::LibraryInspection;
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
            format!(" {} ", tr!(app.translator, "libraries.libs")),
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
        let msg = if !app.library_search_query.is_empty() || app.library_risk_filter.is_some() {
            "No libraries match current filter"
        } else {
            "No libraries found for this process"
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
            tr!(app.translator, "libraries.name"),
            header_style,
        )),
        Cell::from(Span::styled(
            tr!(app.translator, "libraries.size"),
            header_style,
        )),
        Cell::from(Span::styled("Origin", header_style)),
        Cell::from(Span::styled("Sign", header_style)),
        Cell::from(Span::styled(
            tr!(app.translator, "libraries.risk"),
            header_style,
        )),
    ])];

    let search_lc = app.library_search_query.to_lowercase();

    for (i, lib) in libs.iter().enumerate() {
        let selected = app.selected_library_index == i;

        let name_display = if !search_lc.is_empty() && lib.name.to_lowercase().contains(&search_lc)
        {
            format!("▶ {}", lib.name)
        } else {
            lib.name.clone()
        };

        let base_style = if selected {
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::REVERSED)
        } else {
            Style::default().fg(THEME.text_main)
        };

        let risk_style = risk_color_style(lib);
        let sign_style = signature_color_style(lib);
        let origin_style = origin_color_style(lib);

        let size_str = format_size(lib.size);
        let risk_label = risk_display_label(app, lib);
        let sign_label = sign_display_label(lib);
        let origin_label = lib.origin.as_str();

        rows.push(Row::new(vec![
            Cell::from(Span::styled(truncate_str(&name_display, 30), base_style)),
            Cell::from(Span::styled(size_str, Style::default().fg(THEME.text_dim))),
            Cell::from(Span::styled(origin_label, origin_style)),
            Cell::from(Span::styled(sign_label, sign_style)),
            Cell::from(Span::styled(risk_label, risk_style)),
        ]));
    }

    let max_rows = inner.height.saturating_sub(1) as usize;
    let scroll = app.library_lib_scroll;
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

fn render_selected_library_info(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let libs = get_libs_for_selected_process(app);
    let lib = libs.get(app.selected_library_index);

    let block = Block::default()
        .title(Span::styled(
            tr!(app.translator, "libraries.details"),
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
                "(not computed — press H)",
                Style::default().fg(THEME.text_dim),
            )
        } else {
            Span::styled(l.sha256.clone(), Style::default().fg(THEME.text_dim))
        };

        let size_str = format_size(l.size);

        vec![
            Line::from(vec![
                Span::styled(
                    " Name:  ",
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
                    " Path:  ",
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
                    " Size:  ",
                    Style::default()
                        .fg(THEME.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(size_str, Style::default().fg(THEME.text_main)),
                Span::raw("   "),
                Span::styled(
                    "Origin: ",
                    Style::default()
                        .fg(THEME.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(l.origin.as_str(), origin_style),
                Span::raw("   "),
                Span::styled(
                    "Sign: ",
                    Style::default()
                        .fg(THEME.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(l.signature.as_str(), sign_style),
            ]),
            Line::from(vec![
                Span::styled(
                    " PID:   ",
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
                    " Hash:  ",
                    Style::default()
                        .fg(THEME.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                hash_display,
            ]),
            Line::from(vec![Span::styled(
                " [Enter] Copy path   [H] Compute hash   [J] Export JSON   [C] Export CSV",
                Style::default().fg(THEME.text_dim),
            )]),
        ]
    } else {
        vec![Line::from(Span::styled(
            tr!(app.translator, "libraries.select_lib"),
            Style::default().fg(THEME.text_dim),
        ))]
    };

    f.render_widget(Paragraph::new(text).block(block), area);
}

fn group_by_process(app: &App) -> Vec<(String, usize)> {
    app.group_libs_by_process()
}

pub fn get_libs_for_selected_process<'a>(app: &'a App) -> Vec<&'a LibraryInfo> {
    let groups = app.group_libs_by_process();
    let pname = groups
        .get(app.selected_library_process_index)
        .map(|(n, _)| n.as_str())
        .unwrap_or("");

    let search_lc = app.library_search_query.to_lowercase();
    let risk_filter = app.library_risk_filter.as_deref();

    app.libraries
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

fn build_risk_map<'a>(app: &'a App) -> std::collections::HashMap<&'a str, usize> {
    let mut map = std::collections::HashMap::new();
    for lib in &app.libraries {
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
        "Critical" => tr!(app.translator, "libraries.risk_critical").to_string(),
        "Suspicious" => tr!(app.translator, "libraries.risk_suspicious").to_string(),
        "Safe" => tr!(app.translator, "libraries.risk_safe").to_string(),
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
