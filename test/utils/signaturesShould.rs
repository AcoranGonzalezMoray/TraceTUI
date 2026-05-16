#[cfg(test)]
mod signatures_tests {
    use crate::utils::signatures::{SignatureStatus, SignatureVerifier};

    #[test]
    fn test_verify_empty_path_unknown() {
        assert_eq!(SignatureVerifier::verify(""), SignatureStatus::Unknown);
    }

    #[test]
    fn test_verify_unknown_path_returns_unknown_or_unsigned() {
        let status = SignatureVerifier::verify("/nonexistent/path.exe");
        let is_acceptable =
            status == SignatureStatus::Unknown || status == SignatureStatus::Unsigned;
        assert!(
            is_acceptable,
            "Expected Unknown or Unsigned, got {:?}",
            status
        );
    }

    #[test]
    fn test_signature_status_debug() {
        assert_eq!(format!("{:?}", SignatureStatus::Valid), "Valid");
        assert_eq!(format!("{:?}", SignatureStatus::Invalid), "Invalid");
        assert_eq!(format!("{:?}", SignatureStatus::Unsigned), "Unsigned");
        assert_eq!(format!("{:?}", SignatureStatus::Unknown), "Unknown");
    }

    #[test]
    fn test_signature_status_partial_eq() {
        assert_eq!(SignatureStatus::Valid, SignatureStatus::Valid);
        assert_eq!(SignatureStatus::Invalid, SignatureStatus::Invalid);
        assert_eq!(SignatureStatus::Unsigned, SignatureStatus::Unsigned);
        assert_eq!(SignatureStatus::Unknown, SignatureStatus::Unknown);
        assert_ne!(SignatureStatus::Valid, SignatureStatus::Invalid);
        assert_ne!(SignatureStatus::Unknown, SignatureStatus::Unsigned);
    }

    #[test]
    fn test_signature_status_serde() {
        let json = serde_json::to_string(&SignatureStatus::Valid).unwrap();
        let deserialized: SignatureStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, SignatureStatus::Valid);
    }

    #[test]
    fn test_verify_path_unknown_string() {
        let status = SignatureVerifier::verify("Unknown");
        assert_eq!(status, SignatureStatus::Unknown);
    }

    #[test]
    fn test_signature_verifier_caching() {
        let status1 = SignatureVerifier::verify("/cache/test.exe");
        let status2 = SignatureVerifier::verify("/cache/test.exe");
        assert_eq!(status1, status2);
    }

    #[test]
    fn test_signature_clone() {
        let status = SignatureStatus::Valid;
        let cloned = status.clone();
        assert_eq!(cloned, SignatureStatus::Valid);
    }
}
