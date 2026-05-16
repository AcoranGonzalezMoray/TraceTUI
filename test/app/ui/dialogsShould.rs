#[cfg(test)]
mod dialogs_tests {
    #[test]
    fn test_confirmation_dialog_fn_type() {
        let _: fn(&mut ratatui::Frame, &crate::app::App) = |_, _| {};
    }

    #[test]
    fn test_install_dialog_fn_type() {
        let _: fn(&mut ratatui::Frame, &crate::app::App) = |_, _| {};
    }

    #[test]
    fn test_language_modal_fn_type() {
        let _: fn(&mut ratatui::Frame, &crate::app::App) = |_, _| {};
    }

    #[test]
    fn test_nerdfont_dialog_fn_type() {
        let _: fn(&mut ratatui::Frame, &crate::app::App) = |_, _| {};
    }

    #[test]
    fn test_password_modal_fn_type() {
        let _: fn(&mut ratatui::Frame, &crate::app::App) = |_, _| {};
    }
}
