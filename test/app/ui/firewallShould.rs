#[cfg(test)]
mod firewall_tests {
    #[test]
    fn test_render_firewall_mode_fn_type() {
        let _: fn(&mut ratatui::Frame, &crate::app::App, ratatui::layout::Rect) = |_, _, _| {};
    }
}
