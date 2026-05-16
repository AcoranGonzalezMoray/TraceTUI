#[cfg(test)]
mod widgets_tests {
    #[test]
    fn test_render_scrollbar_fn_type() {
        let _: fn(&mut ratatui::Frame, ratatui::layout::Rect, usize, usize) = |_, _, _, _| {};
    }
}
