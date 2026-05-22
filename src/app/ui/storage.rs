use crate::app::storage::{FileEntry, fmt_size, FILE_EXTENSION_FILTERS};
use crate::app::ui::theme::THEME;
use crate::app::App;
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, ListItem, ListState, 
        Paragraph, Row, Table, TableState, Wrap, Scrollbar, ScrollbarOrientation, ScrollbarState
    },
};

pub fn render_storage_view(f: &mut ratatui::Frame, app: &App, area: Rect) {
    if area.height < 10 || area.width < 50 {
        return;
    }
    if app.show_file_viewer {
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

    render_disk_list(f, app, chunks[0]);
    render_file_browser(f, app, chunks[1]);
    render_storage_actions(f, app, chunks[2]);
}

// ── Lista de Discos (Panel Izquierdo) ──

fn render_disk_list(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.storage_focus == 0;
    
    let border_color = if is_focused { THEME.primary } else { THEME.secondary };
    let border_style = if is_focused { Style::default().fg(border_color) } else { Style::default().fg(border_color).dim() };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled(tr!(app.translator, "storage.disks"), Style::default().fg(THEME.primary).bold()),
            Span::raw(" "),
        ]));
        
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.disks.is_empty() {
        let msg = if app.disks_loading {
            tr!(app.translator, "storage.loading")
        } else {
            tr!(app.translator, "storage.no_disks")
        };
        f.render_widget(
            Paragraph::new(msg).alignment(Alignment::Center).style(Style::default().fg(THEME.text_dim)),
            inner,
        );
        return;
    }

    let items: Vec<ListItem> = app
        .disks
        .iter()
        .enumerate()
        .map(|(i, disk)| {
            let is_selected = i == app.selected_disk_index;
            let pct = disk.usage_pct();
            
            let bar_len = 8;
            let filled = ((pct / 100.0) * bar_len as f64).round() as usize;
            let bar: String = std::iter::repeat('■').take(filled.min(bar_len))
                .chain(std::iter::repeat('▱').take(bar_len.saturating_sub(filled)))
                .collect();

            let label_line = vec![
                Span::styled(format!(" {}: ", disk.device), Style::default().fg(if is_selected { THEME.background } else { THEME.text_main }).bold()),
                Span::styled(format!("{} ", bar), Style::default().fg(if is_selected { THEME.background } else { if pct > 85.0 { THEME.danger } else { THEME.success } })),
                Span::styled(format!("{:.0}%", pct), Style::default().fg(if is_selected { THEME.background } else { THEME.text_dim })),
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
    state.select(Some(app.selected_disk_index));
    f.render_stateful_widget(List::new(items), inner, &mut state);
}

// ── Explorador de Archivos (Panel Central) ──

fn file_icon(entry: &FileEntry) -> &'static str {
    if entry.is_dir { return ""; }
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
    if entry.is_dir { return THEME.primary; }
    match entry.extension.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => THEME.success,
        "mp3" | "wav" | "flac" | "mp4" | "mkv" | "mov" => THEME.warning,
        "zip" | "tar" | "gz" | "7z" | "rar" => THEME.accent,
        "exe" | "msi" | "sh" | "bat" => THEME.danger,
        _ => THEME.text_main,
    }
}

fn render_file_browser(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.storage_focus == 1;
    
    let border_color = if is_focused { THEME.primary } else { THEME.secondary };
    let border_style = if is_focused { Style::default().fg(border_color) } else { Style::default().fg(border_color).dim() };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled(tr!(app.translator, "storage.file_browser"), Style::default().fg(THEME.primary).bold()),
            Span::raw(" "),
        ]));
        
    let inner = block.inner(area);
    f.render_widget(block, area);

    let path_str = format!(" 📁 {} ", app.current_directory.to_string_lossy());
    let header_path = Paragraph::new(path_str)
        .style(Style::default().fg(THEME.text_main).bg(THEME.secondary).dim())
        .alignment(Alignment::Left);
        
    if inner.height > 2 {
        f.render_widget(header_path, Rect::new(inner.x, inner.y, inner.width, 1));
    }

    let rest = Rect::new(inner.x, inner.y + 1, inner.width, inner.height.saturating_sub(1));

    if app.search_progress_running {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let s = spinner[(app.frame_count as usize) % spinner.len()];
        let mut lines = vec![Line::from("")];
        
        let loading_msg = if app.file_search_recursive {
            format!(" {} {} {} files...", s, tr!(app.translator, "status.search_recursive_on"), app.search_progress_found)
        } else {
            format!(" {} {} {} files...", s, tr!(app.translator, "storage.loading"), app.search_progress_found)
        };
        
        lines.push(Line::from(Span::styled(loading_msg, Style::default().fg(THEME.warning).bold())));
        f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), rest);
        return;
    }

    let query = app.file_search_query.to_lowercase();
    let ext_idx = app.file_search_extension_idx.min(FILE_EXTENSION_FILTERS.len().saturating_sub(1));
    let (_, _, ext_list) = FILE_EXTENSION_FILTERS[ext_idx];
    
    let filtered: Vec<&FileEntry> = if app.file_search_mode {
        app.file_entries.iter().filter(|e| {
            let matches_query = query.is_empty() || e.name.to_lowercase().contains(&query);
            let matches_ext = ext_list.is_empty() || ext_list.contains(&e.extension.to_lowercase().as_str());
            matches_query && matches_ext
        }).collect()
    } else {
        app.file_entries.iter().collect()
    };

    if filtered.is_empty() {
        f.render_widget(
            Paragraph::new(tr!(app.translator, "storage.empty_dir"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            rest,
        );
        return;
    }

    let widths = [
        Constraint::Fill(1),      
        Constraint::Length(11),     
        Constraint::Length(18),     
    ];

    let header_row = Row::new(vec![
        Cell::from(Line::from(vec![Span::raw("  "), Span::raw(tr!(app.translator, "storage.col_name"))])),
        Cell::from(Line::from(vec![Span::raw(tr!(app.translator, "storage.col_size"))])),
        Cell::from(Line::from(vec![Span::raw(tr!(app.translator, "storage.col_modified"))])),
    ])
    .style(Style::default().fg(THEME.primary).add_modifier(Modifier::BOLD))
    .bottom_margin(1);

    let rows: Vec<Row> = filtered.iter().map(|entry| {
        let icon = file_icon(entry);
        let fg_color = file_color(entry);

        let size_str = if entry.is_dir { "  <DIR>".to_string() } else { fmt_size(entry.size) };

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
            Cell::from(Line::from(Span::styled(size_str, Style::default().fg(fg_color)))),
            Cell::from(Line::from(Span::styled(&entry.modified, Style::default().fg(THEME.text_dim)))),
        ])
    }).collect();

    let total_items = filtered.len();
    let current_scroll = app.file_scroll.min(total_items.saturating_sub(1));

    let mut table_state = TableState::default();
    table_state.select(Some(current_scroll));

    let table = Table::new(rows, widths)
        .header(header_row)
        .highlight_style(Style::default().bg(THEME.primary).dim().fg(THEME.background).add_modifier(Modifier::BOLD))
        .highlight_symbol(" "); 

    f.render_stateful_widget(table, rest, &mut table_state);

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"))
        .track_symbol(Some("│"))
        .thumb_symbol("█")
        .style(Style::default().fg(THEME.primary));

    let mut scrollbar_state = ScrollbarState::new(total_items)
        .position(current_scroll);

    f.render_stateful_widget(
        scrollbar, 
        rest.inner(ratatui::layout::Margin { vertical: 2, horizontal: 0 }), 
        &mut scrollbar_state
    );
}

// ── Panel de Acciones (Panel Derecho) ──

fn render_storage_actions(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.storage_focus == 2;
    
    let border_color = if is_focused { THEME.primary } else { THEME.secondary };
    let border_style = if is_focused { Style::default().fg(border_color) } else { Style::default().fg(border_color).dim() };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled(tr!(app.translator, "storage.actions"), Style::default().fg(THEME.primary).bold()),
            Span::raw(" "),
        ]));
        
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Corregido: Obtenemos los valores de traducción antes de construir el array de referencias
    let refresh_lbl = tr!(app.translator, "storage.refresh");
    let open_lbl = tr!(app.translator, "storage.open");
    let prop_lbl = tr!(app.translator, "storage.properties");
    let parent_lbl = tr!(app.translator, "storage.parent_dir");
    let home_lbl = tr!(app.translator, "storage.go_home");

    let items = [
        ("\u{f021}", &refresh_lbl, "R", THEME.success),
        ("\u{f15b}", &open_lbl, "Enter", THEME.primary),
        ("\u{f0ca}", &prop_lbl, "P", THEME.warning),
        ("\u{f07c}", &parent_lbl, "Backspace", THEME.accent),
        ("\u{f015}", &home_lbl, "H", THEME.danger),
    ];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(inner);

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|(icon, lbl, key, color)| {
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!(" {} ", icon), Style::default().fg(*color).add_modifier(Modifier::BOLD)),
                    Span::styled((*lbl).as_str(), Style::default().fg(THEME.text_main)), // Corregido: .as_str() evita transferir ownership
                ]),
                Line::from(vec![
                    Span::raw("     "),
                    Span::styled(format!("❪ {} ❫", key), Style::default().fg(THEME.text_dim).add_modifier(Modifier::ITALIC)),
                ]),
                Line::from(""), 
            ])
        })
        .collect();

    let mut state = ListState::default();
    f.render_stateful_widget(List::new(list_items), chunks[0], &mut state);

    let footer_text = Line::from(vec![
        Span::styled(" Tab ", Style::default().bg(THEME.secondary).fg(THEME.text_main)),
        Span::raw(" Navig  "),
        Span::styled(" ↑↓ ", Style::default().bg(THEME.secondary).fg(THEME.text_main)),
        Span::raw(" Scroll"),
    ]).alignment(Alignment::Center);
    
    f.render_widget(Paragraph::new(footer_text), chunks[1]);
}

// ── Visor de Archivos (Modal) ──

pub fn render_file_viewer_modal(f: &mut ratatui::Frame, app: &App) {
    let area = centered_rect(f.area(), 85, 85); 
    f.render_widget(Clear, area);

    let path = app.current_directory.join(
        app.file_entries.get(0).map(|e| e.name.as_str()).unwrap_or(""),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.primary))
        .title(Line::from(vec![
            Span::raw(" 📄 "),
            Span::styled(path.file_name().unwrap_or_default().to_string_lossy(), Style::default().fg(THEME.primary).bold()),
            Span::raw(" "),
        ]));
        
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.file_viewer_content.is_empty() {
        f.render_widget(
            Paragraph::new(tr!(app.translator, "storage.viewer_empty"))
                .alignment(Alignment::Center)
                .style(Style::default().fg(THEME.text_dim)),
            inner,
        );
        return;
    }

    let total_lines = app.file_viewer_content.len();
    let visible_height = inner.height.saturating_sub(2) as usize;
    let scroll_pos = app.file_viewer_scroll.min(total_lines.saturating_sub(visible_height).max(0));

    let mut lines: Vec<Line> = Vec::with_capacity(visible_height + 1);
    
    lines.push(Line::from(vec![
        Span::styled(format!(" 📑 Lines {}/{} ", (scroll_pos + visible_height).min(total_lines), total_lines), Style::default().fg(THEME.text_dim)),
        Span::styled(format!(" ({})", tr!(app.translator, "storage.viewer_scroll_hint")), Style::default().fg(THEME.text_dim).add_modifier(Modifier::ITALIC)),
    ]).bg(THEME.secondary).dim());
    lines.push(Line::from("")); 

    for i in scroll_pos..(scroll_pos + visible_height).min(total_lines) {
        if let Some(line) = app.file_viewer_content.get(i) {
            let line_num = i + 1;
            lines.push(Line::from(vec![
                Span::styled(format!("{:>4} │ ", line_num), Style::default().fg(THEME.secondary)),
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
            inner.inner(ratatui::layout::Margin { vertical: 1, horizontal: 0 }),
            &mut scrollbar_state,
        );
    }
}

// ── Modal de Búsqueda y Filtros ──

pub fn render_file_search_modal(f: &mut ratatui::Frame, app: &App) {
    let area = centered_rect(f.area(), 55, 45);
    f.render_widget(Clear, area);

    let state = &app.file_search_state;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.primary))
        .title(Line::from(vec![
            Span::raw(" 🔍 "),
            Span::styled(tr!(app.translator, "storage.search_title"), Style::default().fg(THEME.warning).bold()),
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

    // ── Campo 0: Input del término ──
    let q_focused = state.focused_field == 0;
    let q_text = if state.query.is_empty() {
        String::from("type to filter...")
    } else {
        let cursor = if q_focused && app.frame_count % 2 == 0 { "█" } else { "" };
        format!("{}{}", state.query, cursor)
    };
    
    let border_color_q = if q_focused { THEME.primary } else { THEME.secondary };
    let border_style_q = if q_focused { Style::default().fg(border_color_q) } else { Style::default().fg(border_color_q).dim() };

    let q_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style_q)
        .title(Span::styled(tr!(app.translator, "storage.search_query"), Style::default().fg(if q_focused { THEME.primary } else { THEME.text_dim })));
    
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(q_text, Style::default().fg(if state.query.is_empty() { THEME.text_dim } else { THEME.text_main })))).block(q_block),
        chunks[0],
    );

    // ── Campo 1: Búsqueda Recursiva ──
    let r_focused = state.focused_field == 1;
    let r_icon = if state.recursive { " ✅ " } else { " ❌ " };
    let r_label = if state.recursive {
        tr!(app.translator, "status.search_recursive_on")
    } else {
        tr!(app.translator, "status.search_recursive_off")
    };

    let border_color_r = if r_focused { THEME.primary } else { THEME.secondary };
    let border_style_r = if r_focused { Style::default().fg(border_color_r) } else { Style::default().fg(border_color_r).dim() };

    let r_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style_r)
        .title(Span::styled(tr!(app.translator, "storage.search_recursive"), Style::default().fg(if r_focused { THEME.primary } else { THEME.text_dim })));
    
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(r_icon),
            Span::styled(r_label, Style::default().fg(THEME.text_main)),
        ])).block(r_block),
        chunks[1],
    );

    // ── Campo 2: Selector de Extensiones ──
    let e_focused = state.focused_field == 2;
    let ext_idx = state.extension_idx.min(FILE_EXTENSION_FILTERS.len().saturating_sub(1));
    let (ext_icon, ext_name, _) = FILE_EXTENSION_FILTERS[ext_idx];

    let border_color_e = if e_focused { THEME.primary } else { THEME.secondary };
    let border_style_e = if e_focused { Style::default().fg(border_color_e) } else { Style::default().fg(border_color_e).dim() };

    let e_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style_e)
        .title(Span::styled(tr!(app.translator, "storage.search_filetype"), Style::default().fg(if e_focused { THEME.primary } else { THEME.text_dim })));
    
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!(" {} ", ext_icon), Style::default().fg(THEME.accent)),
            Span::styled(ext_name, Style::default().fg(THEME.text_main).add_modifier(Modifier::BOLD)),
            Span::styled("  ◀ ▶", Style::default().fg(THEME.text_dim)),
        ])).block(e_block),
        chunks[2],
    );

    // ── Botonera Inferior (Aceptar / Cancelar) ──
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[3]);

    let cont_focused = state.focused_field == 3;
    let cont_text = format!(" ✔ {} ", tr!(app.translator, "dialog.continue"));
    f.render_widget(
        Paragraph::new(Line::from(Span::raw(&cont_text)))
            .alignment(Alignment::Center)
            .style(if cont_focused { Style::default().fg(THEME.background).bg(THEME.success).add_modifier(Modifier::BOLD) } else { Style::default().fg(THEME.success) }),
        btn_chunks[0],
    );

    let cancel_focused = state.focused_field == 4;
    let cancel_text = format!(" ✘ {} ", tr!(app.translator, "dialog.cancel"));
    f.render_widget(
        Paragraph::new(Line::from(Span::raw(&cancel_text)))
            .alignment(Alignment::Center)
            .style(if cancel_focused { Style::default().fg(THEME.background).bg(THEME.danger).add_modifier(Modifier::BOLD) } else { Style::default().fg(THEME.danger) }),
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