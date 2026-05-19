use crate::app::firewall_service::FirewallManager;
use crate::app::types::{FirewallPanel, NavView, SidebarFocus};
use crate::app::App;
use crate::config;
use crate::resources;
use crate::tr;
use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
impl App {
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if self.show_welcome_dialog {
            self.handle_welcome_keys(key);
            return;
        }
        if self.show_language_modal {
            self.handle_language_keys(key);
            return;
        }
        if self.show_password_modal {
            self.handle_password_keys(key);
            return;
        }
        if self.show_nerdfont_dialog {
            self.handle_nerdfont_dialog_keys(key);
            return;
        }
        if self.show_install_dialog {
            self.handle_install_dialog_keys(key);
            return;
        }
        if self.firewall_mode {
            self.handle_firewall_keys(key);
            return;
        }
        if self.show_confirmation {
            self.handle_confirmation_keys(key);
            return;
        }
        if self.show_update_dialog {
            self.handle_update_dialog_keys(key);
            return;
        }
        if self.search_mode {
            self.handle_search_keys(key);
            return;
        }
        if self.show_map {
            if key.code == KeyCode::Esc
                || key.code == KeyCode::Char('q')
                || key.code == KeyCode::Char('Q')
            {
                self.show_map = false;
                self.selected_action_index = 0;
            }
            return;
        }
        self.handle_dashboard_keys(key);
    }
    fn handle_dashboard_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                self.sidebar_focus = match self.sidebar_focus {
                    SidebarFocus::Nav => SidebarFocus::Left,
                    SidebarFocus::Left => SidebarFocus::Center,
                    SidebarFocus::Center => SidebarFocus::Right,
                    SidebarFocus::Right => SidebarFocus::Nav,
                };
                self.status_message = tr!(
                    self.translator,
                    "status.focus",
                    format!("{:?}", self.sidebar_focus)
                )
                .to_string();
            }
            KeyCode::BackTab => {
                self.sidebar_focus = match self.sidebar_focus {
                    SidebarFocus::Nav => SidebarFocus::Right,
                    SidebarFocus::Left => SidebarFocus::Nav,
                    SidebarFocus::Center => SidebarFocus::Left,
                    SidebarFocus::Right => SidebarFocus::Center,
                };
                self.status_message = tr!(
                    self.translator,
                    "status.focus",
                    format!("{:?}", self.sidebar_focus)
                )
                .to_string();
            }
            KeyCode::Up => {
                let in_investigation = self.investigation_report.is_some() || self.is_investigating;
                match self.sidebar_focus {
                    SidebarFocus::Nav => {
                        self.current_nav_view = match self.current_nav_view {
                            NavView::Main => NavView::Containers,
                            NavView::TrendGraphs => NavView::Main,
                            NavView::DgaDetector => NavView::TrendGraphs,
                            NavView::LibraryInspection => NavView::DgaDetector,
                            NavView::Containers => NavView::LibraryInspection,
                        };
                    }
                    SidebarFocus::Left if !in_investigation && self.selected_app_index > 0 => {
                        self.selected_app_index -= 1;
                        self.selected_connection_index = 0;
                        self.trigger_geo_lookup_for_selected_app();
                    }
                    SidebarFocus::Center
                        if !in_investigation && self.selected_connection_index > 0 =>
                    {
                        self.selected_connection_index -= 1;
                    }
                    SidebarFocus::Right if self.selected_action_index > 0 => {
                        self.selected_action_index -= 1;
                    }
                    _ => {}
                }
            }
            KeyCode::Down => {
                let in_investigation = self.investigation_report.is_some() || self.is_investigating;
                match self.sidebar_focus {
                    SidebarFocus::Nav => {
                        self.current_nav_view = match self.current_nav_view {
                            NavView::Main => NavView::TrendGraphs,
                            NavView::TrendGraphs => NavView::DgaDetector,
                            NavView::DgaDetector => NavView::LibraryInspection,
                            NavView::LibraryInspection => NavView::Containers,
                            NavView::Containers => NavView::Main,
                        };
                    }
                    SidebarFocus::Left if !in_investigation => {
                        let filtered_count = self.get_filtered_apps().len();
                        if self.selected_app_index < filtered_count.saturating_sub(1) {
                            self.selected_app_index += 1;
                            self.selected_connection_index = 0;
                            self.trigger_geo_lookup_for_selected_app();
                        }
                    }
                    SidebarFocus::Center if !in_investigation => {
                        if let Some(app) = self.get_selected_app() {
                            if self.selected_connection_index
                                < app.connections.len().saturating_sub(1)
                            {
                                self.selected_connection_index += 1;
                            }
                        }
                    }
                    SidebarFocus::Right if self.selected_action_index < config::ACTION_COUNT => {
                        self.selected_action_index += 1;
                    }
                    _ => {}
                }
            }
            KeyCode::Enter => {
                let in_investigation = self.investigation_report.is_some() || self.is_investigating;
                match self.sidebar_focus {
                    SidebarFocus::Nav => {
                        // Enter on Nav simply reflects that the view is already changed by Up/Down
                        // or we could expand/collapse
                        self.nav_sidebar_expanded = !self.nav_sidebar_expanded;
                    }
                    SidebarFocus::Right => self.execute_action(),
                    SidebarFocus::Center if !in_investigation => self.start_investigation(),
                    SidebarFocus::Left => {
                        self.sidebar_focus = SidebarFocus::Center;
                        self.selected_connection_index = 0;
                    }
                    _ => {}
                }
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.status_message = tr!(self.translator, "status.refresh").to_string();
                self.start_batch_analysis();
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.analysis_paused = !self.analysis_paused;
                self.continuous_refresh_counter = 0;
                if self.analysis_paused {
                    self.status_message =
                        tr!(self.translator, "status.analysis_paused").to_string();
                } else {
                    self.status_message =
                        tr!(self.translator, "status.analysis_resumed").to_string();
                    self.start_batch_analysis();
                }
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                self.enter_firewall_mode();
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                self.nav_sidebar_expanded = !self.nav_sidebar_expanded;
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.show_language_modal = true;
                if let Some(idx) = crate::i18n::Translator::available_locales()
                    .iter()
                    .position(|(code, _)| *code == self.translator.locale)
                {
                    self.language_selection_index = idx;
                } else {
                    self.language_selection_index = 0;
                }
            }
            KeyCode::Char('1') => {
                self.center_tab = 0;
                self.status_message = tr!(self.translator, "status.tab_connections").to_string();
            }
            KeyCode::Char('2') if self.get_selected_app().is_some() => {
                self.center_tab = 1;
                self.status_message = tr!(self.translator, "status.tab_risk").to_string();
            }
            KeyCode::Char('3') => {
                self.center_tab = 2;
                self.status_message = tr!(self.translator, "status.tab_timeline").to_string();
            }
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.search_query.clear();
                self.selected_app_index = 0;
                self.status_message = tr!(self.translator, "status.search_active").to_string();
            }
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.hunter_mode = !self.hunter_mode;
                self.selected_app_index = 0;
                if self.hunter_mode {
                    self.status_message = tr!(self.translator, "status.hunter_on").to_string();
                } else {
                    self.status_message = tr!(self.translator, "status.hunter_off").to_string();
                }
                self.start_batch_analysis();
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.filter_high_risk_only = !self.filter_high_risk_only;
                self.selected_app_index = 0;
                if self.filter_high_risk_only {
                    self.status_message =
                        tr!(self.translator, "status.filter_high_risk").to_string();
                } else {
                    self.status_message = tr!(self.translator, "status.filter_all").to_string();
                }
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                if let Some(app) = self.get_selected_app() {
                    self.confirmation_message = tr!(
                        self.translator,
                        "dialog.kill_process",
                        &app.process_name,
                        app.pid
                    );
                    self.show_confirmation = true;
                } else {
                    self.status_message = tr!(self.translator, "status.no_selection").to_string();
                }
            }
            KeyCode::Char('-') => {
                if let Some(app) = self.get_selected_app() {
                    self.confirmation_message = tr!(
                        self.translator,
                        "dialog.kill_conns",
                        app.connections.len(),
                        &app.process_name
                    );
                    self.show_confirmation = true;
                } else {
                    self.status_message = tr!(self.translator, "status.no_selection").to_string();
                }
            }
            KeyCode::Char('g') | KeyCode::Char('G') => {
                if let Some(app) = self.get_selected_app() {
                    let search_url = format!(
                        "{}{}",
                        resources::URLS.google_search_url,
                        urlencoding::encode(&app.process_name)
                    );
                    if let Err(e) = open::that(&search_url) {
                        self.status_message = tr!(self.translator, "status.browser_fail", e);
                    } else {
                        self.status_message = tr!(
                            self.translator,
                            "status.searching_online",
                            &app.process_name
                        );
                    }
                } else {
                    self.status_message = tr!(self.translator, "status.no_selection").to_string();
                }
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.should_quit = true;
                } else if let Some(app) = self.get_selected_app() {
                    let path = app.process_path.clone();
                    match arboard::Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(&path) {
                            Ok(_) => {
                                self.status_message = tr!(self.translator, "status.copied", path);
                            }
                            Err(e) => {
                                self.status_message =
                                    tr!(self.translator, "status.clipboard_fail", e);
                            }
                        },
                        Err(e) => {
                            self.status_message =
                                tr!(self.translator, "status.clipboard_unavail", e);
                        }
                    }
                } else {
                    self.status_message = tr!(self.translator, "status.no_selection").to_string();
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.export_to_json();
            }
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                if self.investigation_report.is_some() || self.is_investigating {
                    self.investigation_report = None;
                    self.is_investigating = false;
                    self.analysis_paused = false;
                } else if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.selected_app_index = 0;
                    self.status_message = tr!(self.translator, "status.filter_cleared").to_string();
                } else {
                    self.should_quit = true;
                }
            }
            _ => {}
        }
    }
    fn handle_language_keys(&mut self, key: KeyEvent) {
        let locales = crate::i18n::Translator::available_locales();
        let locale_count = locales.len();
        let visible = config::LANGUAGE_VISIBLE_ITEMS;
        match key.code {
            KeyCode::Esc => {
                self.show_language_modal = false;
            }
            KeyCode::Up => {
                if self.language_selection_index > 0 {
                    self.language_selection_index -= 1;
                } else {
                    self.language_selection_index = locale_count - 1;
                }
                if self.language_selection_index < self.language_scroll_offset {
                    self.language_scroll_offset = self.language_selection_index;
                }
            }
            KeyCode::Down => {
                if self.language_selection_index < locale_count - 1 {
                    self.language_selection_index += 1;
                } else {
                    self.language_selection_index = 0;
                }
                if self.language_selection_index >= self.language_scroll_offset + visible {
                    self.language_scroll_offset =
                        self.language_selection_index.saturating_sub(visible - 1);
                }
            }
            KeyCode::Enter => {
                if let Some((code, _)) = locales.get(self.language_selection_index) {
                    self.translator = crate::i18n::Translator::new(code);
                    self.show_language_modal = false;
                    self.status_message = tr!(self.translator, "status.language_changed", *code);
                    crate::config::save_language(code);
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let idx = (c as u8 - b'0') as usize;
                if let Some((code, _)) = locales.get(idx) {
                    self.translator = crate::i18n::Translator::new(code);
                    self.show_language_modal = false;
                    self.status_message = tr!(self.translator, "status.language_changed", *code);
                    crate::config::save_language(code);
                }
            }
            _ => {}
        }
    }

    fn handle_search_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search_mode = false;
                self.search_query.clear();
                self.selected_app_index = 0;
                self.status_message = tr!(self.translator, "status.search_closed").to_string();
            }
            KeyCode::Enter => {
                self.search_mode = false;
                self.selected_app_index = 0;
                let count = self.get_filtered_apps().len();
                self.status_message = tr!(
                    self.translator,
                    "status.search_results",
                    count,
                    &self.search_query
                );
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.selected_app_index = 0;
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.selected_app_index = 0;
            }
            _ => {}
        }
    }
    fn handle_confirmation_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if self.confirmation_message.contains("kill process") {
                    if let Some(app) = self.get_selected_app() {
                        let pid = app.pid;
                        let mut manager = crate::app::process::ProcessManager::new();
                        match manager.kill_process(pid) {
                            Ok(true) => {
                                self.status_message = tr!(self.translator, "status.killed", pid);
                                self.start_batch_analysis();
                            }
                            Ok(false) => {
                                self.status_message = tr!(self.translator, "status.kill_fail", pid);
                            }
                            Err(e) => {
                                self.status_message = format!("[!] {}", e);
                            }
                        }
                    }
                } else if self.confirmation_message.contains("kill all connections") {
                    if let Some(app) = self.get_selected_app() {
                        let pid = app.pid;
                        let conn_count = app.connections.len();
                        let manager = crate::app::process::ProcessManager::new();
                        match manager.kill_connections(pid) {
                            Ok(count) => {
                                if count > 0 {
                                    self.status_message = format!(
                                        "[+] {} connection(s) closed for process {}",
                                        count, pid
                                    );
                                    self.start_batch_analysis();
                                } else if conn_count > 0 {
                                    self.status_message = format!(
                                        "[!] Found {} connection(s) but failed to close them. Run as Administrator",
                                        conn_count
                                    );
                                } else {
                                    self.status_message =
                                        "[!] No active connections found for this process"
                                            .to_string();
                                }
                            }
                            Err(e) => {
                                self.status_message = format!("[!] {}", e);
                            }
                        }
                    }
                }
                self.show_confirmation = false;
                self.confirmation_message.clear();
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.status_message = tr!(self.translator, "status.action_cancelled").to_string();
                self.show_confirmation = false;
                self.confirmation_message.clear();
            }
            _ => {}
        }
    }
    fn handle_nerdfont_dialog_keys(&mut self, key: KeyEvent) {
        if self.nerdfont_installing && !self.nerdfont_install_done {
            if key.code == KeyCode::Esc {
                self.show_nerdfont_dialog = false;
                self.nerdfont_installing = false;
                self.nerdfont_dialog_dismissed = true;
                self.status_message = tr!(self.translator, "status.nerdfont_cancelled").to_string();
            }
            return;
        }
        match key.code {
            KeyCode::Enter => {
                if !self.nerdfont_installing {
                    self.nerdfont_installing = true;
                    self.nerdfont_install_done = false;
                    self.nerdfont_install_message =
                        tr!(self.translator, "dialog.nerdfont_start").to_string();
                    self.status_message =
                        tr!(self.translator, "status.nerdfont_installing").to_string();
                    crate::app::installation::spawn_nerdfont_install(
                        &mut self.nerdfont_install_rx,
                        &mut self.nerdfont_install_message,
                    );
                } else {
                    self.show_nerdfont_dialog = false;
                    self.nerdfont_dialog_dismissed = true;
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.show_nerdfont_dialog = false;
                self.nerdfont_dialog_dismissed = true;
                self.status_message = tr!(self.translator, "status.nerdfont_skipped").to_string();
            }
            _ => {}
        }
    }
    fn handle_install_dialog_keys(&mut self, key: KeyEvent) {
        if self.install_done {
            match key.code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.show_install_dialog = false;
                }
                _ => {}
            }
            return;
        }
        if self.is_installing {
            if key.code == KeyCode::Esc {
                self.show_install_dialog = false;
                self.is_installing = false;
                self.status_message = tr!(self.translator, "status.install_cancelled").to_string();
            }
            return;
        }
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                self.is_installing = true;
                self.install_done = false;
                self.install_needs_password = false;
                self.install_message =
                    tr!(self.translator, "dialog.net_tools_checking").to_string();
                self.status_message = tr!(self.translator, "status.install_checking").to_string();
                crate::app::installation::spawn_check_sudo(&mut self.install_child);
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.show_install_dialog = false;
                self.install_message.clear();
                self.status_message = tr!(self.translator, "status.install_cancelled").to_string();
            }
            _ => {}
        }
    }
    fn handle_update_dialog_keys(&mut self, key: KeyEvent) {
        if self.is_updating {
            if key.code == KeyCode::Esc {
                self.show_update_dialog = false;
            }
            return;
        }
        if self.update_done {
            match key.code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.show_update_dialog = false;
                }
                _ => {}
            }
            return;
        }
        match key.code {
            KeyCode::Enter => {
                self.start_self_update();
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.show_update_dialog = false;
            }
            _ => {}
        }
    }
    fn handle_password_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.show_password_modal = false;
                self.install_password.clear();
                self.is_installing = false;
                self.install_done = true;
                self.install_success = false;
                self.install_message =
                    tr!(self.translator, "dialog.password_cancelled").to_string();
                self.status_message = tr!(self.translator, "status.install_cancelled").to_string();
            }
            KeyCode::Enter if !self.install_password.is_empty() => {
                let password = std::mem::take(&mut self.install_password);
                self.show_password_modal = false;
                self.status_message = tr!(self.translator, "status.install_installing").to_string();
                crate::app::installation::spawn_install_with_password(
                    &mut self.install_child,
                    password,
                );
            }
            KeyCode::Backspace => {
                self.install_password.pop();
            }
            KeyCode::Char(c) => {
                self.install_password.push(c);
            }
            _ => {}
        }
    }
    fn handle_firewall_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.exit_firewall_mode();
            }
            KeyCode::Tab => {
                self.firewall_focus = FirewallManager::cycle_focus_forward(self.firewall_focus);
                self.firewall_action_index = 0;
            }
            KeyCode::BackTab => {
                self.firewall_focus = FirewallManager::cycle_focus_backward(self.firewall_focus);
                self.firewall_action_index = 0;
            }
            KeyCode::Up => self.firewall_scroll(-1),
            KeyCode::Down => self.firewall_scroll(1),
            KeyCode::Char(' ') => match self.firewall_focus {
                FirewallPanel::Connections => {
                    if let Some(checked) =
                        self.firewall_conn_checked.get_mut(self.firewall_conn_index)
                    {
                        *checked = !*checked;
                    }
                }
                FirewallPanel::BlockedList => {
                    if let Some(checked) = self
                        .firewall_blocked_checked
                        .get_mut(self.firewall_blocked_index)
                    {
                        *checked = !*checked;
                    }
                }
                FirewallPanel::Actions => {
                    self.toggle_selected_conn_checkbox();
                }
            },
            KeyCode::Enter => match self.firewall_focus {
                FirewallPanel::Connections => {
                    if let Some(checked) =
                        self.firewall_conn_checked.get_mut(self.firewall_conn_index)
                    {
                        *checked = !*checked;
                    }
                }
                FirewallPanel::BlockedList => {
                    if let Some(checked) = self
                        .firewall_blocked_checked
                        .get_mut(self.firewall_blocked_index)
                    {
                        *checked = !*checked;
                    }
                }
                FirewallPanel::Actions => {
                    self.execute_firewall_action();
                }
            },
            KeyCode::Char('b') | KeyCode::Char('B') => {
                self.firewall_action_index = 1;
                self.firewall_focus = FirewallPanel::Actions;
                self.execute_firewall_action();
            }
            KeyCode::Char('u') | KeyCode::Char('U') => {
                self.firewall_action_index = 2;
                self.firewall_focus = FirewallPanel::Actions;
                self.execute_firewall_action();
            }
            _ => {}
        }
    }
    fn firewall_scroll(&mut self, delta: i32) {
        match self.firewall_focus {
            FirewallPanel::Connections => {
                let max = self.firewall_connections.len().saturating_sub(1);
                self.firewall_conn_index = apply_scroll(self.firewall_conn_index, delta, max);
            }
            FirewallPanel::BlockedList => {
                let max = self.blocked_ips.len().saturating_sub(1);
                self.firewall_blocked_index = apply_scroll(self.firewall_blocked_index, delta, max);
            }
            FirewallPanel::Actions => {
                let max = FirewallManager::get_firewall_action_count();
                self.firewall_action_index = apply_scroll(self.firewall_action_index, delta, max);
            }
        }
    }
    pub fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        if self.show_language_modal
            || self.show_password_modal
            || self.show_nerdfont_dialog
            || self.show_install_dialog
            || self.show_confirmation
            || self.show_update_dialog
        {
            return;
        }
        if self.firewall_mode {
            match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    self.firewall_focus = FirewallManager::panel_from_x(mouse.column);
                }
                MouseEventKind::ScrollDown => {
                    self.firewall_scroll(1);
                }
                MouseEventKind::ScrollUp => {
                    self.firewall_scroll(-1);
                }
                _ => {}
            }
            return;
        }
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_dashboard_mouse_click(mouse.column, mouse.row);
            }
            MouseEventKind::ScrollDown => {
                self.handle_mouse_scroll(1);
            }
            MouseEventKind::ScrollUp => {
                self.handle_mouse_scroll(-1);
            }
            _ => {}
        }
    }
    fn handle_dashboard_mouse_click(&mut self, x: u16, _y: u16) {
        let (term_width, _) = crossterm::terminal::size()
            .unwrap_or((config::DEFAULT_TERM_WIDTH, config::DEFAULT_TERM_HEIGHT));
        
        let nav_width = if self.nav_sidebar_expanded { 20 } else { 7 };
        
        if x < nav_width {
            self.sidebar_focus = SidebarFocus::Nav;
            return;
        }

        let remaining_width = term_width.saturating_sub(nav_width);
        let left = nav_width + (remaining_width as f32 * config::SIDEBAR_LEFT_PCT as f32 / 100.0) as u16;
        let center = left + (remaining_width as f32 * config::CENTER_PANEL_PCT as f32 / 100.0) as u16;
        
        if x < left {
            self.sidebar_focus = SidebarFocus::Left;
        } else if x < center {
            self.sidebar_focus = SidebarFocus::Center;
        } else {
            self.sidebar_focus = SidebarFocus::Right;
        }
    }
    fn handle_mouse_scroll(&mut self, delta: i32) {
        match self.sidebar_focus {
            SidebarFocus::Nav => {
                // Scroll through nav views
                if delta > 0 {
                    self.current_nav_view = match self.current_nav_view {
                        NavView::Main => NavView::TrendGraphs,
                        NavView::TrendGraphs => NavView::DgaDetector,
                        NavView::DgaDetector => NavView::LibraryInspection,
                        NavView::LibraryInspection => NavView::Containers,
                        NavView::Containers => NavView::Main,
                    };
                } else {
                    self.current_nav_view = match self.current_nav_view {
                        NavView::Main => NavView::Containers,
                        NavView::TrendGraphs => NavView::Main,
                        NavView::DgaDetector => NavView::TrendGraphs,
                        NavView::LibraryInspection => NavView::DgaDetector,
                        NavView::Containers => NavView::LibraryInspection,
                    };
                }
            }
            SidebarFocus::Left => {
                if self.investigation_report.is_none() && !self.is_investigating {
                    let max = self.get_filtered_apps().len().saturating_sub(1);
                    if apply_scroll_bool(self.selected_app_index, delta, max) {
                        self.selected_app_index = apply_scroll(self.selected_app_index, delta, max);
                        self.trigger_geo_lookup_for_selected_app();
                    }
                }
            }
            SidebarFocus::Center => {
                if self.investigation_report.is_none() && !self.is_investigating {
                    if let Some(app) = self.get_selected_app() {
                        let max = app.connections.len().saturating_sub(1);
                        self.selected_connection_index =
                            apply_scroll(self.selected_connection_index, delta, max);
                    }
                }
            }
            SidebarFocus::Right => {
                let max = config::ACTION_COUNT;
                self.selected_action_index = apply_scroll(self.selected_action_index, delta, max);
            }
        }
    }
    pub fn execute_action(&mut self) {
        if self.investigation_report.is_some() {
            if self.selected_action_index == 0 {
                self.show_map = !self.show_map;
                self.selected_action_index = 0;
            }
            return;
        }
        match self.selected_action_index {
            0 => {
                self.analysis_paused = !self.analysis_paused;
                self.continuous_refresh_counter = 0;
                if self.analysis_paused {
                    self.status_message =
                        tr!(self.translator, "status.analysis_paused").to_string();
                } else {
                    self.status_message =
                        tr!(self.translator, "status.analysis_resumed").to_string();
                    self.start_batch_analysis();
                }
            }
            1 => {
                if let Some(app) = self.get_selected_app() {
                    self.confirmation_message = tr!(
                        self.translator,
                        "dialog.kill_process",
                        &app.process_name,
                        app.pid
                    );
                    self.show_confirmation = true;
                } else {
                    self.status_message = tr!(self.translator, "status.no_selection").to_string();
                }
            }
            2 => {
                if let Some(app) = self.get_selected_app() {
                    self.confirmation_message = tr!(
                        self.translator,
                        "dialog.kill_conns",
                        app.connections.len(),
                        &app.process_name
                    );
                    self.show_confirmation = true;
                } else {
                    self.status_message = tr!(self.translator, "status.no_selection").to_string();
                }
            }
            3 => {
                if let Some(app) = self.get_selected_app() {
                    let search_url = format!(
                        "{}{}",
                        resources::URLS.google_search_url,
                        urlencoding::encode(&app.process_name)
                    );
                    if let Err(e) = open::that(&search_url) {
                        self.status_message = tr!(self.translator, "status.browser_fail", e);
                    } else {
                        self.status_message = tr!(
                            self.translator,
                            "status.searching_online",
                            &app.process_name
                        );
                    }
                } else {
                    self.status_message = tr!(self.translator, "status.no_selection").to_string();
                }
            }
            4 => {
                if let Some(app) = self.get_selected_app() {
                    let path = app.process_path.clone();
                    match arboard::Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(&path) {
                            Ok(_) => {
                                self.status_message = tr!(self.translator, "status.copied", path);
                            }
                            Err(e) => {
                                self.status_message =
                                    tr!(self.translator, "status.clipboard_fail", e);
                            }
                        },
                        Err(e) => {
                            self.status_message =
                                tr!(self.translator, "status.clipboard_unavail", e);
                        }
                    }
                } else {
                    self.status_message = tr!(self.translator, "status.no_selection").to_string();
                }
            }
            5 => {
                self.export_to_json();
            }
            6 => {
                self.filter_high_risk_only = !self.filter_high_risk_only;
                self.selected_app_index = 0;
                if self.filter_high_risk_only {
                    self.status_message =
                        tr!(self.translator, "status.filter_high_risk").to_string();
                } else {
                    self.status_message = tr!(self.translator, "status.filter_all").to_string();
                }
            }
            7 => {
                self.enter_firewall_mode();
            }
            8 => {
                self.show_language_modal = true;
            }
            _ => {}
        }
    }
    fn execute_firewall_action(&mut self) {
        match self.firewall_action_index {
            0 => {
                self.toggle_selected_conn_checkbox();
            }
            1 => {
                let to_block: Vec<String> = self
                    .firewall_connections
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| self.firewall_conn_checked.get(*i).copied().unwrap_or(false))
                    .map(|(_, conn)| conn.foreign_address.clone())
                    .collect();
                if to_block.is_empty() {
                    self.status_message =
                        tr!(self.translator, "status.firewall_no_conns").to_string();
                    return;
                }
                let name = self.firewall_process_name.clone();
                let count = to_block.len();
                for ip in &to_block {
                    FirewallManager::block_ip(ip, &name, &self.database);
                }
                self.firewall_conn_checked = vec![false; self.firewall_connections.len()];
                self.status_message = tr!(self.translator, "status.firewall_blocked", count);
            }
            2 => {
                let to_unblock: Vec<String> = self
                    .blocked_ips
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| {
                        self.firewall_blocked_checked
                            .get(*i)
                            .copied()
                            .unwrap_or(false)
                    })
                    .map(|(_, (ip, _, _))| ip.clone())
                    .collect();
                if to_unblock.is_empty() {
                    self.status_message =
                        tr!(self.translator, "status.firewall_no_ips").to_string();
                    return;
                }
                let count = to_unblock.len();
                for ip in &to_unblock {
                    FirewallManager::unblock_ip(ip, &self.database);
                }
                self.refresh_blocked_ips();
                self.status_message = tr!(self.translator, "status.firewall_unblocked", count);
            }
            3 => {
                self.exit_firewall_mode();
            }
            _ => {}
        }
    }
    fn enter_firewall_mode(&mut self) {
        let selected = self
            .get_selected_app()
            .map(|a| (a.connections.clone(), a.process_name.clone()));
        if let Some((conns, name)) = selected {
            self.firewall_connections = conns;
            self.firewall_process_name = name;
            self.firewall_conn_index = 0;
            self.firewall_blocked_index = 0;
            self.firewall_action_index = 0;
            self.firewall_focus = FirewallPanel::Connections;
            self.firewall_conn_checked = vec![false; self.firewall_connections.len()];
            self.refresh_blocked_ips();
            self.firewall_blocked_checked = vec![false; self.blocked_ips.len()];
            self.firewall_mode = true;
            self.status_message = tr!(
                self.translator,
                "status.firewall_entered",
                self.firewall_connections.len(),
                self.blocked_ips.len()
            );
        } else {
            self.status_message = tr!(self.translator, "status.no_selection").to_string();
        }
    }
    fn exit_firewall_mode(&mut self) {
        self.firewall_mode = false;
        self.firewall_connections.clear();
        self.firewall_process_name.clear();
        self.blocked_ips.clear();
        self.firewall_conn_checked.clear();
        self.firewall_blocked_checked.clear();
        self.status_message = tr!(self.translator, "status.firewall_exited").to_string();
    }
    fn refresh_blocked_ips(&mut self) {
        if let Ok(ips) = self.database.get_blocked_ips() {
            self.blocked_ips = ips;
            self.firewall_blocked_checked = vec![false; self.blocked_ips.len()];
            if self.firewall_blocked_index >= self.blocked_ips.len().saturating_sub(1) {
                self.firewall_blocked_index = self.blocked_ips.len().saturating_sub(1);
            }
        }
    }
    pub fn export_to_json(&mut self) {
        use std::fs::File;
        use std::io::Write;
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let default_name = format!("network_analysis_{}.json", timestamp);
        let path = self
            .pick_save_path(&default_name)
            .unwrap_or_else(|| std::path::PathBuf::from(&default_name));
        match serde_json::to_string_pretty(&self.app_connections) {
            Ok(json) => match File::create(&path) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(json.as_bytes()) {
                        self.status_message = tr!(self.translator, "status.export_fail_write", e);
                    } else {
                        self.status_message = tr!(
                            self.translator,
                            "status.exported",
                            path.display().to_string()
                        );
                    }
                }
                Err(e) => {
                    self.status_message = tr!(self.translator, "status.export_fail_create", e);
                }
            },
            Err(e) => {
                self.status_message = tr!(self.translator, "status.export_fail_serialize", e);
            }
        }
    }
    fn pick_save_path(&self, default_name: &str) -> Option<std::path::PathBuf> {
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
            let output = Command::new("zenity")
                .args([
                    "--file-selection",
                    "--save",
                    "--title=Export Network Analysis",
                    &format!("--filename={}", default_name),
                ])
                .output()
                .ok()?;
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if path.is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(path))
            }
        }
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let script = format!(
                r#"return POSIX path of (choose file name with prompt "Export Network Analysis" default name "{}")"#,
                default_name
            );
            let output = Command::new("osascript")
                .args(["-e", &script])
                .output()
                .ok()?;
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if path.is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(path))
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            None
        }
    }
    pub fn toggle_selected_conn_checkbox(&mut self) {
        if let Some(checked) = self.firewall_conn_checked.get_mut(self.firewall_conn_index) {
            *checked = !*checked;
        }
    }
    pub fn any_conn_checked(&self) -> bool {
        self.firewall_conn_checked.iter().any(|&c| c)
    }
    pub fn any_blocked_checked(&self) -> bool {
        self.firewall_blocked_checked.iter().any(|&c| c)
    }

    fn handle_welcome_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left | KeyCode::Right => {
                self.welcome_index = if self.welcome_index == 0 { 1 } else { 0 };
            }
            KeyCode::Enter => {
                if self.welcome_index == 1 {
                    let _ = open::that(&resources::URLS.github_releases_page);
                }
                self.show_welcome_dialog = false;
            }
            KeyCode::Esc => {
                self.show_welcome_dialog = false;
            }
            _ => {}
        }
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
