#[cfg(test)]
mod sidebar_right_tests {
    #[test]
    fn test_render_right_sidebar_fn_type() {
        let _: fn(&mut ratatui::Frame, &crate::app::App, ratatui::layout::Rect) = |_, _, _| {};
    }
}
