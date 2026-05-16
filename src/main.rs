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
use std::time::{Duration, Instant};
#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    let mut last_tick = Instant::now();
    let tick_rate = config::tick_rate();
    app.perform_auto_analysis();
    app.check_for_updates();
    while !app.auto_analysis_complete {
        terminal.draw(|f| render_ui(f, &app))?;
        app.on_tick();
        std::thread::sleep(config::auto_analyze_sleep());
    }
    loop {
        terminal.draw(|f| render_ui(f, &app))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            match event::read()? {
                crossterm::event::Event::Key(key) => app.handle_key_event(key),
                crossterm::event::Event::Mouse(mouse) => app.handle_mouse_event(mouse),
                _ => {}
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
        if app.should_quit {
            break;
        }
    }
    restore_terminal(&mut terminal)?;
    Ok(())
}
