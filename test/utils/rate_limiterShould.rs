#[cfg(test)]
mod rate_limiter_tests {
    use crate::utils::rate_limiter::{RateLimitError, RateLimiter};
    use tokio;

    #[tokio::test]
    async fn test_acquire_within_limit() {
        let limiter = RateLimiter::new(10, 60);
        let result = limiter.acquire_request().await;
        assert!(result.is_ok());
        limiter.release_request();
    }

    #[tokio::test]
    async fn test_acquire_multiple_within_limit() {
        let limiter = RateLimiter::new(5, 60);
        for _ in 0..5 {
            let result = limiter.acquire_request().await;
            assert!(result.is_ok());
        }
        for _ in 0..5 {
            limiter.release_request();
        }
    }

    #[tokio::test]
    async fn test_rate_limit_exceeded() {
        let limiter = RateLimiter::new(2, 60);
        assert!(limiter.acquire_request().await.is_ok());
        assert!(limiter.acquire_request().await.is_ok());
        let result = limiter.acquire_request().await;
        assert!(result.is_err());
        match result {
            Err(RateLimitError::LimitExceeded { current, max, .. }) => {
                assert_eq!(current, 2);
                assert_eq!(max, 2);
            }
            _ => panic!("Expected LimitExceeded"),
        }
    }

    #[tokio::test]
    async fn test_release_and_retry() {
        let limiter = RateLimiter::new(1, 60);
        assert!(limiter.acquire_request().await.is_ok());
        let result = limiter.acquire_request().await;
        assert!(result.is_err());
        limiter.release_request();
    }
}
