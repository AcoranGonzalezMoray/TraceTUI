#[cfg(test)]
mod theme_tests {
    use crate::app::ui::theme::{Theme, THEME};
    use ratatui::style::Color;

    #[test]
    fn test_theme_primary_color() {
        assert_eq!(THEME.primary, Color::Rgb(100, 149, 237));
    }

    #[test]
    fn test_theme_background_color() {
        assert_eq!(THEME.background, Color::Rgb(20, 20, 25));
    }

    #[test]
    fn test_theme_success_color() {
        assert_eq!(THEME.success, Color::Rgb(50, 205, 50));
    }

    #[test]
    fn test_theme_warning_color() {
        assert_eq!(THEME.warning, Color::Rgb(255, 215, 0));
    }

    #[test]
    fn test_theme_danger_color() {
        assert_eq!(THEME.danger, Color::Rgb(220, 20, 60));
    }

    #[test]
    fn test_theme_struct_public() {
        let _theme = Theme {
            primary: Color::White,
            secondary: Color::Black,
            accent: Color::Red,
            background: Color::Black,
            text_main: Color::White,
            text_dim: Color::Gray,
            success: Color::Green,
            warning: Color::Yellow,
            danger: Color::Red,
        };
    }
}
