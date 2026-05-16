#[cfg(test)]
mod main_tests {
    #[test]
    fn test_module_declarations_compile() {
        let _ = (crate::config::TICK_RATE_MS, crate::app::App::new());
    }
}
