#[cfg(test)]
mod installation_tests {
    #[test]
    fn test_make_failed_output_contains_msg() {
        let msg = b"test error message";
        let output = crate::app::installation::make_failed_output(msg);
        assert!(!output.status.success());
        assert_eq!(output.stderr, msg);
    }

    #[test]
    fn test_make_failed_output_empty_msg() {
        let output = crate::app::installation::make_failed_output(b"");
        assert!(!output.status.success());
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn test_make_failed_output_unicode_msg() {
        let msg = "error: séñal".as_bytes();
        let output = crate::app::installation::make_failed_output(msg);
        assert!(!output.status.success());
        assert_eq!(output.stderr, msg);
    }

    #[test]
    fn test_make_failed_output_stdout_empty() {
        let output = crate::app::installation::make_failed_output(b"error");
        assert!(output.stdout.is_empty());
    }
}
