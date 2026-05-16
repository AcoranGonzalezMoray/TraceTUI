use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
#[derive(Debug, Clone)]
pub struct RateLimiter {
    max_requests: u32,
    window_duration: Duration,
    current_requests: Arc<AtomicU32>,
    window_start: Arc<RwLock<Instant>>,
    pending_requests: Arc<AtomicU32>,
}
impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_duration: Duration::from_secs(window_seconds),
            current_requests: Arc::new(AtomicU32::new(0)),
            window_start: Arc::new(RwLock::new(Instant::now())),
            pending_requests: Arc::new(AtomicU32::new(0)),
        }
    }
    pub async fn acquire_request(&self) -> Result<(), RateLimitError> {
        self.reset_if_needed().await;
        let current = self.current_requests.load(Ordering::Relaxed);
        if current >= self.max_requests {
            return Err(RateLimitError::LimitExceeded {
                current,
                max: self.max_requests,
                reset_in: self.time_until_reset().await,
            });
        }
        self.current_requests.fetch_add(1, Ordering::Relaxed);
        self.pending_requests.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    pub fn release_request(&self) {
        self.pending_requests.fetch_sub(1, Ordering::Relaxed);
    }
    async fn reset_if_needed(&self) {
        let window_start = self.window_start.read().await;
        if window_start.elapsed() >= self.window_duration {
            drop(window_start);
            let mut window_start = self.window_start.write().await;
            if window_start.elapsed() >= self.window_duration {
                self.current_requests.store(0, Ordering::Relaxed);
                *window_start = Instant::now();
            }
        }
    }
    async fn time_until_reset(&self) -> Duration {
        let window_start = self.window_start.read().await;
        let elapsed = window_start.elapsed();
        if elapsed >= self.window_duration {
            Duration::ZERO
        } else {
            self.window_duration - elapsed
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded: {current}/{max} requests. Reset in {reset_in:?}")]
    LimitExceeded {
        current: u32,
        max: u32,
        reset_in: Duration,
    },
}
