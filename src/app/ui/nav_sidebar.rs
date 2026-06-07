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
    if item_count == 0 || index >= item_count || inner_area.height == 0 {
        return Rect {
            x: inner_area.x,
            y: inner_area.y,
            width: inner_area.width,
            height: 0,
        };
    }

    let count = item_count as u16;
    let index = index as u16;
    let preferred_height: u16 = if inner_area.height >= count.saturating_mul(3) {
        3
    } else {
        1
    };

    if inner_area.height < count.saturating_mul(preferred_height) {
        let base = inner_area.height / count;
        let extra = inner_area.height % count;
        let mut y = inner_area.y;
        for i in 0..index {
            y += base + u16::from(i < extra);
        }
        let height = base + u16::from(index < extra);
        return Rect {
            x: inner_area.x,
            y,
            width: inner_area.width,
            height,
        };
    }

    let total_items_height = preferred_height.saturating_mul(count);
    let extra = inner_area.height.saturating_sub(total_items_height);
    let gap_count = count.saturating_sub(1);
    let (base_gap, extra_gaps) = match extra.checked_div(gap_count) {
        Some(base_gap) => (base_gap, extra % gap_count),
        None => (0, 0),
    };
    let mut y = inner_area.y;
    for i in 0..index {
        y += preferred_height;
        y += base_gap + u16::from(i < extra_gaps);
    }

    let height = preferred_height.min(
        inner_area
            .y
            .saturating_add(inner_area.height)
            .saturating_sub(y),
    );

    Rect {
        x: inner_area.x,
        y,
        width: inner_area.width,
        height,
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
    let compact = inner_area.height < nav_item_count as u16 * 3;
    for (i, (view, icon, name)) in nav_items.into_iter().enumerate() {
        let is_selected = app.ui.current_nav_view == view;
        let area = nav_item_area(inner_area, i, nav_item_count);

        if area.height == 0 {
            continue;
        }

        let style = if is_selected {
            if compact {
                Style::default()
                    .fg(THEME.background)
                    .bg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(THEME.primary)
                    .add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default().fg(THEME.text_dim)
        };

        let content = if app.ui.nav_sidebar_expanded && area.height >= 2 {
            Paragraph::new(Line::from(vec![
                Span::styled(format!(" {} ", icon), style),
                Span::styled(name, style),
            ]))
        } else {
            Paragraph::new(Line::from(vec![Span::styled(icon, style)]))
                .alignment(ratatui::layout::Alignment::Center)
        };

        if compact || area.height < 3 {
            f.render_widget(content, area);
        } else {
            let block = if is_selected {
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(THEME.primary))
            } else {
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(THEME.background))
            };
            f.render_widget(content.block(block), area);
        }
    }
}
