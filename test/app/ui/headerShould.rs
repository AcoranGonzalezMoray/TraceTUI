#[cfg(test)]
mod header_tests {
    #[test]
    fn test_render_header_fn_type() {
        let _: fn(&mut ratatui::Frame, &crate::app::App, ratatui::layout::Rect) = |_, _, _| {};
    }
}
