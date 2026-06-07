use crate::app::ui::theme::THEME;
use crate::app::{App, NavView, SidebarFocus};
use crate::tr;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub const NAV_ITEMS: [NavView; 6] = [
    NavView::Main,
    NavView::TrendGraphs,
    NavView::Storage,
    NavView::LibraryInspection,
    NavView::Containers,
    NavView::Agents,
];

pub fn nav_item_area(inner_area: Rect, index: usize, item_count: usize) -> Rect {
    if item_count == 0 || index >= item_count {
        return Rect {
            x: inner_area.x,
            y: inner_area.y,
            width: inner_area.width,
            height: 0,
        };
    }

    let item_count = item_count as u16;
    let item_height: u16 = if inner_area.height >= item_count.saturating_mul(3) {
        3
    } else {
        1
    };
    let used_height = item_height.saturating_mul(item_count);
    let gap_count = item_count.saturating_sub(1);
    let available_gap = inner_area.height.saturating_sub(used_height);
    let gap_height = available_gap.checked_div(gap_count).unwrap_or(0);
    let remainder = if gap_count > 0 {
        available_gap % gap_count
    } else {
        0
    };
    let top_padding = remainder / 2;
    let index = index as u16;
    let y = inner_area.y
        + top_padding
        + index.saturating_mul(item_height.saturating_add(gap_height))
        + index.min(remainder % gap_count.max(1));

    Rect {
        x: inner_area.x,
        y,
        width: inner_area.width,
        height: item_height.min(
            inner_area
                .y
                .saturating_add(inner_area.height)
                .saturating_sub(y),
        ),
    }
}

pub fn render_nav_sidebar(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let is_focused = app.ui.sidebar_focus == SidebarFocus::Nav;
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
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(border_type);

    f.render_widget(block.clone(), area);
    let inner_area = block.inner(area);

    let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    let storage_icon =
        if app.storage.search_progress_running && app.ui.current_nav_view != NavView::Storage {
            spinners[(app.ui.frame_count as usize) % spinners.len()]
        } else {
            "󰋊"
        };

    let libs_icon = if app.libraries.libraries_loading
        && app.ui.current_nav_view != NavView::LibraryInspection
    {
        spinners[(app.ui.frame_count as usize) % spinners.len()]
    } else {
        "󰅩"
    };

    let agents_icon =
        if app.agents.completed_notifications > 0 && app.ui.current_nav_view != NavView::Agents {
            "●"
        } else if app.agents.agents.iter().any(|a| {
            matches!(
                a.status,
                crate::app::types::AgentStatus::Running(_) | crate::app::types::AgentStatus::Queued
            )
        }) && app.ui.current_nav_view != NavView::Agents
        {
            spinners[(app.ui.frame_count as usize) % spinners.len()]
        } else {
            "󰚩"
        };

    let nav_items = vec![
        (NavView::Main, "󰞶", tr!(app.ui.translator, "nav.main")),
        (
            NavView::TrendGraphs,
            "󰄪",
            tr!(app.ui.translator, "nav.trends"),
        ),
        (
            NavView::Storage,
            storage_icon,
            tr!(app.ui.translator, "nav.storage"),
        ),
        (
            NavView::LibraryInspection,
            libs_icon,
            tr!(app.ui.translator, "nav.libs"),
        ),
        (
            NavView::Containers,
            "󰡨",
            tr!(app.ui.translator, "nav.containers"),
        ),
        (
            NavView::Agents,
            agents_icon,
            tr!(app.ui.translator, "nav.agents"),
        ),
    ];
    let nav_item_count = nav_items.len();
    for (i, (view, icon, name)) in nav_items.into_iter().enumerate() {
        let is_selected = app.ui.current_nav_view == view;
        let area = nav_item_area(inner_area, i, nav_item_count);

        if area.height == 0 {
            continue;
        }

        let style = if is_selected {
            Style::default()
                .fg(THEME.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(THEME.text_dim)
        };

        let block = if is_selected {
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(THEME.primary))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(THEME.background)) // Mantener estructura
        };

        let content = if app.ui.nav_sidebar_expanded {
            Paragraph::new(Line::from(vec![
                Span::styled(format!(" {} ", icon), style),
                Span::styled(name, style),
            ]))
        } else {
            Paragraph::new(Line::from(vec![Span::styled(icon, style)]))
                .alignment(ratatui::layout::Alignment::Center)
        };

        f.render_widget(content.block(block), area);
    }
}
