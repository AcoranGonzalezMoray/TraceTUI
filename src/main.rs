mod app;
mod config;
mod i18n;
mod resources;
mod services;
#[cfg(test)]
#[path = "../test/mod.rs"]
mod test_tests;
mod utils;
use app::services::analysis_service;
use app::services::input_service;
use app::ui::render_ui;
use app::{restore_terminal, setup_terminal, App};
use crossterm::event;
use std::io::Result;
use std::time::Instant;
#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    let mut last_tick = Instant::now();
    let tick_rate = config::tick_rate();
    analysis_service::perform_auto_analysis(&mut app);
    analysis_service::check_for_updates(&mut app);
    while !app.ui.should_quit {
        terminal.draw(|f| render_ui(f, &app))?;

        let timeout = if !app.ui.auto_analysis_complete {
            config::auto_analyze_sleep()
        } else {
            tick_rate.saturating_sub(last_tick.elapsed())
        };

        if crossterm::event::poll(timeout)? {
            match event::read()? {
                crossterm::event::Event::Key(key) => input_service::handle_key_event(&mut app, key),
                crossterm::event::Event::Mouse(mouse) => input_service::handle_mouse_event(&mut app, mouse),
                crossterm::event::Event::Resize(_, _) => {
                    app.ui.needs_clear = true;
                }
                _ => {}
            }
        }

        if app.ui.needs_clear {
            terminal.clear()?;
            app.ui.needs_clear = false;
        }

        if last_tick.elapsed() >= tick_rate || !app.ui.auto_analysis_complete {
            analysis_service::on_tick(&mut app);
            if app.ui.auto_analysis_complete && last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
    }
    restore_terminal(&mut terminal)?;
    Ok(())
}
