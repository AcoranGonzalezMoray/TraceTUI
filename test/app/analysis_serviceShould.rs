#[cfg(test)]
mod analysis_service_tests {
    use crate::app::services::analysis_service::is_newer;

    #[test]
    fn test_is_newer_same_version() {
        assert!(!is_newer("v1.0.0", "1.0.0"));
    }

    #[test]
    fn test_is_newer_local_newer() {
        assert!(!is_newer("2.0.0", "1.0.0"));
    }

    #[test]
    fn test_is_newer_remote_newer() {
        assert!(is_newer("1.0.0", "2.0.0"));
    }

    #[test]
    fn test_is_newer_remote_major() {
        assert!(is_newer("1.0.0", "2.0.0"));
    }

    #[test]
    fn test_is_newer_remote_minor() {
        assert!(is_newer("1.0.0", "1.1.0"));
    }

    #[test]
    fn test_is_newer_remote_patch() {
        assert!(is_newer("1.0.0", "1.0.1"));
    }

    #[test]
    fn test_is_newer_without_v_prefix() {
        assert!(is_newer("1.0.0", "1.0.1"));
    }

    #[test]
    fn test_is_newer_empty_remote() {
        assert!(!is_newer("1.0.0", ""));
    }

    #[test]
    fn test_is_newer_invalid_remote() {
        assert!(!is_newer("1.0.0", "not-a-version"));
    }
}
