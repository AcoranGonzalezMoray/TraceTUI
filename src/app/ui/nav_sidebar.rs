use crate::app::ui::theme::THEME;
use crate::app::{App, NavView, SidebarFocus};
use crate::tr;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

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
    let available = inner_area.height as usize;
    let num_items = nav_items.len();
    let is_compact = available < num_items * 4;

    let item_height: u16 = if !is_compact { 3 } else { 1 };

    let total_item_space = num_items as u16 * item_height;
    let mut constraints = Vec::new();

    if available as u16 > total_item_space {
        let extra = available as u16 - total_item_space;
        let gap = extra / (num_items + 1) as u16;

        constraints.push(Constraint::Length(gap));
        for _ in 0..num_items {
            constraints.push(Constraint::Length(item_height));
            constraints.push(Constraint::Length(gap));
        }
    } else {
        constraints.push(Constraint::Length(0));
        for _ in 0..num_items {
            constraints.push(Constraint::Min(item_height));
            constraints.push(Constraint::Length(0));
        }
    }

    let item_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    for (i, (view, icon, name)) in nav_items.into_iter().enumerate() {
        let is_selected = app.ui.current_nav_view == view;
        let area = item_chunks[i * 2 + 1];

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

        let block = if is_compact {
            if is_selected {
                Block::default()
                    .borders(Borders::LEFT)
                    .border_style(Style::default().fg(THEME.primary))
            } else {
                Block::default()
            }
        } else {
            if is_selected {
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(THEME.primary))
            } else {
                Block::default().padding(ratatui::widgets::Padding::new(1, 1, 1, 1))
            }
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
