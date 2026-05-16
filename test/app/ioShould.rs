#[cfg(test)]
mod io_tests {
    use ratatui::backend::CrosstermBackend;
    use ratatui::Terminal;
    use std::io::Result;

    #[test]
    fn test_setup_terminal_fn_type() {
        let _: fn() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> =
            || Ok(Terminal::new(CrosstermBackend::new(std::io::stdout())).unwrap());
    }

    #[test]
    fn test_restore_terminal_fn_type() {
        let _: fn(&mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> = |_| Ok(());
    }
}
