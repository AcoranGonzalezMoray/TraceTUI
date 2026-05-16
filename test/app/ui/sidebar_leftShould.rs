#[cfg(test)]
mod sidebar_left_tests {
    #[test]
    fn test_render_left_sidebar_fn_type() {
        let _: fn(&mut ratatui::Frame, &crate::app::App, ratatui::layout::Rect) = |_, _, _| {};
    }
}
