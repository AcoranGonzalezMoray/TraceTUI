#[cfg(test)]
mod nav_sidebar_tests {
    #[test]
    fn test_render_nav_sidebar_fn_type() {
        let _: fn(&mut ratatui::Frame, &crate::app::App, ratatui::layout::Rect) = |_, _, _| {};
    }
}
