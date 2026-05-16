use ratatui::style::Color;
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub text_main: Color,
    pub text_dim: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
}
pub const THEME: Theme = Theme {
    primary: Color::Rgb(100, 149, 237),
    secondary: Color::Rgb(119, 136, 153),
    accent: Color::Rgb(255, 165, 0),
    background: Color::Rgb(20, 20, 25),
    text_main: Color::Rgb(240, 248, 255),
    text_dim: Color::Rgb(169, 169, 169),
    success: Color::Rgb(50, 205, 50),
    warning: Color::Rgb(255, 215, 0),
    danger: Color::Rgb(220, 20, 60),
};
