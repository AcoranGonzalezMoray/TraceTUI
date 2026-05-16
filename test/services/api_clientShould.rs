#[cfg(test)]
mod api_client_tests {
    use crate::services::api_client::{ApiClient, ApiError};

    #[test]
    fn test_api_client_new() {
        let client = ApiClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_api_error_display_request() {
        let err = ApiError::RateLimit("too many".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Rate limit exceeded"));
        assert!(msg.contains("too many"));
    }

    #[test]
    fn test_api_error_debug() {
        let err = ApiError::RateLimit("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("RateLimit"));
    }

    #[test]
    fn test_api_client_clone() {
        let client = ApiClient::new().unwrap();
        let _cloned = client.clone();
    }
}
