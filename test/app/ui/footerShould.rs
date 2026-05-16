#[cfg(test)]
mod footer_tests {
    #[test]
    fn test_render_footer_fn_type() {
        let _: fn(&mut ratatui::Frame, &crate::app::App, ratatui::layout::Rect) = |_, _, _| {};
    }
}
