#[cfg(test)]
mod nav_sidebar_tests {
    use crate::app::ui::nav_sidebar::nav_item_area;
    use ratatui::layout::Rect;

    #[test]
    fn test_render_nav_sidebar_fn_type() {
        let _: fn(&mut ratatui::Frame, &crate::app::App, ratatui::layout::Rect) = |_, _, _| {};
    }

    #[test]
    fn nav_item_area_keeps_all_six_items_visible_when_height_is_tight() {
        let inner = Rect::new(0, 0, 5, 9);
        let mut max_bottom = 0u16;

        for index in 0..6 {
            let area = nav_item_area(inner, index, 6);
            assert!(
                area.height > 0,
                "item {index} should remain visible in a 9-row sidebar"
            );
            assert!(area.y + area.height <= inner.y + inner.height);
            max_bottom = max_bottom.max(area.y + area.height);
        }

        assert_eq!(max_bottom, inner.y + inner.height);
    }

    #[test]
    fn nav_item_area_uses_full_height_for_six_items() {
        let inner = Rect::new(0, 0, 5, 11);
        let last = nav_item_area(inner, 5, 6);

        assert_eq!(last.height, 1);
        assert_eq!(last.y + last.height, inner.y + inner.height);
    }

    #[test]
    fn nav_item_area_shows_maximum_icons_when_height_is_below_item_count() {
        let inner = Rect::new(0, 0, 5, 5);
        let visible = (0..6)
            .map(|index| nav_item_area(inner, index, 6))
            .filter(|area| area.height > 0)
            .count();

        assert_eq!(visible, 5);
    }

    #[test]
    fn nav_item_area_fits_all_six_one_line_items_without_gaps() {
        let inner = Rect::new(0, 0, 5, 6);
        for index in 0..6 {
            let area = nav_item_area(inner, index, 6);
            assert_eq!(area.height, 1, "item {index} should get one row");
            assert_eq!(area.y, index as u16);
        }
    }
}
