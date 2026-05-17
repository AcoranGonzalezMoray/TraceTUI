mod app;
mod config;
mod i18n;
mod resources;
mod services;
#[cfg(test)]
#[path = "../test/mod.rs"]
mod test_tests;
mod utils;
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
    app.perform_auto_analysis();
    app.check_for_updates();
    while !app.should_quit {
        terminal.draw(|f| render_ui(f, &app))?;

        let timeout = if !app.auto_analysis_complete {
            config::auto_analyze_sleep()
        } else {
            tick_rate.saturating_sub(last_tick.elapsed())
        };

        if crossterm::event::poll(timeout)? {
            match event::read()? {
                crossterm::event::Event::Key(key) => app.handle_key_event(key),
                crossterm::event::Event::Mouse(mouse) => app.handle_mouse_event(mouse),
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate || !app.auto_analysis_complete {
            app.on_tick();
            if app.auto_analysis_complete && last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
    }
    restore_terminal(&mut terminal)?;
    Ok(())
}
