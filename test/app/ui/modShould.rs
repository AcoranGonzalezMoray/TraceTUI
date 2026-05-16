#[cfg(test)]
mod ui_mod_tests {
    #[test]
    fn test_render_ui_fn_type() {
        let _: fn(&mut ratatui::Frame, &crate::app::App) = |_, _| {};
    }
}
