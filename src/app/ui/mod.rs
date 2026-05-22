use crate::app::{App, AppState, NavView};
use crate::config;
use crate::tr;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};
pub mod center_panel;
pub mod containers;
pub mod dialogs;
pub mod firewall;
pub mod footer;
pub mod header;
pub mod nav_sidebar;
pub mod sidebar_left;
pub mod sidebar_right;
pub mod theme;
pub mod trends;
pub mod widgets;
pub use center_panel::render_center_panel;
pub use containers::{
    render_container_console_modal, render_container_logs_modal, render_containers_view,
    render_docker_hub_modal,
};
pub use dialogs::render_confirmation_dialog;
pub use dialogs::render_install_dialog;
pub use dialogs::render_language_modal;
pub use dialogs::render_nerdfont_dialog;
pub use dialogs::render_password_modal;
pub use dialogs::render_update_dialog;
pub use dialogs::render_welcome_dialog;
pub use firewall::render_firewall_mode;
pub use footer::render_footer;
pub use header::render_header;
pub use nav_sidebar::render_nav_sidebar;
pub use sidebar_left::render_left_sidebar;
pub use sidebar_right::render_right_sidebar;
pub use theme::THEME;
pub use trends::render_trends_view;
pub fn render_ui(f: &mut ratatui::Frame, app: &App) {
    let search_bar_height = if app.search_mode { 3 } else { 0 };
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(search_bar_height),
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(3),
        ])
        .split(f.area());
    render_header(f, app, main_chunks[0]);
    if app.search_mode {
        render_search_bar(f, app, main_chunks[1]);
    }
    if app.firewall_mode {
        render_firewall_mode(f, app, main_chunks[2]);
    } else {
        render_main_layout_with_nav(f, app, main_chunks[2]);
    }
    let t = &app.translator;
    let hint_spans = if app.firewall_mode {
        vec![
            Span::styled(
                " Esc/Q ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "hint.exit")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " Tab ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.text_dim)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "hint.switch_panel")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " Enter ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}", tr!(t, "hint.execute")),
                Style::default().fg(THEME.text_dim),
            ),
        ]
    } else if app.search_mode {
        vec![
            Span::styled(
                " ESC ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "hint.close_search")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " Enter ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "hint.confirm_filter")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " Backspace ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.text_dim)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}", tr!(t, "hint.delete_char")),
                Style::default().fg(THEME.text_dim),
            ),
        ]
    } else if app.current_nav_view == NavView::Containers {
        vec![
            Span::styled(
                " R ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "containers.action_refresh")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " V ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "containers.action_logs")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " C ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "containers.action_console")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " L ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "actions.language")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " M ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {} ", tr!(t, "nav.menu")),
                Style::default().fg(THEME.text_dim),
            ),
        ]
    } else if app.current_nav_view == NavView::TrendGraphs {
        vec![
            Span::styled(
                " Q ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.text_dim)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "hint.quit")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " Tab ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.text_dim)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "hint.switch_panel")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " L ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "actions.language")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " M ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {} ", tr!(t, "nav.menu")),
                Style::default().fg(THEME.text_dim),
            ),
        ]
    } else {
        vec![
            Span::styled(
                " Q ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.text_dim)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "hint.quit")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " Tab ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.text_dim)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "hint.switch_panel")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " / ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.text_dim)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "hint.search")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " L ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {}  ", tr!(t, "actions.language")),
                Style::default().fg(THEME.text_dim),
            ),
            Span::styled(
                " M ",
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {} ", tr!(t, "nav.menu")),
                Style::default().fg(THEME.text_dim),
            ),
        ]
    };
    let help_hint = Paragraph::new(Line::from(hint_spans)).alignment(Alignment::Center);
    f.render_widget(help_hint, main_chunks[3]);
    render_footer(f, app, main_chunks[4]);
    if app.show_welcome_dialog {
        render_welcome_dialog(f, app);
    } else if app.show_language_modal {
        render_language_modal(f, app);
    } else if app.show_password_modal {
        render_password_modal(f, app);
    } else if app.show_nerdfont_dialog {
        render_nerdfont_dialog(f, app);
    } else if app.show_install_dialog {
        render_install_dialog(f, app);
    } else if app.show_confirmation {
        render_confirmation_dialog(f, app);
    } else if app.show_update_dialog {
        render_update_dialog(f, app);
    }
    if app.current_nav_view == NavView::Containers && app.show_container_logs_modal {
        render_container_logs_modal(f, app);
    }
    if app.current_nav_view == NavView::Containers && app.show_container_console_modal {
        render_container_console_modal(f, app);
    }
    if app.current_nav_view == NavView::Containers && app.show_docker_hub_modal {
        render_docker_hub_modal(f, app);
    }
}
fn render_search_bar(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let margin = (100 - config::SEARCH_BAR_PCT) / 2;
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(margin),
            Constraint::Percentage(config::SEARCH_BAR_PCT),
            Constraint::Percentage(margin),
        ])
        .split(area);
    let search_area = h_chunks[1];
    let count = app.get_filtered_apps().len();
    let cursor = if app.frame_count.is_multiple_of(2) {
        "█"
    } else {
        " "
    };
    let search_line = Line::from(vec![
        Span::styled(
            " 󰍉 SEARCH ",
            Style::default()
                .fg(THEME.background)
                .bg(THEME.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default()),
        Span::styled(
            &app.search_query,
            Style::default()
                .fg(THEME.text_main)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(cursor, Style::default().fg(THEME.primary)),
        Span::styled(
            format!("  ({})", tr!(app.translator, "search.matches", count)),
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
        .style(Style::default().bg(Color::Reset));
    let search_widget = Paragraph::new(search_line).block(block);
    f.render_widget(search_widget, search_area);
}
fn render_main_layout_with_nav(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let nav_width = if app.nav_sidebar_expanded { 22 } else { 7 };

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(nav_width), Constraint::Min(0)])
        .split(area);

    render_nav_sidebar(f, app, main_layout[0]);

    match app.current_nav_view {
        NavView::Main => match app.current_state {
            AppState::Dashboard => render_ide_layout(f, app, main_layout[1]),
        },
        NavView::TrendGraphs => render_trends_view(f, app, main_layout[1]),
        NavView::DgaDetector => render_placeholder_view(f, "Detector DGA", main_layout[1]),
        NavView::LibraryInspection => {
            render_placeholder_view(f, "Inspección de Librerías", main_layout[1])
        }
        NavView::Containers => render_containers_view(f, app, main_layout[1]),
    }
}

fn render_placeholder_view(f: &mut ratatui::Frame, title: &str, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(format!(" {} ", title))
        .border_style(Style::default().fg(THEME.primary));

    let paragraph = Paragraph::new(vec![
        Line::from(""),
        Line::from("  Próximamente..."),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Esta funcionalidad ("),
            Span::styled(
                title,
                Style::default()
                    .fg(THEME.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(") será implementada pronto."),
        ]),
    ])
    .block(block);

    f.render_widget(paragraph, area);
}

fn render_ide_layout(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(config::SIDEBAR_LEFT_PCT),
            Constraint::Percentage(config::CENTER_PANEL_PCT),
            Constraint::Percentage(config::SIDEBAR_RIGHT_PCT),
        ])
        .split(area);
    render_left_sidebar(f, app, columns[0]);
    render_center_panel(f, app, columns[1]);
    render_right_sidebar(f, app, columns[2]);
}
