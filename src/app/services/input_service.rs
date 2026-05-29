use crate::app::firewall_service::FirewallManager;
use crate::app::storage::fmt_size;
use crate::app::types::{ConfirmationAction, FirewallPanel, NavView, SidebarFocus};
use crate::app::App;
use crate::config;
use crate::resources;
use crate::tr;
use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

fn switch_nav_view(app: &mut App, view: NavView) {
    if app.ui.current_nav_view == view {
        return;
    }
    app.ui.current_nav_view = view;
    app.ui.needs_clear = true;
    app.containers.show_container_logs_modal = false;
    app.containers.show_container_console_modal = false;
    app.ui.search_mode = false;
    app.ui.continuous_refresh_counter = 0;
    if view == NavView::Main || view == NavView::TrendGraphs {
        app.ui.selected_action_index = 0;
        app.ui.status_message = tr!(app.ui.translator, "status.analysis_resumed").to_string();
        app.ui.analysis_paused = false;
    } else {
        app.ui.status_message = tr!(app.ui.translator, "status.section_changed").to_string();
        app.ui.analysis_paused = true;
    }
    if view == NavView::Storage {
        if app.storage.disks.is_empty() {
            refresh_disks(app);
        }
        if !app.storage.file_search_mode && !app.storage.search_progress_running {
            if let Some(disk) = app.get_selected_disk() {
                let p = std::path::Path::new(&disk.mount_point);
                if p.exists() {
                    app.storage.current_directory = p.to_path_buf();
                    app.storage.file_scroll = 0;
                    load_directory(app);
                }
            }
        }
    }
    if view == NavView::LibraryInspection {
        app.ui.selected_action_index = 0;
        if app.ui.auto_analysis_complete {
            app.refresh_libraries();
        }
    }
}

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }
    if app.ui.show_welcome_dialog {
        handle_welcome_keys(app, key);
        return;
    }
    if app.ui.show_language_modal {
        handle_language_keys(app, key);
        return;
    }
    if app.install.show_password_modal {
        handle_password_keys(app, key);
        return;
    }
    if app.nerdfont.show_dialog {
        handle_nerdfont_dialog_keys(app, key);
        return;
    }
    if app.install.show_dialog {
        handle_install_dialog_keys(app, key);
        return;
    }
    if app.firewall.firewall_mode {
        handle_firewall_keys(app, key);
        return;
    }
    if app.ui.show_confirmation {
        handle_confirmation_keys(app, key);
        return;
    }
    if app.update.show_update_dialog {
        handle_update_dialog_keys(app, key);
        return;
    }
    if app.ui.search_mode {
        handle_search_keys(app, key);
        return;
    }
    if app.ui.current_nav_view == NavView::Containers && app.containers.show_docker_hub_modal {
        handle_docker_hub_keys(app, key);
        return;
    }
    if app.ui.current_nav_view == NavView::Containers {
        handle_container_keys(app, key);
        return;
    }
    if app.ui.current_nav_view == NavView::Storage && app.storage.show_file_viewer {
        handle_file_viewer_keys(app, key);
        return;
    }
    if app.ui.current_nav_view == NavView::LibraryInspection && app.libraries.show_hash_info_modal {
        handle_library_hash_modal_keys(app, key);
        return;
    }
    if app.ui.current_nav_view == NavView::LibraryInspection
        && app.libraries.show_library_binary_viewer
    {
        handle_library_binary_viewer_keys(app, key);
        return;
    }
    if app.ui.current_nav_view == NavView::Storage {
        handle_storage_keys(app, key);
        return;
    }
    if app.ui.current_nav_view == NavView::LibraryInspection {
        handle_libraries_keys(app, key);
        return;
    }
    if app.ui.show_map {
        if key.code == KeyCode::Esc
            || key.code == KeyCode::Char('q')
            || key.code == KeyCode::Char('Q')
        {
            app.ui.show_map = false;
            app.ui.selected_action_index = 0;
        }
        return;
    }
    handle_dashboard_keys(app, key);
}
fn handle_dashboard_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Tab => {
            app.ui.sidebar_focus = match app.ui.sidebar_focus {
                SidebarFocus::Nav => SidebarFocus::Left,
                SidebarFocus::Left => SidebarFocus::Center,
                SidebarFocus::Center => SidebarFocus::Right,
                SidebarFocus::Right => SidebarFocus::Nav,
            };
            app.ui.status_message = tr!(
                app.ui.translator,
                "status.focus",
                format!("{:?}", app.ui.sidebar_focus)
            )
            .to_string();
        }
        KeyCode::BackTab => {
            app.ui.sidebar_focus = match app.ui.sidebar_focus {
                SidebarFocus::Nav => SidebarFocus::Right,
                SidebarFocus::Left => SidebarFocus::Nav,
                SidebarFocus::Center => SidebarFocus::Left,
                SidebarFocus::Right => SidebarFocus::Center,
            };
            app.ui.status_message = tr!(
                app.ui.translator,
                "status.focus",
                format!("{:?}", app.ui.sidebar_focus)
            )
            .to_string();
        }
        KeyCode::Up => {
            let in_investigation = app.investigation.investigation_report.is_some()
                || app.investigation.is_investigating;
            match app.ui.sidebar_focus {
                SidebarFocus::Nav => {
                    let next = match app.ui.current_nav_view {
                        NavView::Main => NavView::Containers,
                        NavView::TrendGraphs => NavView::Main,
                        NavView::Storage => NavView::TrendGraphs,
                        NavView::LibraryInspection => NavView::Storage,
                        NavView::Containers => NavView::LibraryInspection,
                    };
                    switch_nav_view(app, next);
                }
                SidebarFocus::Left if !in_investigation && app.network.selected_app_index > 0 => {
                    app.network.selected_app_index -= 1;
                    app.network.selected_connection_index = 0;
                    app.trigger_geo_lookup_for_selected_app();
                }
                SidebarFocus::Center
                    if !in_investigation && app.network.selected_connection_index > 0 =>
                {
                    app.network.selected_connection_index -= 1;
                }
                SidebarFocus::Right if app.ui.selected_action_index > 0 => {
                    app.ui.selected_action_index -= 1;
                }
                _ => {}
            }
        }
        KeyCode::Down => {
            let in_investigation = app.investigation.investigation_report.is_some()
                || app.investigation.is_investigating;
            match app.ui.sidebar_focus {
                SidebarFocus::Nav => {
                    let next = match app.ui.current_nav_view {
                        NavView::Main => NavView::TrendGraphs,
                        NavView::TrendGraphs => NavView::Storage,
                        NavView::Storage => NavView::LibraryInspection,
                        NavView::LibraryInspection => NavView::Containers,
                        NavView::Containers => NavView::Main,
                    };
                    switch_nav_view(app, next);
                }
                SidebarFocus::Left if !in_investigation => {
                    let filtered_count = app.get_filtered_apps().len();
                    if app.network.selected_app_index < filtered_count.saturating_sub(1) {
                        app.network.selected_app_index += 1;
                        app.network.selected_connection_index = 0;
                        app.trigger_geo_lookup_for_selected_app();
                    }
                }
                SidebarFocus::Center if !in_investigation => {
                    if let Some(selected) = app.get_selected_app() {
                        if app.network.selected_connection_index
                            < selected.connections.len().saturating_sub(1)
                        {
                            app.network.selected_connection_index += 1;
                        }
                    }
                }
                SidebarFocus::Right if app.ui.selected_action_index < config::ACTION_COUNT => {
                    app.ui.selected_action_index += 1;
                }
                _ => {}
            }
        }
        KeyCode::Enter => {
            let in_investigation = app.investigation.investigation_report.is_some()
                || app.investigation.is_investigating;
            match app.ui.sidebar_focus {
                SidebarFocus::Nav => {
                    app.ui.nav_sidebar_expanded = !app.ui.nav_sidebar_expanded;
                    app.ui.sidebar_focus = SidebarFocus::Nav;
                }
                SidebarFocus::Right => execute_action(app),
                SidebarFocus::Center if !in_investigation => app.start_investigation(),
                SidebarFocus::Left => {
                    app.ui.sidebar_focus = SidebarFocus::Center;
                    app.network.selected_connection_index = 0;
                }
                _ => {}
            }
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.ui.status_message = tr!(app.ui.translator, "status.refresh").to_string();
            app.start_batch_analysis();
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.ui.analysis_paused = !app.ui.analysis_paused;
            app.ui.continuous_refresh_counter = 0;
            if app.ui.analysis_paused {
                app.ui.status_message =
                    tr!(app.ui.translator, "status.analysis_paused").to_string();
            } else {
                app.ui.status_message =
                    tr!(app.ui.translator, "status.analysis_resumed").to_string();
                app.start_batch_analysis();
            }
        }
        KeyCode::Char('b') | KeyCode::Char('B') => {
            enter_firewall_mode(app);
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.ui.nav_sidebar_expanded = !app.ui.nav_sidebar_expanded;
            app.ui.sidebar_focus = SidebarFocus::Nav;
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.ui.show_language_modal = true;
            if let Some(idx) = crate::i18n::Translator::available_locales()
                .iter()
                .position(|(code, _)| *code == app.ui.translator.locale)
            {
                app.ui.language_selection_index = idx;
            } else {
                app.ui.language_selection_index = 0;
            }
        }
        KeyCode::Char('1') => {
            app.ui.center_tab = 0;
            app.ui.status_message = tr!(app.ui.translator, "status.tab_connections").to_string();
        }
        KeyCode::Char('2') if app.get_selected_app().is_some() => {
            app.ui.center_tab = 1;
            app.ui.status_message = tr!(app.ui.translator, "status.tab_risk").to_string();
        }
        KeyCode::Char('3') => {
            app.ui.center_tab = 2;
            app.ui.status_message = tr!(app.ui.translator, "status.tab_timeline").to_string();
        }
        KeyCode::Char('/') => {
            app.ui.search_mode = true;
            app.ui.search_query.clear();
            app.network.selected_app_index = 0;
            app.ui.status_message = tr!(app.ui.translator, "status.search_active").to_string();
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.ui.hunter_mode = !app.ui.hunter_mode;
            app.network.selected_app_index = 0;
            if app.ui.hunter_mode {
                app.ui.status_message = tr!(app.ui.translator, "status.hunter_on").to_string();
            } else {
                app.ui.status_message = tr!(app.ui.translator, "status.hunter_off").to_string();
            }
            app.start_batch_analysis();
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            app.ui.filter_high_risk_only = !app.ui.filter_high_risk_only;
            app.network.selected_app_index = 0;
            if app.ui.filter_high_risk_only {
                app.ui.status_message =
                    tr!(app.ui.translator, "status.filter_high_risk").to_string();
            } else {
                app.ui.status_message = tr!(app.ui.translator, "status.filter_all").to_string();
            }
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            if let Some(selected) = app.get_selected_app() {
                app.ui.confirmation_message = tr!(
                    app.ui.translator,
                    "dialog.kill_process",
                    &selected.process_name,
                    selected.pid
                );
                app.ui.pending_confirmation_action = Some(ConfirmationAction::KillProcess);
                app.ui.show_confirmation = true;
            } else {
                app.ui.status_message = tr!(app.ui.translator, "status.no_selection").to_string();
            }
        }
        KeyCode::Char('-') => {
            if let Some(selected) = app.get_selected_app() {
                app.ui.confirmation_message = tr!(
                    app.ui.translator,
                    "dialog.kill_conns",
                    selected.connections.len(),
                    &selected.process_name
                );
                app.ui.pending_confirmation_action = Some(ConfirmationAction::KillAllConnections);
                app.ui.show_confirmation = true;
            } else {
                app.ui.status_message = tr!(app.ui.translator, "status.no_selection").to_string();
            }
        }
        KeyCode::Char('g') | KeyCode::Char('G') => {
            if let Some(selected) = app.get_selected_app() {
                let search_url = format!(
                    "{}{}",
                    resources::URLS.google_search_url,
                    urlencoding::encode(&selected.process_name)
                );
                if let Err(e) = open::that(&search_url) {
                    app.ui.status_message = tr!(app.ui.translator, "status.browser_fail", e);
                } else {
                    app.ui.status_message = tr!(
                        app.ui.translator,
                        "status.searching_online",
                        &selected.process_name
                    );
                }
            } else {
                app.ui.status_message = tr!(app.ui.translator, "status.no_selection").to_string();
            }
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                app.ui.should_quit = true;
            } else if let Some(selected) = app.get_selected_app() {
                let path = selected.process_path.clone();
                match arboard::Clipboard::new() {
                    Ok(mut clipboard) => match clipboard.set_text(&path) {
                        Ok(_) => {
                            app.ui.status_message = tr!(app.ui.translator, "status.copied", path);
                        }
                        Err(e) => {
                            app.ui.status_message =
                                tr!(app.ui.translator, "status.clipboard_fail", e);
                        }
                    },
                    Err(e) => {
                        app.ui.status_message =
                            tr!(app.ui.translator, "status.clipboard_unavail", e);
                    }
                }
            } else {
                app.ui.status_message = tr!(app.ui.translator, "status.no_selection").to_string();
            }
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            export_to_json(app);
        }
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            if app.investigation.investigation_report.is_some()
                || app.investigation.is_investigating
            {
                app.investigation.investigation_report = None;
                app.investigation.is_investigating = false;
                app.ui.analysis_paused = false;
            } else if !app.ui.search_query.is_empty() {
                app.ui.search_query.clear();
                app.network.selected_app_index = 0;
                app.ui.status_message = tr!(app.ui.translator, "status.filter_cleared").to_string();
            } else {
                app.ui.should_quit = true;
            }
        }
        _ => {}
    }
}
fn handle_container_keys(app: &mut App, key: KeyEvent) {
    if app.containers.show_container_console_modal {
        handle_container_console_keys(app, key);
        return;
    }
    if app.containers.show_container_logs_modal {
        handle_container_logs_keys(app, key);
        return;
    }

    match key.code {
        KeyCode::Tab => {
            app.ui.sidebar_focus = match app.ui.sidebar_focus {
                SidebarFocus::Nav => SidebarFocus::Left,
                SidebarFocus::Left => SidebarFocus::Center,
                SidebarFocus::Center => SidebarFocus::Right,
                SidebarFocus::Right => SidebarFocus::Nav,
            };
            app.ui.status_message = tr!(
                app.ui.translator,
                "status.focus",
                format!("{:?}", app.ui.sidebar_focus)
            )
            .to_string();
        }
        KeyCode::BackTab => {
            app.ui.sidebar_focus = match app.ui.sidebar_focus {
                SidebarFocus::Nav => SidebarFocus::Right,
                SidebarFocus::Left => SidebarFocus::Nav,
                SidebarFocus::Center => SidebarFocus::Left,
                SidebarFocus::Right => SidebarFocus::Center,
            };
        }
        KeyCode::Up => match app.ui.sidebar_focus {
            SidebarFocus::Nav => {
                switch_nav_view(app, NavView::LibraryInspection);
            }
            SidebarFocus::Left if app.containers.selected_container_index > 0 => {
                app.containers.selected_container_index -= 1;
                app.containers.container_detail_scroll = 0;
                app.containers.container_logs.clear();
            }
            SidebarFocus::Center if app.containers.container_detail_scroll > 0 => {
                app.containers.container_detail_scroll -= 1;
            }
            SidebarFocus::Right if app.containers.selected_container_action_index > 0 => {
                app.containers.selected_container_action_index -= 1;
            }
            _ => {}
        },
        KeyCode::Down => match app.ui.sidebar_focus {
            SidebarFocus::Nav => {
                switch_nav_view(app, NavView::Main);
            }
            SidebarFocus::Left => {
                let max = app.containers.containers.len().saturating_sub(1);
                if app.containers.selected_container_index < max {
                    app.containers.selected_container_index += 1;
                    app.containers.container_detail_scroll = 0;
                    app.containers.container_logs.clear();
                }
            }
            SidebarFocus::Center => {
                let max = app.containers.container_logs.len().saturating_sub(1);
                if app.containers.container_detail_scroll < max {
                    app.containers.container_detail_scroll += 1;
                }
            }
            SidebarFocus::Right => {
                let max = crate::app::containers::CONTAINER_RIGHT_ACTION_COUNT.saturating_sub(1);
                if app.containers.selected_container_action_index < max {
                    app.containers.selected_container_action_index += 1;
                }
            }
        },
        KeyCode::Enter => match app.ui.sidebar_focus {
            SidebarFocus::Nav => {
                app.ui.nav_sidebar_expanded = !app.ui.nav_sidebar_expanded;
                app.ui.sidebar_focus = SidebarFocus::Nav;
            }
            SidebarFocus::Left => app.ui.sidebar_focus = SidebarFocus::Center,
            SidebarFocus::Right => app.execute_container_right_action(),
            _ => app.refresh_selected_container_logs_async(),
        },
        KeyCode::Char('r') | KeyCode::Char('R') => app.refresh_containers_async(),
        KeyCode::Char('v') | KeyCode::Char('V') => app.refresh_selected_container_logs_async(),
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.ui.show_language_modal = !app.ui.show_language_modal;
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.containers.show_docker_hub_modal = !app.containers.show_docker_hub_modal;
            if !app.containers.show_docker_hub_modal {
                app.containers.docker_hub_search =
                    crate::app::containers::DockerHubSearchState::default();
            }
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.containers.selected_container_action_index = 3;
            app.execute_container_right_action();
        }
        KeyCode::Char('t') | KeyCode::Char('T') => {
            app.containers.selected_container_action_index = 4;
            app.execute_container_right_action();
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            app.containers.selected_container_action_index = 5;
            app.execute_container_right_action();
        }
        KeyCode::Char('p') | KeyCode::Char('P') => {
            app.containers.selected_container_action_index = 6;
            app.execute_container_right_action();
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            app.containers.selected_container_action_index = 2;
            app.execute_container_right_action();
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.containers.selected_container_action_index =
                crate::app::containers::DOCKER_ACTION_OFFSET;
            app.execute_container_right_action();
        }
        KeyCode::Char('o') | KeyCode::Char('O') => {
            app.containers.selected_container_action_index =
                crate::app::containers::DOCKER_ACTION_OFFSET + 1;
            app.execute_container_right_action();
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.ui.nav_sidebar_expanded = !app.ui.nav_sidebar_expanded;
            app.ui.sidebar_focus = SidebarFocus::Nav;
        }
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            switch_nav_view(app, NavView::Main);
            app.ui.sidebar_focus = SidebarFocus::Nav;
        }
        _ => {}
    }
}

fn handle_container_logs_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.containers.show_container_logs_modal = false;
        }
        KeyCode::Up => {
            app.containers.container_logs_scroll =
                app.containers.container_logs_scroll.saturating_sub(1);
        }
        KeyCode::Down => {
            let max = app.containers.container_logs.len().saturating_sub(1);
            app.containers.container_logs_scroll = app
                .containers
                .container_logs_scroll
                .saturating_add(1)
                .min(max);
        }
        KeyCode::PageUp => {
            app.containers.container_logs_scroll =
                app.containers.container_logs_scroll.saturating_sub(10);
        }
        KeyCode::PageDown => {
            let max = app.containers.container_logs.len().saturating_sub(1);
            app.containers.container_logs_scroll = app
                .containers
                .container_logs_scroll
                .saturating_add(10)
                .min(max);
        }
        KeyCode::End => {
            app.containers.container_logs_scroll =
                app.containers.container_logs.len().saturating_sub(1);
        }
        KeyCode::Char('r') | KeyCode::Char('R') => app.refresh_selected_container_logs_async(),
        _ => {}
    }
}

fn handle_container_console_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.containers.show_container_console_modal = false;
            app.containers.container_console_input.clear();
        }
        KeyCode::Enter => app.execute_container_console_command_async(),
        KeyCode::Backspace => {
            app.containers.container_console_input.pop();
        }
        KeyCode::Up => {
            app.containers.container_console_scroll =
                app.containers.container_console_scroll.saturating_sub(1);
        }
        KeyCode::Down => {
            let max = app
                .containers
                .container_console_output
                .len()
                .saturating_sub(1);
            app.containers.container_console_scroll = app
                .containers
                .container_console_scroll
                .saturating_add(1)
                .min(max);
        }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.containers.container_console_input.push(c);
        }
        _ => {}
    }
}

fn handle_docker_hub_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.containers.show_docker_hub_modal = false;
            app.containers.docker_hub_search =
                crate::app::containers::DockerHubSearchState::default();
        }
        KeyCode::Tab => {
            app.containers.docker_hub_search.focused_field =
                (app.containers.docker_hub_search.focused_field + 1)
                    % config::DOCKER_HUB_FIELD_COUNT;
        }
        KeyCode::BackTab => {
            app.containers.docker_hub_search.focused_field =
                if app.containers.docker_hub_search.focused_field == 0 {
                    config::DOCKER_HUB_FIELD_COUNT - 1
                } else {
                    app.containers.docker_hub_search.focused_field - 1
                };
        }
        KeyCode::Up | KeyCode::Down
            if app.containers.docker_hub_search.focused_field == 0
                && !app.containers.docker_hub_search.results.is_empty() =>
        {
            match key.code {
                KeyCode::Up => {
                    app.containers.docker_hub_search.selected_result_index = app
                        .containers
                        .docker_hub_search
                        .selected_result_index
                        .saturating_sub(1);
                }
                KeyCode::Down => {
                    let max = app
                        .containers
                        .docker_hub_search
                        .results
                        .len()
                        .saturating_sub(1);
                    if app.containers.docker_hub_search.selected_result_index < max {
                        app.containers.docker_hub_search.selected_result_index += 1;
                    }
                }
                _ => {}
            }
        }
        KeyCode::Char(c)
            if app.containers.docker_hub_search.focused_field == 0
                && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.containers.docker_hub_search.search_query.push(c);
        }
        KeyCode::Backspace if app.containers.docker_hub_search.focused_field == 0 => {
            app.containers.docker_hub_search.search_query.pop();
        }
        KeyCode::Enter if app.containers.docker_hub_search.focused_field == 0 => {
            start_docker_hub_search_async(app);
        }
        KeyCode::Char(c)
            if app.containers.docker_hub_search.focused_field == 1
                && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.containers.docker_hub_search.container_name.push(c);
        }
        KeyCode::Backspace if app.containers.docker_hub_search.focused_field == 1 => {
            app.containers.docker_hub_search.container_name.pop();
        }
        KeyCode::Char(c)
            if app.containers.docker_hub_search.focused_field == 2
                && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.containers.docker_hub_search.ports.push(c);
        }
        KeyCode::Backspace if app.containers.docker_hub_search.focused_field == 2 => {
            app.containers.docker_hub_search.ports.pop();
        }
        KeyCode::Char(c)
            if app.containers.docker_hub_search.focused_field == 3
                && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.containers.docker_hub_search.env_vars.push(c);
        }
        KeyCode::Backspace if app.containers.docker_hub_search.focused_field == 3 => {
            app.containers.docker_hub_search.env_vars.pop();
        }
        KeyCode::Enter if app.containers.docker_hub_search.focused_field == 4 => {
            if !app.containers.docker_hub_search.results.is_empty()
                && app.containers.docker_hub_search.selected_result_index
                    < app.containers.docker_hub_search.results.len()
            {
                let image = app.containers.docker_hub_search.results
                    [app.containers.docker_hub_search.selected_result_index]
                    .name
                    .clone();
                let name = if app.containers.docker_hub_search.container_name.is_empty() {
                    image.clone()
                } else {
                    app.containers.docker_hub_search.container_name.clone()
                };
                let ports = app.containers.docker_hub_search.ports.clone();
                let env_vars = app.containers.docker_hub_search.env_vars.clone();
                start_create_container_async(app, &image, &name, &ports, &env_vars);
            } else {
                app.ui.status_message =
                    tr!(app.ui.translator, "containers.docker_hub_no_results").to_string();
            }
        }
        KeyCode::Enter if app.containers.docker_hub_search.focused_field == 5 => {
            app.containers.show_docker_hub_modal = false;
            app.containers.docker_hub_search =
                crate::app::containers::DockerHubSearchState::default();
        }
        _ => {}
    }
}

fn start_docker_hub_search_async(app: &mut App) {
    if app.containers.docker_hub_search.search_query.is_empty() {
        app.ui.status_message = tr!(
            app.ui.translator,
            "containers.docker_hub_search_placeholder"
        )
        .to_string();
        return;
    }

    let query = app.containers.docker_hub_search.search_query.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    app.ui.status_message = tr!(app.ui.translator, "containers.docker_hub_searching").to_string();

    std::thread::spawn(move || {
        let _ = tx.send(crate::app::containers::ContainerManager::search_docker_hub(
            &query,
        ));
    });

    app.containers.docker_hub_search_rx = Some(rx);
}

fn start_create_container_async(
    app: &mut App,
    image: &str,
    name: &str,
    ports: &str,
    env_vars: &str,
) {
    let image = image.to_string();
    let name = name.to_string();
    let ports = ports.to_string();
    let env_vars = env_vars.to_string();

    let (tx, rx) = std::sync::mpsc::channel();
    app.ui.status_message = tr!(app.ui.translator, "containers.docker_hub_creating").to_string();

    std::thread::spawn(move || {
        let _ = tx.send(crate::app::containers::ContainerManager::create_and_run(
            &image, &name, &ports, &env_vars,
        ));
    });

    app.containers.docker_hub_create_rx = Some(rx);
}

fn handle_library_hash_modal_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            app.rehash_suspicious_libraries();
            app.ui.status_message = tr!(app.ui.translator, "libraries.hash_computed").to_string();
            app.libraries.show_hash_info_modal = false;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.libraries.show_hash_info_modal = false;
        }
        _ => {}
    }
}
fn handle_library_binary_viewer_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.libraries.library_binary_scroll =
                app.libraries.library_binary_scroll.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let total = if app.libraries.library_binary_tab == 0 {
                app.libraries.library_binary_hex_lines.len()
            } else {
                app.libraries.library_binary_disasm_lines.len()
            };
            app.libraries.library_binary_scroll = app
                .libraries
                .library_binary_scroll
                .saturating_add(1)
                .min(total.saturating_sub(1));
        }
        KeyCode::PageUp => {
            app.libraries.library_binary_scroll =
                app.libraries.library_binary_scroll.saturating_sub(20);
        }
        KeyCode::PageDown => {
            let total = if app.libraries.library_binary_tab == 0 {
                app.libraries.library_binary_hex_lines.len()
            } else {
                app.libraries.library_binary_disasm_lines.len()
            };
            app.libraries.library_binary_scroll = app
                .libraries
                .library_binary_scroll
                .saturating_add(20)
                .min(total.saturating_sub(1));
        }
        KeyCode::Tab => {
            app.libraries.library_binary_tab = (app.libraries.library_binary_tab + 1) % 2;
            app.libraries.library_binary_scroll = 0;
        }
        KeyCode::BackTab => {
            app.libraries.library_binary_tab =
                app.libraries.library_binary_tab.saturating_sub(1) % 2;
            app.libraries.library_binary_scroll = 0;
        }
        KeyCode::Esc => {
            app.libraries.show_library_binary_viewer = false;
        }
        _ => {}
    }
}
fn handle_libraries_keys(app: &mut App, key: KeyEvent) {
    if app.libraries.library_search_active {
        match key.code {
            KeyCode::Esc => {
                app.libraries.library_search_active = false;
            }
            KeyCode::Enter => {
                app.libraries.library_search_active = false;
            }
            KeyCode::Backspace => {
                app.libraries.library_search_query.pop();
                app.libraries.selected_library_index = 0;
                app.libraries.library_lib_scroll = 0;
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.libraries.library_search_query.push(c);
                app.libraries.selected_library_index = 0;
                app.libraries.library_lib_scroll = 0;
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Tab => {
            app.ui.sidebar_focus = match app.ui.sidebar_focus {
                SidebarFocus::Nav => SidebarFocus::Left,
                SidebarFocus::Left => SidebarFocus::Center,
                SidebarFocus::Center => SidebarFocus::Right,
                SidebarFocus::Right => SidebarFocus::Nav,
            };
            app.ui.status_message = format!("Library focus: {:?}", app.ui.sidebar_focus);
        }
        KeyCode::BackTab => {
            app.ui.sidebar_focus = match app.ui.sidebar_focus {
                SidebarFocus::Nav => SidebarFocus::Right,
                SidebarFocus::Left => SidebarFocus::Nav,
                SidebarFocus::Center => SidebarFocus::Left,
                SidebarFocus::Right => SidebarFocus::Center,
            };
        }

        KeyCode::Up if app.ui.sidebar_focus == SidebarFocus::Nav => {
            let next = match app.ui.current_nav_view {
                NavView::Main => NavView::Containers,
                NavView::TrendGraphs => NavView::Main,
                NavView::Storage => NavView::TrendGraphs,
                NavView::LibraryInspection => NavView::Storage,
                NavView::Containers => NavView::LibraryInspection,
            };
            switch_nav_view(app, next);
        }
        KeyCode::Down if app.ui.sidebar_focus == SidebarFocus::Nav => {
            let next = match app.ui.current_nav_view {
                NavView::Main => NavView::TrendGraphs,
                NavView::TrendGraphs => NavView::Storage,
                NavView::Storage => NavView::LibraryInspection,
                NavView::LibraryInspection => NavView::Containers,
                NavView::Containers => NavView::Main,
            };
            switch_nav_view(app, next);
        }
        KeyCode::Up if app.ui.sidebar_focus == SidebarFocus::Left => {
            if app.libraries.selected_library_process_index > 0 {
                app.libraries.selected_library_process_index -= 1;
                app.libraries.selected_library_index = 0;
                app.libraries.library_lib_scroll = 0;
                app.libraries.library_process_scroll =
                    app.libraries.library_process_scroll.saturating_sub(0);
            }

            if app.libraries.selected_library_process_index < app.libraries.library_process_scroll {
                app.libraries.library_process_scroll = app.libraries.selected_library_process_index;
            }
        }
        KeyCode::Down if app.ui.sidebar_focus == SidebarFocus::Left => {
            let groups = app.group_libs_by_process();
            if app.libraries.selected_library_process_index + 1 < groups.len() {
                app.libraries.selected_library_process_index += 1;
                app.libraries.selected_library_index = 0;
                app.libraries.library_lib_scroll = 0;
            }
        }
        KeyCode::PageUp if app.ui.sidebar_focus == SidebarFocus::Left => {
            app.libraries.selected_library_process_index = app
                .libraries
                .selected_library_process_index
                .saturating_sub(10);
            app.libraries.selected_library_index = 0;
            app.libraries.library_lib_scroll = 0;
        }
        KeyCode::PageDown if app.ui.sidebar_focus == SidebarFocus::Left => {
            let max = app.group_libs_by_process().len().saturating_sub(1);
            app.libraries.selected_library_process_index =
                (app.libraries.selected_library_process_index + 10).min(max);
            app.libraries.selected_library_index = 0;
            app.libraries.library_lib_scroll = 0;
        }

        KeyCode::Up
            if app.ui.sidebar_focus == SidebarFocus::Center
                && app.libraries.selected_library_index > 0 =>
        {
            app.libraries.selected_library_index -= 1;
            if app.libraries.selected_library_index < app.libraries.library_lib_scroll {
                app.libraries.library_lib_scroll = app.libraries.selected_library_index;
            }
        }
        KeyCode::Down if app.ui.sidebar_focus == SidebarFocus::Center => {
            let libs = crate::app::ui::libraries::get_libs_for_selected_process(app);
            let max = libs.len().saturating_sub(1);
            if app.libraries.selected_library_index < max {
                app.libraries.selected_library_index += 1;
            }
        }
        KeyCode::PageUp if app.ui.sidebar_focus == SidebarFocus::Center => {
            app.libraries.selected_library_index =
                app.libraries.selected_library_index.saturating_sub(10);
            app.libraries.library_lib_scroll = app.libraries.library_lib_scroll.saturating_sub(10);
        }
        KeyCode::PageDown if app.ui.sidebar_focus == SidebarFocus::Center => {
            let libs = crate::app::ui::libraries::get_libs_for_selected_process(app);
            let max = libs.len().saturating_sub(1);
            app.libraries.selected_library_index =
                (app.libraries.selected_library_index + 10).min(max);
        }
        KeyCode::Home if app.ui.sidebar_focus == SidebarFocus::Center => {
            app.libraries.selected_library_index = 0;
            app.libraries.library_lib_scroll = 0;
        }
        KeyCode::End if app.ui.sidebar_focus == SidebarFocus::Center => {
            let libs = crate::app::ui::libraries::get_libs_for_selected_process(app);
            app.libraries.selected_library_index = libs.len().saturating_sub(1);
        }

        KeyCode::Up
            if app.ui.sidebar_focus == SidebarFocus::Right && app.ui.selected_action_index > 0 =>
        {
            app.ui.selected_action_index -= 1;
        }
        KeyCode::Down
            if app.ui.sidebar_focus == SidebarFocus::Right
                && app.ui.selected_action_index + 1
                    < crate::app::libraries::LIBRARY_ACTION_COUNT =>
        {
            app.ui.selected_action_index += 1;
        }
        KeyCode::Enter if app.ui.sidebar_focus == SidebarFocus::Right => {
            execute_library_action(app);
        }

        KeyCode::Enter if app.ui.sidebar_focus == SidebarFocus::Center => {
            let libs = crate::app::ui::libraries::get_libs_for_selected_process(app);
            if let Some(lib) = libs.get(app.libraries.selected_library_index) {
                let path = lib.path.clone();
                match arboard::Clipboard::new() {
                    Ok(mut clipboard) => match clipboard.set_text(&path) {
                        Ok(_) => {
                            app.ui.status_message = format!("Copied to clipboard: {}", path);
                        }
                        Err(e) => {
                            app.ui.status_message = format!("Clipboard error: {}", e);
                        }
                    },
                    Err(e) => {
                        app.ui.status_message = format!("Clipboard unavailable: {}", e);
                    }
                }
            }
        }

        KeyCode::Char('/') => {
            app.libraries.library_search_active = true;
            app.ui.status_message =
                "Library search active. Type to filter, Esc to close.".to_string();
        }

        KeyCode::Delete => {
            app.libraries.library_search_query.clear();
            app.libraries.selected_library_index = 0;
            app.libraries.library_lib_scroll = 0;
            app.ui.status_message = "Search cleared.".to_string();
        }

        KeyCode::Char('f') | KeyCode::Char('F') => {
            app.libraries.library_risk_filter = match app.libraries.library_risk_filter.as_deref() {
                None => Some("Critical".to_string()),
                Some("Critical") => Some("Suspicious".to_string()),
                Some("Suspicious") => None,
                _ => None,
            };
            app.libraries.selected_library_index = 0;
            app.libraries.library_lib_scroll = 0;
            let label = match app.libraries.library_risk_filter.as_deref() {
                Some(f) => format!("Filter: {}", f),
                None => "Filter: All".to_string(),
            };
            app.ui.status_message = label;
        }

        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.refresh_libraries();
        }

        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.ui.selected_action_index = 5;
            execute_library_action(app);
        }

        KeyCode::Char('j') | KeyCode::Char('J') => {
            let sq = app.libraries.library_search_query.clone();
            let rf = app.libraries.library_risk_filter.clone();
            app.export_libraries_with_filter("json", &sq, rf.as_deref());
        }

        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                app.ui.should_quit = true;
            } else {
                let sq = app.libraries.library_search_query.clone();
                let rf = app.libraries.library_risk_filter.clone();
                app.export_libraries_with_filter("csv", &sq, rf.as_deref());
            }
        }

        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.ui.nav_sidebar_expanded = !app.ui.nav_sidebar_expanded;
            app.ui.sidebar_focus = SidebarFocus::Nav;
        }

        KeyCode::Char('v') | KeyCode::Char('V') => {
            app.ui.selected_action_index = 6;
            execute_library_action(app);
        }

        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.ui.show_language_modal = true;
        }

        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            if !app.libraries.library_search_query.is_empty() {
                app.libraries.library_search_query.clear();
                app.libraries.library_risk_filter = None;
                app.libraries.selected_library_index = 0;
                app.libraries.library_lib_scroll = 0;
                app.ui.status_message = "Filter cleared.".to_string();
            } else if app.investigation.investigation_report.is_some()
                || app.investigation.is_investigating
            {
                app.investigation.investigation_report = None;
                app.investigation.is_investigating = false;
                app.ui.analysis_paused = false;
            } else {
                switch_nav_view(app, NavView::Main);
                app.ui.sidebar_focus = SidebarFocus::Nav;
            }
        }

        _ => {}
    }
}

fn handle_storage_keys(app: &mut App, key: KeyEvent) {
    if app.ui.show_file_search_modal {
        handle_file_search_modal_keys(app, key);
        return;
    }
    match key.code {
        KeyCode::Tab => {
            if app.ui.sidebar_focus == SidebarFocus::Nav {
                app.ui.sidebar_focus = SidebarFocus::Left;
                app.storage.storage_focus = 0;
            } else if app.storage.storage_focus >= 2 {
                app.ui.sidebar_focus = SidebarFocus::Nav;
            } else {
                app.storage.storage_focus += 1;
            }
        }
        KeyCode::BackTab => {
            if app.ui.sidebar_focus == SidebarFocus::Nav {
                app.ui.sidebar_focus = SidebarFocus::Left;
                app.storage.storage_focus = 2;
            } else if app.storage.storage_focus == 0 {
                app.ui.sidebar_focus = SidebarFocus::Nav;
            } else {
                app.storage.storage_focus -= 1;
            }
        }
        KeyCode::Up => {
            if app.ui.sidebar_focus == SidebarFocus::Nav {
                let next = match app.ui.current_nav_view {
                    NavView::Main => NavView::Containers,
                    NavView::TrendGraphs => NavView::Main,
                    NavView::Storage => NavView::TrendGraphs,
                    NavView::LibraryInspection => NavView::Storage,
                    NavView::Containers => NavView::LibraryInspection,
                };
                switch_nav_view(app, next);
            } else {
                match app.storage.storage_focus {
                    0 if app.storage.selected_disk_index > 0 => {
                        app.storage.selected_disk_index -= 1;
                        load_selected_disk(app);
                    }
                    1 if app.storage.file_scroll > 0 => {
                        app.storage.file_scroll -= 1;
                    }
                    2 if app.storage.selected_storage_action_index > 0 => {
                        app.storage.selected_storage_action_index -= 1;
                    }
                    _ => {}
                }
            }
        }
        KeyCode::Down => {
            if app.ui.sidebar_focus == SidebarFocus::Nav {
                let next = match app.ui.current_nav_view {
                    NavView::Main => NavView::TrendGraphs,
                    NavView::TrendGraphs => NavView::Storage,
                    NavView::Storage => NavView::LibraryInspection,
                    NavView::LibraryInspection => NavView::Containers,
                    NavView::Containers => NavView::Main,
                };
                switch_nav_view(app, next);
            } else {
                match app.storage.storage_focus {
                    0 => {
                        let max = app.storage.disks.len().saturating_sub(1);
                        if app.storage.selected_disk_index < max {
                            app.storage.selected_disk_index += 1;
                            load_selected_disk(app);
                        }
                    }
                    1 => {
                        let max = app.storage.file_entries.len().saturating_sub(1);
                        if app.storage.file_scroll < max {
                            app.storage.file_scroll += 1;
                        }
                    }
                    2 => {
                        let max = config::STORAGE_ACTION_COUNT;
                        if app.storage.selected_storage_action_index < max {
                            app.storage.selected_storage_action_index += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        KeyCode::Enter => {
            if app.storage.storage_focus == 0 {
                if let Some(disk) = app.get_selected_disk() {
                    let p = std::path::Path::new(&disk.mount_point);
                    if p.exists() {
                        app.storage.current_directory = p.to_path_buf();
                        app.storage.file_scroll = 0;
                        load_directory(app);
                    }
                }
            } else if app.storage.storage_focus == 1 {
                open_selected_file(app);
            }
        }
        KeyCode::Backspace if app.storage.storage_focus == 1 => {
            if let Some(parent) = app.storage.current_directory.parent() {
                app.storage.current_directory = parent.to_path_buf();
                app.storage.file_scroll = 0;
                app.storage.file_search_mode = false;
                app.storage.file_search_query.clear();
                load_directory(app);
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            refresh_disks(app);
        }
        KeyCode::Char('h') | KeyCode::Char('H') if app.storage.storage_focus == 1 => {
            #[cfg(windows)]
            {
                let root = app
                    .storage
                    .current_directory
                    .components()
                    .next()
                    .map(|c| c.as_os_str().to_os_string())
                    .unwrap_or_else(|| std::ffi::OsString::from("C:\\"));
                app.storage.current_directory = std::path::PathBuf::from(root);
            }
            #[cfg(unix)]
            {
                app.storage.current_directory = std::path::PathBuf::from("/");
            }
            app.storage.file_scroll = 0;
            app.storage.file_search_mode = false;
            app.storage.file_search_query.clear();
            load_directory(app);
        }
        KeyCode::Char('p') | KeyCode::Char('P') if app.storage.storage_focus == 1 => {
            open_selected_file(app);
        }
        KeyCode::Char('s') | KeyCode::Char('S') if app.storage.storage_focus == 1 => {
            app.storage.file_sort_mode = app.storage.file_sort_mode.next();
            sort_file_entries(app);
            app.storage.file_scroll = 0;
            app.compute_filtered_indices();
            app.ui.status_message = format!("Sort: {}", app.storage.file_sort_mode.label());
        }
        KeyCode::Char('/') if app.storage.storage_focus == 1 => {
            app.ui.show_file_search_modal = true;
            app.ui.file_search_state = crate::app::types::FileSearchState::default();
        }
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            switch_nav_view(app, NavView::Main);
            app.ui.sidebar_focus = SidebarFocus::Nav;
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.ui.nav_sidebar_expanded = !app.ui.nav_sidebar_expanded;
            app.ui.sidebar_focus = SidebarFocus::Nav;
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.ui.show_language_modal = !app.ui.show_language_modal;
        }
        _ => {}
    }
}

fn handle_file_viewer_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.storage.show_file_viewer = false;
            app.storage.file_viewer_content.clear();
        }
        KeyCode::Up => {
            app.storage.file_viewer_scroll = app.storage.file_viewer_scroll.saturating_sub(1);
        }
        KeyCode::Down => {
            let max = app.storage.file_viewer_content.len().saturating_sub(1);
            app.storage.file_viewer_scroll =
                app.storage.file_viewer_scroll.saturating_add(1).min(max);
        }
        KeyCode::PageUp => {
            app.storage.file_viewer_scroll = app.storage.file_viewer_scroll.saturating_sub(20);
        }
        KeyCode::PageDown => {
            let max = app.storage.file_viewer_content.len().saturating_sub(1);
            app.storage.file_viewer_scroll =
                app.storage.file_viewer_scroll.saturating_add(20).min(max);
        }
        KeyCode::End => {
            app.storage.file_viewer_scroll =
                app.storage.file_viewer_content.len().saturating_sub(1);
        }
        _ => {}
    }
}

fn handle_file_search_modal_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.ui.show_file_search_modal = false;
        }
        KeyCode::Tab => {
            app.ui.file_search_state.focused_field =
                (app.ui.file_search_state.focused_field + 1) % config::SEARCH_MODAL_FIELD_COUNT;
        }
        KeyCode::BackTab => {
            app.ui.file_search_state.focused_field = if app.ui.file_search_state.focused_field == 0
            {
                config::SEARCH_MODAL_FIELD_COUNT - 1
            } else {
                app.ui.file_search_state.focused_field - 1
            };
        }

        KeyCode::Char(c)
            if app.ui.file_search_state.focused_field == 0
                && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.ui.file_search_state.query.push(c);
        }
        KeyCode::Backspace if app.ui.file_search_state.focused_field == 0 => {
            app.ui.file_search_state.query.pop();
        }

        KeyCode::Enter | KeyCode::Char(' ') if app.ui.file_search_state.focused_field == 1 => {
            app.ui.file_search_state.recursive = !app.ui.file_search_state.recursive;
        }

        KeyCode::Left if app.ui.file_search_state.focused_field == 2 => {
            let max = crate::app::storage::FILE_EXTENSION_FILTERS
                .len()
                .saturating_sub(1);
            app.ui.file_search_state.extension_idx = if app.ui.file_search_state.extension_idx == 0
            {
                max
            } else {
                app.ui.file_search_state.extension_idx - 1
            };
        }
        KeyCode::Right if app.ui.file_search_state.focused_field == 2 => {
            let max = crate::app::storage::FILE_EXTENSION_FILTERS
                .len()
                .saturating_sub(1);
            app.ui.file_search_state.extension_idx =
                if app.ui.file_search_state.extension_idx >= max {
                    0
                } else {
                    app.ui.file_search_state.extension_idx + 1
                };
        }

        KeyCode::Enter if app.ui.file_search_state.focused_field == 3 => {
            app.storage.file_search_query = app.ui.file_search_state.query.clone();
            app.storage.file_search_recursive = app.ui.file_search_state.recursive;
            app.storage.file_search_extension_idx = app.ui.file_search_state.extension_idx;
            app.storage.file_search_mode = true;
            app.ui.show_file_search_modal = false;
            app.abort_search();
            if app.storage.file_search_recursive {
                start_recursive_search(app);
            } else {
                load_directory(app);
                app.compute_filtered_indices();
            }
            app.ui.status_message = tr!(app.ui.translator, "status.search_active").to_string();
        }

        KeyCode::Enter if app.ui.file_search_state.focused_field == 4 => {
            app.ui.show_file_search_modal = false;
        }
        _ => {}
    }
}

fn load_selected_disk(app: &mut App) {
    app.abort_search();
    app.storage.file_search_mode = false;
    app.storage.file_search_query.clear();
    if let Some(disk) = app.get_selected_disk() {
        let p = std::path::Path::new(&disk.mount_point);
        if p.exists() {
            app.storage.current_directory = p.to_path_buf();
            app.storage.file_scroll = 0;
            load_directory(app);
        }
    }
}

fn start_recursive_search(app: &mut App) {
    let start_dir = app.storage.current_directory.clone();
    let query = app.storage.file_search_query.to_lowercase();
    let ext_idx = app.storage.file_search_extension_idx.min(
        crate::app::storage::FILE_EXTENSION_FILTERS
            .len()
            .saturating_sub(1),
    );
    let exts = crate::app::storage::FILE_EXTENSION_FILTERS[ext_idx].1;
    let (tx, rx) = std::sync::mpsc::channel();
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let abort = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let c = count.clone();
    let a = abort.clone();
    std::thread::spawn(move || {
        let mut all = Vec::new();
        let mut dirs = vec![start_dir];
        while let Some(dir) = dirs.pop() {
            if a.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            if let Ok(entries) = crate::app::storage::StorageManager::list_directory(&dir) {
                for entry in entries {
                    if entry.is_dir {
                        dirs.push(entry.path.clone());

                        c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    } else {
                        let matches_query =
                            query.is_empty() || entry.name.to_lowercase().contains(&query);
                        let matches_ext = exts.is_empty()
                            || exts.contains(&entry.extension.to_lowercase().as_str());
                        if matches_query && matches_ext {
                            all.push(entry);
                        }
                        c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }
        }
        let _ = tx.send(all);
    });
    app.storage.search_progress_rx = Some(rx);
    app.storage.search_progress_count = Some(count);
    app.storage.search_progress_abort = Some(abort);
    app.storage.search_progress_running = true;
    app.storage.search_progress_found = 0;
}

fn open_selected_file(app: &mut App) {
    let idx = app
        .storage
        .file_scroll
        .min(app.storage.file_entries.len().saturating_sub(1));
    let Some(entry) = app.storage.file_entries.get(idx) else {
        return;
    };
    if entry.is_dir {
        let dir = entry.path.clone();
        app.abort_search();
        app.storage.file_search_mode = false;
        app.storage.file_search_query.clear();
        app.storage.current_directory = dir;
        app.storage.file_scroll = 0;
        load_directory(app);
        return;
    }
    if crate::app::storage::StorageManager::is_text_file(&entry.extension) {
        match crate::app::storage::StorageManager::read_file(&entry.path) {
            Ok(content) => {
                app.storage.file_viewer_content = content.lines().map(|l| l.to_string()).collect();
                app.storage.file_viewer_scroll = 0;
                app.storage.show_file_viewer = true;
            }
            Err(e) => {
                app.ui.status_message = format!("[-] Failed to read file: {}", e);
            }
        }
    } else if crate::app::storage::StorageManager::is_image_file(&entry.extension) {
        let preview = crate::app::storage::render_image_preview(&entry.path);
        app.storage.file_viewer_is_ansi = preview.is_some();
        if let Some(lines) = preview {
            app.storage.file_viewer_content = lines;
        } else {
            let size = fmt_size(entry.size);
            app.storage.file_viewer_content = vec![
                    format!("File: {}", entry.name),
                    format!("Size: {}", size),
                    format!("Type: Image ({})", entry.extension.to_uppercase()),
                    format!("Path: {}", entry.path.display()),
                    String::new(),
                    "Image preview not available (install chafa/catimg on Linux, or PowerShell + .NET on Windows).".to_string(),
                    "Press Esc to close.".to_string(),
                ];
        }
        app.storage.file_viewer_scroll = 0;
        app.storage.show_file_viewer = true;
    } else {
        let size = fmt_size(entry.size);
        app.storage.file_viewer_content = vec![
            format!("File: {}", entry.name),
            format!("Size: {}", size),
            format!("Path: {}", entry.path.display()),
            String::new(),
            "Binary file — cannot display content.".to_string(),
            "Press Esc to close.".to_string(),
        ];
        app.storage.file_viewer_scroll = 0;
        app.storage.show_file_viewer = true;
    }
}

fn load_directory(app: &mut App) {
    app.storage.file_entries =
        crate::app::storage::StorageManager::list_directory(&app.storage.current_directory)
            .unwrap_or_default();
    sort_file_entries(app);
    app.storage.file_scroll = 0;
    app.compute_filtered_indices();
}

fn refresh_disks(app: &mut App) {
    app.storage.disks = crate::app::storage::StorageManager::list_disks();
    app.storage.disks_loading = false;
    app.ui.status_message = tr!(app.ui.translator, "storage.refreshed").to_string();
}

fn sort_file_entries(app: &mut App) {
    crate::app::storage::StorageManager::sort_entries(
        &mut app.storage.file_entries,
        app.storage.file_sort_mode,
    );
}

fn handle_language_keys(app: &mut App, key: KeyEvent) {
    let locales = crate::i18n::Translator::available_locales();
    let locale_count = locales.len();
    let visible = config::LANGUAGE_VISIBLE_ITEMS;
    match key.code {
        KeyCode::Esc => {
            app.ui.show_language_modal = false;
        }
        KeyCode::Up => {
            if app.ui.language_selection_index > 0 {
                app.ui.language_selection_index -= 1;
            } else {
                app.ui.language_selection_index = locale_count - 1;
            }
            if app.ui.language_selection_index < app.ui.language_scroll_offset {
                app.ui.language_scroll_offset = app.ui.language_selection_index;
            }
        }
        KeyCode::Down => {
            if app.ui.language_selection_index < locale_count - 1 {
                app.ui.language_selection_index += 1;
            } else {
                app.ui.language_selection_index = 0;
            }
            if app.ui.language_selection_index >= app.ui.language_scroll_offset + visible {
                app.ui.language_scroll_offset =
                    app.ui.language_selection_index.saturating_sub(visible - 1);
            }
        }
        KeyCode::Enter => {
            if let Some((code, _)) = locales.get(app.ui.language_selection_index) {
                app.ui.translator = crate::i18n::Translator::new(code);
                app.ui.show_language_modal = false;
                app.ui.status_message = tr!(app.ui.translator, "status.language_changed", *code);
                crate::config::save_language(code);
            }
        }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let idx = (c as u8 - b'0') as usize;
            if let Some((code, _)) = locales.get(idx) {
                app.ui.translator = crate::i18n::Translator::new(code);
                app.ui.show_language_modal = false;
                app.ui.status_message = tr!(app.ui.translator, "status.language_changed", *code);
                crate::config::save_language(code);
            }
        }
        _ => {}
    }
}

fn handle_search_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.ui.search_mode = false;
            app.ui.search_query.clear();
            app.network.selected_app_index = 0;
            app.ui.status_message = tr!(app.ui.translator, "status.search_closed").to_string();
        }
        KeyCode::Enter => {
            app.ui.search_mode = false;
            app.network.selected_app_index = 0;
            let count = app.get_filtered_apps().len();
            app.ui.status_message = tr!(
                app.ui.translator,
                "status.search_results",
                count,
                &app.ui.search_query
            );
        }
        KeyCode::Backspace => {
            app.ui.search_query.pop();
            app.network.selected_app_index = 0;
        }
        KeyCode::Char(c) => {
            app.ui.search_query.push(c);
            app.network.selected_app_index = 0;
        }
        _ => {}
    }
}
fn handle_confirmation_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y')
        | KeyCode::Char('Y')
        | KeyCode::Char('s')
        | KeyCode::Char('S')
        | KeyCode::Enter => {
            if let Some(action) = app.ui.pending_confirmation_action {
                match action {
                    ConfirmationAction::KillProcess => {
                        let pid = app.get_selected_app().map(|s| s.pid);
                        if let Some(pid) = pid {
                            app.ui.action_in_progress =
                                Some(tr!(app.ui.translator, "status.killing_process", pid));
                            let tx = app.ui.status_message_tx.clone();

                            let success_msg = tr!(app.ui.translator, "status.killed", pid);
                            let error_msg_prefix = tr!(app.ui.translator, "status.kill_error", "");

                            std::thread::spawn(move || {
                                let mut manager = crate::app::process::ProcessManager::new();
                                let result = manager.kill_process(pid);
                                if let Some(tx) = tx {
                                    match result {
                                        Ok(_) => {
                                            let _ = tx.send(success_msg);
                                        }
                                        Err(e) => {
                                            let _ = tx.send(format!("{}{}", error_msg_prefix, e));
                                        }
                                    }
                                }
                            });
                        }
                    }
                    ConfirmationAction::KillAllConnections => {
                        let pid = app.get_selected_app().map(|s| s.pid);
                        if let Some(pid) = pid {
                            app.ui.action_in_progress =
                                Some(tr!(app.ui.translator, "status.closing_connections", pid));
                            let tx = app.ui.status_message_tx.clone();

                            let success_msg_fmt = tr!(app.ui.translator, "status.kill_conns", pid);
                            let error_msg_prefix = tr!(app.ui.translator, "status.kill_error", "");

                            std::thread::spawn(move || {
                                let manager = crate::app::process::ProcessManager::new();
                                let result = manager.kill_connections(pid);
                                if let Some(tx) = tx {
                                    match result {
                                        Ok(count) => {
                                            let _ = tx.send(
                                                success_msg_fmt.replace("{}", &count.to_string()),
                                            );
                                        }
                                        Err(e) => {
                                            let _ = tx.send(format!("{}{}", error_msg_prefix, e));
                                        }
                                    }
                                }
                            });
                        }
                    }
                }
                app.ui.pending_confirmation_action = None;
            } else if let Some(container_action) = app.containers.pending_container_action {
                app.run_selected_container_action_confirmed(container_action);
                app.containers.pending_container_action = None;
            } else if let Some(docker_action) = app.containers.pending_docker_action {
                app.containers.docker_action_in_progress = Some(docker_action);
                app.execute_docker_action_confirmed(docker_action);
                app.containers.pending_docker_action = None;
            }
            app.ui.show_confirmation = false;
            app.ui.confirmation_message.clear();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.ui.show_confirmation = false;
            app.ui.pending_confirmation_action = None;
            app.ui.status_message = tr!(app.ui.translator, "status.action_cancelled").to_string();
            app.ui.confirmation_message.clear();
            app.containers.pending_container_action = None;
            app.containers.pending_docker_action = None;
        }
        _ => {}
    }
}
fn handle_nerdfont_dialog_keys(app: &mut App, key: KeyEvent) {
    if app.nerdfont.installing && !app.nerdfont.install_done {
        if key.code == KeyCode::Esc {
            app.nerdfont.show_dialog = false;
            app.nerdfont.installing = false;
            app.nerdfont.dialog_dismissed = true;
            app.ui.status_message = tr!(app.ui.translator, "status.nerdfont_cancelled").to_string();
        }
        return;
    }
    match key.code {
        KeyCode::Enter => {
            if !app.nerdfont.installing {
                app.nerdfont.installing = true;
                app.nerdfont.install_done = false;
                app.nerdfont.install_message =
                    tr!(app.ui.translator, "dialog.nerdfont_start").to_string();
                app.ui.status_message =
                    tr!(app.ui.translator, "status.nerdfont_installing").to_string();
                crate::app::installation::spawn_nerdfont_install(
                    &mut app.nerdfont.install_rx,
                    &mut app.nerdfont.install_message,
                );
            } else {
                app.nerdfont.show_dialog = false;
                app.nerdfont.dialog_dismissed = true;
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.nerdfont.show_dialog = false;
            app.nerdfont.dialog_dismissed = true;
            app.ui.status_message = tr!(app.ui.translator, "status.nerdfont_skipped").to_string();
        }
        _ => {}
    }
}
fn handle_install_dialog_keys(app: &mut App, key: KeyEvent) {
    if app.install.done {
        match key.code {
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                app.install.show_dialog = false;
            }
            _ => {}
        }
        return;
    }
    if app.install.installing {
        if key.code == KeyCode::Esc {
            app.install.show_dialog = false;
            app.install.installing = false;
            app.ui.status_message = tr!(app.ui.translator, "status.install_cancelled").to_string();
        }
        return;
    }
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            app.install.installing = true;
            app.install.done = false;
            app.install.needs_password = false;
            app.install.message = tr!(app.ui.translator, "dialog.net_tools_checking").to_string();
            app.ui.status_message = tr!(app.ui.translator, "status.install_checking").to_string();
            crate::app::installation::spawn_check_sudo(&mut app.install.child);
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.install.show_dialog = false;
            app.install.message.clear();
            app.ui.status_message = tr!(app.ui.translator, "status.install_cancelled").to_string();
        }
        _ => {}
    }
}
fn handle_update_dialog_keys(app: &mut App, key: KeyEvent) {
    if app.update.is_updating {
        if key.code == KeyCode::Esc {
            app.update.show_update_dialog = false;
        }
        return;
    }
    if app.update.update_done {
        match key.code {
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                app.update.show_update_dialog = false;
            }
            _ => {}
        }
        return;
    }
    match key.code {
        KeyCode::Enter => {
            app.start_self_update();
        }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.update.show_update_dialog = false;
        }
        _ => {}
    }
}
fn handle_password_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.install.show_password_modal = false;
            app.install.password.clear();
            app.install.installing = false;
            app.install.done = true;
            app.install.success = false;
            app.install.message = tr!(app.ui.translator, "dialog.password_cancelled").to_string();
            app.ui.status_message = tr!(app.ui.translator, "status.install_cancelled").to_string();
        }
        KeyCode::Enter if !app.install.password.is_empty() => {
            let password = std::mem::take(&mut app.install.password);
            app.install.show_password_modal = false;
            app.ui.status_message = tr!(app.ui.translator, "status.install_installing").to_string();
            crate::app::installation::spawn_install_with_password(&mut app.install.child, password);
        }
        KeyCode::Backspace => {
            app.install.password.pop();
        }
        KeyCode::Char(c) => {
            app.install.password.push(c);
        }
        _ => {}
    }
}
fn handle_firewall_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            exit_firewall_mode(app);
        }
        KeyCode::Tab => {
            app.firewall.firewall_focus =
                FirewallManager::cycle_focus_forward(app.firewall.firewall_focus);
            app.firewall.firewall_action_index = 0;
        }
        KeyCode::BackTab => {
            app.firewall.firewall_focus =
                FirewallManager::cycle_focus_backward(app.firewall.firewall_focus);
            app.firewall.firewall_action_index = 0;
        }
        KeyCode::Up => firewall_scroll(app, -1),
        KeyCode::Down => firewall_scroll(app, 1),
        KeyCode::Char(' ') => match app.firewall.firewall_focus {
            FirewallPanel::Connections => {
                if let Some(checked) = app
                    .firewall
                    .firewall_conn_checked
                    .get_mut(app.firewall.firewall_conn_index)
                {
                    *checked = !*checked;
                }
            }
            FirewallPanel::BlockedList => {
                if let Some(checked) = app
                    .firewall
                    .firewall_blocked_checked
                    .get_mut(app.firewall.firewall_blocked_index)
                {
                    *checked = !*checked;
                }
            }
            FirewallPanel::Actions => {
                toggle_selected_conn_checkbox(app);
            }
        },
        KeyCode::Enter => match app.firewall.firewall_focus {
            FirewallPanel::Connections => {
                if let Some(checked) = app
                    .firewall
                    .firewall_conn_checked
                    .get_mut(app.firewall.firewall_conn_index)
                {
                    *checked = !*checked;
                }
            }
            FirewallPanel::BlockedList => {
                if let Some(checked) = app
                    .firewall
                    .firewall_blocked_checked
                    .get_mut(app.firewall.firewall_blocked_index)
                {
                    *checked = !*checked;
                }
            }
            FirewallPanel::Actions => {
                execute_firewall_action(app);
            }
        },
        KeyCode::Char('b') | KeyCode::Char('B') => {
            app.firewall.firewall_action_index = 1;
            app.firewall.firewall_focus = FirewallPanel::Actions;
            execute_firewall_action(app);
        }
        KeyCode::Char('u') | KeyCode::Char('U') => {
            app.firewall.firewall_action_index = 2;
            app.firewall.firewall_focus = FirewallPanel::Actions;
            execute_firewall_action(app);
        }
        _ => {}
    }
}
fn firewall_scroll(app: &mut App, delta: i32) {
    match app.firewall.firewall_focus {
        FirewallPanel::Connections => {
            let max = app.firewall.firewall_connections.len().saturating_sub(1);
            app.firewall.firewall_conn_index =
                apply_scroll(app.firewall.firewall_conn_index, delta, max);
        }
        FirewallPanel::BlockedList => {
            let max = app.firewall.blocked_ips.len().saturating_sub(1);
            app.firewall.firewall_blocked_index =
                apply_scroll(app.firewall.firewall_blocked_index, delta, max);
        }
        FirewallPanel::Actions => {
            let max = FirewallManager::get_firewall_action_count();
            app.firewall.firewall_action_index =
                apply_scroll(app.firewall.firewall_action_index, delta, max);
        }
    }
}
pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent) {
    if app.ui.show_language_modal
        || app.install.show_password_modal
        || app.nerdfont.show_dialog
        || app.install.show_dialog
        || app.ui.show_confirmation
        || app.update.show_update_dialog
    {
        return;
    }
    if app.containers.show_container_logs_modal {
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                let max = app.containers.container_logs.len().saturating_sub(1);
                app.containers.container_logs_scroll = app
                    .containers
                    .container_logs_scroll
                    .saturating_add(1)
                    .min(max);
            }
            MouseEventKind::ScrollUp => {
                app.containers.container_logs_scroll =
                    app.containers.container_logs_scroll.saturating_sub(1);
            }
            _ => {}
        }
        return;
    }
    if app.firewall.firewall_mode {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                app.firewall.firewall_focus = FirewallManager::panel_from_x(mouse.column);
            }
            MouseEventKind::ScrollDown => {
                firewall_scroll(app, 1);
            }
            MouseEventKind::ScrollUp => {
                firewall_scroll(app, -1);
            }
            _ => {}
        }
        return;
    }
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            handle_dashboard_mouse_click(app, mouse.column, mouse.row);
        }
        MouseEventKind::ScrollDown => {
            handle_mouse_scroll(app, 1);
        }
        MouseEventKind::ScrollUp => {
            handle_mouse_scroll(app, -1);
        }
        _ => {}
    }
}
fn handle_dashboard_mouse_click(app: &mut App, x: u16, _y: u16) {
    let (term_width, _) = crossterm::terminal::size()
        .unwrap_or((config::DEFAULT_TERM_WIDTH, config::DEFAULT_TERM_HEIGHT));

    let nav_width = if app.ui.nav_sidebar_expanded { 20 } else { 7 };

    if x < nav_width {
        app.ui.sidebar_focus = SidebarFocus::Nav;
        return;
    }

    let remaining_width = term_width.saturating_sub(nav_width);
    let left =
        nav_width + (remaining_width as f32 * config::SIDEBAR_LEFT_PCT as f32 / 100.0) as u16;
    let center = left + (remaining_width as f32 * config::CENTER_PANEL_PCT as f32 / 100.0) as u16;

    if x < left {
        app.ui.sidebar_focus = SidebarFocus::Left;
    } else if x < center {
        app.ui.sidebar_focus = SidebarFocus::Center;
    } else {
        app.ui.sidebar_focus = SidebarFocus::Right;
    }
}
fn handle_mouse_scroll(app: &mut App, delta: i32) {
    if app.ui.current_nav_view == NavView::LibraryInspection
        && app.libraries.show_library_binary_viewer
    {
        let max = if app.libraries.library_binary_tab == 0 {
            app.libraries
                .library_binary_hex_lines
                .len()
                .saturating_sub(1)
        } else {
            app.libraries
                .library_binary_disasm_lines
                .len()
                .saturating_sub(1)
        };
        app.libraries.library_binary_scroll =
            apply_scroll(app.libraries.library_binary_scroll, delta, max);
        return;
    }
    if app.ui.current_nav_view == NavView::Storage && app.storage.show_file_viewer {
        let max = app.storage.file_viewer_content.len().saturating_sub(1);
        app.storage.file_viewer_scroll = apply_scroll(app.storage.file_viewer_scroll, delta, max);
        return;
    }
    match app.ui.sidebar_focus {
        SidebarFocus::Nav => {
            if delta > 0 {
                let next = match app.ui.current_nav_view {
                    NavView::Main => NavView::TrendGraphs,
                    NavView::TrendGraphs => NavView::Storage,
                    NavView::Storage => NavView::LibraryInspection,
                    NavView::LibraryInspection => NavView::Containers,
                    NavView::Containers => NavView::Main,
                };
                switch_nav_view(app, next);
            } else {
                let next = match app.ui.current_nav_view {
                    NavView::Main => NavView::Containers,
                    NavView::TrendGraphs => NavView::Main,
                    NavView::Storage => NavView::TrendGraphs,
                    NavView::LibraryInspection => NavView::Storage,
                    NavView::Containers => NavView::LibraryInspection,
                };
                switch_nav_view(app, next);
            }
        }
        SidebarFocus::Left => {
            if app.ui.current_nav_view == NavView::Containers {
                let max = app.containers.containers.len().saturating_sub(1);
                if apply_scroll_bool(app.containers.selected_container_index, delta, max) {
                    app.containers.selected_container_index =
                        apply_scroll(app.containers.selected_container_index, delta, max);
                    app.containers.container_detail_scroll = 0;
                    app.containers.container_logs.clear();
                }
            } else if app.ui.current_nav_view == NavView::Storage {
                let max = app.storage.disks.len().saturating_sub(1);
                app.storage.selected_disk_index =
                    apply_scroll(app.storage.selected_disk_index, delta, max);
            } else if app.ui.current_nav_view == NavView::LibraryInspection {
                let groups = app.group_libs_by_process();
                let max = groups.len().saturating_sub(1);
                if apply_scroll_bool(app.libraries.selected_library_process_index, delta, max) {
                    app.libraries.selected_library_process_index =
                        apply_scroll(app.libraries.selected_library_process_index, delta, max);
                    app.libraries.selected_library_index = 0;
                    app.libraries.library_lib_scroll = 0;
                }
            } else if app.investigation.investigation_report.is_none()
                && !app.investigation.is_investigating
            {
                let max = app.get_filtered_apps().len().saturating_sub(1);
                if apply_scroll_bool(app.network.selected_app_index, delta, max) {
                    app.network.selected_app_index =
                        apply_scroll(app.network.selected_app_index, delta, max);
                    app.trigger_geo_lookup_for_selected_app();
                }
            }
        }
        SidebarFocus::Center => {
            if app.ui.current_nav_view == NavView::Containers {
                if app.containers.show_container_logs_modal {
                    let max = app.containers.container_logs.len().saturating_sub(1);
                    app.containers.container_logs_scroll =
                        apply_scroll(app.containers.container_logs_scroll, delta, max);
                } else {
                    let max = app.containers.container_logs.len().saturating_sub(1);
                    app.containers.container_detail_scroll =
                        apply_scroll(app.containers.container_detail_scroll, delta, max);
                }
            } else if app.ui.current_nav_view == NavView::Storage {
                let max = app.storage.file_entries.len().saturating_sub(1);
                app.storage.file_scroll = apply_scroll(app.storage.file_scroll, delta, max);
            } else if app.ui.current_nav_view == NavView::LibraryInspection {
                let libs = crate::app::ui::libraries::get_libs_for_selected_process(app);
                let max = libs.len().saturating_sub(1);
                app.libraries.selected_library_index =
                    apply_scroll(app.libraries.selected_library_index, delta, max);
            } else if app.investigation.investigation_report.is_none()
                && !app.investigation.is_investigating
            {
                if let Some(selected) = app.get_selected_app() {
                    let max = selected.connections.len().saturating_sub(1);
                    app.network.selected_connection_index =
                        apply_scroll(app.network.selected_connection_index, delta, max);
                }
            }
        }
        SidebarFocus::Right => {
            if app.ui.current_nav_view == NavView::Containers {
                let max = crate::app::containers::CONTAINER_RIGHT_ACTION_COUNT.saturating_sub(1);
                app.containers.selected_container_action_index =
                    apply_scroll(app.containers.selected_container_action_index, delta, max);
            } else if app.ui.current_nav_view == NavView::LibraryInspection {
                let max = crate::app::libraries::LIBRARY_ACTION_COUNT.saturating_sub(1);
                app.ui.selected_action_index =
                    apply_scroll(app.ui.selected_action_index, delta, max);
            } else if app.ui.current_nav_view == NavView::Storage {
            } else {
                let max = config::ACTION_COUNT;
                app.ui.selected_action_index =
                    apply_scroll(app.ui.selected_action_index, delta, max);
            }
        }
    }
}
pub fn execute_action(app: &mut App) {
    if app.investigation.investigation_report.is_some() {
        if app.ui.selected_action_index == 0 {
            app.ui.show_map = !app.ui.show_map;
            app.ui.selected_action_index = 0;
        }
        return;
    }
    match app.ui.selected_action_index {
        0 => {
            app.ui.analysis_paused = !app.ui.analysis_paused;
            app.ui.continuous_refresh_counter = 0;
            if app.ui.analysis_paused {
                app.ui.status_message =
                    tr!(app.ui.translator, "status.analysis_paused").to_string();
            } else {
                app.ui.status_message =
                    tr!(app.ui.translator, "status.analysis_resumed").to_string();
                app.start_batch_analysis();
            }
        }
        1 => {
            if let Some(selected) = app.get_selected_app() {
                app.ui.confirmation_message = tr!(
                    app.ui.translator,
                    "dialog.kill_process",
                    &selected.process_name,
                    selected.pid
                );
                app.ui.pending_confirmation_action = Some(ConfirmationAction::KillProcess);
                app.ui.show_confirmation = true;
            } else {
                app.ui.status_message = tr!(app.ui.translator, "status.no_selection").to_string();
            }
        }
        2 => {
            if let Some(selected) = app.get_selected_app() {
                app.ui.confirmation_message = tr!(
                    app.ui.translator,
                    "dialog.kill_conns",
                    selected.connections.len(),
                    &selected.process_name
                );
                app.ui.pending_confirmation_action = Some(ConfirmationAction::KillAllConnections);
                app.ui.show_confirmation = true;
            } else {
                app.ui.status_message = tr!(app.ui.translator, "status.no_selection").to_string();
            }
        }
        3 => {
            if let Some(selected) = app.get_selected_app() {
                let search_url = format!(
                    "{}{}",
                    resources::URLS.google_search_url,
                    urlencoding::encode(&selected.process_name)
                );
                if let Err(e) = open::that(&search_url) {
                    app.ui.status_message = tr!(app.ui.translator, "status.browser_fail", e);
                } else {
                    app.ui.status_message = tr!(
                        app.ui.translator,
                        "status.searching_online",
                        &selected.process_name
                    );
                }
            } else {
                app.ui.status_message = tr!(app.ui.translator, "status.no_selection").to_string();
            }
        }
        4 => {
            if let Some(selected) = app.get_selected_app() {
                let path = selected.process_path.clone();
                match arboard::Clipboard::new() {
                    Ok(mut clipboard) => match clipboard.set_text(&path) {
                        Ok(_) => {
                            app.ui.status_message = tr!(app.ui.translator, "status.copied", path);
                        }
                        Err(e) => {
                            app.ui.status_message =
                                tr!(app.ui.translator, "status.clipboard_fail", e);
                        }
                    },
                    Err(e) => {
                        app.ui.status_message =
                            tr!(app.ui.translator, "status.clipboard_unavail", e);
                    }
                }
            } else {
                app.ui.status_message = tr!(app.ui.translator, "status.no_selection").to_string();
            }
        }
        5 => {
            export_to_json(app);
        }
        6 => {
            app.ui.filter_high_risk_only = !app.ui.filter_high_risk_only;
            app.network.selected_app_index = 0;
            if app.ui.filter_high_risk_only {
                app.ui.status_message =
                    tr!(app.ui.translator, "status.filter_high_risk").to_string();
            } else {
                app.ui.status_message = tr!(app.ui.translator, "status.filter_all").to_string();
            }
        }
        7 => {
            enter_firewall_mode(app);
        }
        8 => {
            app.ui.show_language_modal = true;
        }
        _ => {}
    }
}
pub fn execute_library_action(app: &mut App) {
    match app.ui.selected_action_index {
        0 => app.refresh_libraries(),
        1 => {
            app.libraries.library_risk_filter = match app.libraries.library_risk_filter.as_deref() {
                None => Some("Critical".to_string()),
                Some("Critical") => Some("Suspicious".to_string()),
                Some("Suspicious") => None,
                _ => None,
            };
            app.libraries.selected_library_index = 0;
            app.libraries.library_lib_scroll = 0;
            let label = match app.libraries.library_risk_filter.as_deref() {
                Some(f) => format!("Filter: {}", f),
                None => "Filter: All".to_string(),
            };
            app.ui.status_message = label;
        }
        2 => {
            let libs = crate::app::ui::libraries::get_libs_for_selected_process(app);
            if let Some(lib) = libs.get(app.libraries.selected_library_index) {
                let path = lib.path.clone();
                match arboard::Clipboard::new() {
                    Ok(mut clipboard) => match clipboard.set_text(&path) {
                        Ok(_) => {
                            app.ui.status_message = format!("Copied to clipboard: {}", path);
                        }
                        Err(e) => {
                            app.ui.status_message = format!("Clipboard error: {}", e);
                        }
                    },
                    Err(e) => {
                        app.ui.status_message = format!("Clipboard unavailable: {}", e);
                    }
                }
            }
        }
        3 => {
            let sq = app.libraries.library_search_query.clone();
            let rf = app.libraries.library_risk_filter.clone();
            app.export_libraries_with_filter("json", &sq, rf.as_deref());
        }
        4 => {
            let sq = app.libraries.library_search_query.clone();
            let rf = app.libraries.library_risk_filter.clone();
            app.export_libraries_with_filter("csv", &sq, rf.as_deref());
        }
        5 => {
            app.libraries.show_hash_info_modal = true;
        }
        6 => {
            let libs = crate::app::ui::libraries::get_libs_for_selected_process(app);
            if let Some(lib) = libs.get(app.libraries.selected_library_index) {
                let path = lib.path.clone();
                app.libraries.library_binary_path = path.clone();
                app.libraries.library_binary_hex_lines =
                    crate::app::libraries::load_binary_hex(&path);
                app.libraries.library_binary_disasm_lines =
                    crate::app::libraries::load_binary_disasm(&path);
                app.libraries.library_binary_scroll = 0;
                app.libraries.library_binary_tab = 0;
                app.libraries.show_library_binary_viewer = true;
            } else {
                app.ui.status_message = "No library selected.".to_string();
            }
        }
        _ => {}
    }
}
fn execute_firewall_action(app: &mut App) {
    match app.firewall.firewall_action_index {
        0 => {
            toggle_selected_conn_checkbox(app);
        }
        1 => {
            let to_block: Vec<String> = app
                .firewall
                .firewall_connections
                .iter()
                .enumerate()
                .filter(|(i, _)| {
                    app.firewall
                        .firewall_conn_checked
                        .get(*i)
                        .copied()
                        .unwrap_or(false)
                })
                .map(|(_, conn)| conn.foreign_address.clone())
                .collect();
            if to_block.is_empty() {
                app.ui.status_message =
                    tr!(app.ui.translator, "status.firewall_no_conns").to_string();
                return;
            }
            let name = app.firewall.firewall_process_name.clone();
            let count = to_block.len();
            for ip in &to_block {
                FirewallManager::block_ip(ip, &name, &app.database);
            }
            app.firewall.firewall_conn_checked =
                vec![false; app.firewall.firewall_connections.len()];
            app.ui.status_message = tr!(app.ui.translator, "status.firewall_blocked", count);
        }
        2 => {
            let to_unblock: Vec<String> = app
                .firewall
                .blocked_ips
                .iter()
                .enumerate()
                .filter(|(i, _)| {
                    app.firewall
                        .firewall_blocked_checked
                        .get(*i)
                        .copied()
                        .unwrap_or(false)
                })
                .map(|(_, (ip, _, _))| ip.clone())
                .collect();
            if to_unblock.is_empty() {
                app.ui.status_message =
                    tr!(app.ui.translator, "status.firewall_no_ips").to_string();
                return;
            }
            let count = to_unblock.len();
            for ip in &to_unblock {
                FirewallManager::unblock_ip(ip, &app.database);
            }
            refresh_blocked_ips(app);
            app.ui.status_message = tr!(app.ui.translator, "status.firewall_unblocked", count);
        }
        3 => {
            exit_firewall_mode(app);
        }
        _ => {}
    }
}
fn enter_firewall_mode(app: &mut App) {
    let selected = app
        .get_selected_app()
        .map(|a| (a.connections.clone(), a.process_name.clone()));
    if let Some((conns, name)) = selected {
        app.firewall.firewall_connections = conns;
        app.firewall.firewall_process_name = name;
        app.firewall.firewall_conn_index = 0;
        app.firewall.firewall_blocked_index = 0;
        app.firewall.firewall_action_index = 0;
        app.firewall.firewall_focus = FirewallPanel::Connections;
        app.firewall.firewall_conn_checked = vec![false; app.firewall.firewall_connections.len()];
        refresh_blocked_ips(app);
        app.firewall.firewall_blocked_checked = vec![false; app.firewall.blocked_ips.len()];
        app.firewall.firewall_mode = true;
        app.ui.status_message = tr!(
            app.ui.translator,
            "status.firewall_entered",
            app.firewall.firewall_connections.len(),
            app.firewall.blocked_ips.len()
        );
    } else {
        app.ui.status_message = tr!(app.ui.translator, "status.no_selection").to_string();
    }
}
fn exit_firewall_mode(app: &mut App) {
    app.firewall.firewall_mode = false;
    app.firewall.firewall_connections.clear();
    app.firewall.firewall_process_name.clear();
    app.firewall.blocked_ips.clear();
    app.firewall.firewall_conn_checked.clear();
    app.firewall.firewall_blocked_checked.clear();
    app.ui.status_message = tr!(app.ui.translator, "status.firewall_exited").to_string();
}
fn refresh_blocked_ips(app: &mut App) {
    if let Ok(ips) = app.database.get_blocked_ips() {
        app.firewall.blocked_ips = ips;
        app.firewall.firewall_blocked_checked = vec![false; app.firewall.blocked_ips.len()];
        if app.firewall.firewall_blocked_index >= app.firewall.blocked_ips.len().saturating_sub(1) {
            app.firewall.firewall_blocked_index = app.firewall.blocked_ips.len().saturating_sub(1);
        }
    }
}
pub fn export_to_json(app: &mut App) {
    use std::fs::File;
    use std::io::Write;
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let default_name = format!("network_analysis_{}.json", timestamp);
    let path = pick_save_path(app, &default_name)
        .unwrap_or_else(|| std::path::PathBuf::from(&default_name));
    match serde_json::to_string_pretty(&app.network.app_connections) {
        Ok(json) => match File::create(&path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(json.as_bytes()) {
                    app.ui.status_message = tr!(app.ui.translator, "status.export_fail_write", e);
                } else {
                    app.ui.status_message = tr!(
                        app.ui.translator,
                        "status.exported",
                        path.display().to_string()
                    );
                }
            }
            Err(e) => {
                app.ui.status_message = tr!(app.ui.translator, "status.export_fail_create", e);
            }
        },
        Err(e) => {
            app.ui.status_message = tr!(app.ui.translator, "status.export_fail_serialize", e);
        }
    }
}
fn pick_save_path(_app: &App, default_name: &str) -> Option<std::path::PathBuf> {
    if cfg!(test) {
        return Some(std::path::PathBuf::from(default_name));
    }
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let script = format!(
            r#"Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.SaveFileDialog; $f.FileName = '{}'; $f.Filter = 'JSON Files (*.json)|*.json'; if ($f.ShowDialog() -eq 'OK') {{ $f.FileName }}"#,
            default_name
        );
        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output()
            .ok()?;
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(path))
        }
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        if let Ok(output) = Command::new("zenity")
            .args([
                "--file-selection",
                "--save",
                "--confirm-overwrite",
                "--title=Export Network Analysis",
                &format!("--filename={}", default_name),
            ])
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(std::path::PathBuf::from(path));
                }
            }
        }

        if let Ok(output) = Command::new("kdialog")
            .args([
                "--getsavefilename",
                ".",
                default_name,
                "--title",
                "Export Network Analysis",
            ])
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(std::path::PathBuf::from(path));
                }
            }
        }

        let py_script = format!(
            "import tkinter as tk; from tkinter import filedialog; root = tk.Tk(); root.withdraw(); \
             path = filedialog.asksaveasfilename(initialfile='{}', title='Export Network Analysis', \
             filetypes=[('JSON files','*.json'),('All files','*')]); print(path)",
            default_name
        );
        if let Ok(output) = Command::new("python3").args(["-c", &py_script]).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(std::path::PathBuf::from(path));
                }
            }
        }

        None
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        None
    }
}
#[allow(dead_code)]
pub fn pick_save_path_for_test(app: &App, default_name: &str) -> Option<std::path::PathBuf> {
    pick_save_path(app, default_name)
}
pub fn toggle_selected_conn_checkbox(app: &mut App) {
    if let Some(checked) = app
        .firewall
        .firewall_conn_checked
        .get_mut(app.firewall.firewall_conn_index)
    {
        *checked = !*checked;
    }
}
pub fn any_conn_checked(app: &App) -> bool {
    app.firewall.firewall_conn_checked.iter().any(|&c| c)
}
pub fn any_blocked_checked(app: &App) -> bool {
    app.firewall.firewall_blocked_checked.iter().any(|&c| c)
}

fn handle_welcome_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Left | KeyCode::Right => {
            app.ui.welcome_index = (app.ui.welcome_index + 1) % crate::config::WELCOME_PAGE_COUNT;
        }
        KeyCode::Enter => {
            if app.ui.welcome_index == crate::config::WELCOME_PAGE_COUNT - 1 {
                let _ = open::that(&resources::URLS.github_releases_page);
            }
            app.ui.show_welcome_dialog = false;
        }
        KeyCode::Esc => {
            app.ui.show_welcome_dialog = false;
        }
        _ => {}
    }
}
fn apply_scroll(current: usize, delta: i32, max: usize) -> usize {
    if delta > 0 {
        current.saturating_add(1).min(max)
    } else if delta < 0 {
        current.saturating_sub(1)
    } else {
        current
    }
}
fn apply_scroll_bool(current: usize, delta: i32, max: usize) -> bool {
    if delta > 0 {
        current < max
    } else if delta < 0 {
        current > 0
    } else {
        false
    }
}
