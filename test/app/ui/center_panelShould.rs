#[cfg(test)]
mod center_panel_tests {
    #[test]
    fn test_render_center_panel_fn_type() {
        let _: fn(&mut ratatui::Frame, &crate::app::App, ratatui::layout::Rect) = |_, _, _| {};
    }
}
