//! Retry logic with exponential backoff
//!
//! This module provides retry strategies for handling transient failures
//! like rate limits and server errors.

use crate::error::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Retry configuration for API requests
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Initial backoff duration
    pub initial_backoff: Duration,

    /// Maximum backoff duration
    pub max_backoff: Duration,

    /// Backoff multiplier (exponential growth)
    pub backoff_multiplier: f64,

    /// Whether to respect retry-after headers
    pub respect_retry_after: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            respect_retry_after: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum number of attempts
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Set initial backoff duration
    pub fn with_initial_backoff(mut self, duration: Duration) -> Self {
        self.initial_backoff = duration;
        self
    }

    /// Set maximum backoff duration
    pub fn with_max_backoff(mut self, duration: Duration) -> Self {
        self.max_backoff = duration;
        self
    }

    /// Set backoff multiplier for exponential growth
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Calculate backoff duration for a given attempt number
    fn calculate_backoff(&self, attempt: u32, retry_after: Option<u64>) -> Duration {
        // Respect retry-after header if present
        if self.respect_retry_after {
            if let Some(seconds) = retry_after {
                return Duration::from_secs(seconds).min(self.max_backoff);
            }
        }

        // Exponential backoff: initial * multiplier^attempt
        let backoff_secs =
            self.initial_backoff.as_secs_f64() * self.backoff_multiplier.powi(attempt as i32);

        Duration::from_secs_f64(backoff_secs.min(self.max_backoff.as_secs_f64()))
    }
}

/// Execute an async operation with retry logic
///
/// # Example
///
/// ```rust,no_run
/// use claude_sdk::retry::{retry_with_backoff, RetryConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = RetryConfig::new().with_max_attempts(5);
///
/// let result = retry_with_backoff(config, || async {
///     // Your API call here
///     Ok::<_, claude_sdk::Error>("success")
/// }).await?;
/// # Ok(())
/// # }
/// ```
pub async fn retry_with_backoff<F, Fut, T>(config: RetryConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;

    loop {
        attempt += 1;

        debug!("Attempt {}/{}", attempt, config.max_attempts);

        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!("Request succeeded after {} attempts", attempt);
                }
                return Ok(result);
            }
            Err(error) => {
                // Check if we should retry
                if !error.is_retryable() {
                    debug!("Error is not retryable: {:?}", error);
                    return Err(error);
                }

                // Check if we've exhausted retries
                if attempt >= config.max_attempts {
                    warn!(
                        "Max retry attempts ({}) reached, failing",
                        config.max_attempts
                    );
                    return Err(error);
                }

                // Calculate backoff
                let retry_after = error.retry_after();
                let backoff = config.calculate_backoff(attempt - 1, retry_after);

                warn!(
                    "Request failed (attempt {}/{}): {:?}. Retrying in {:?}",
                    attempt, config.max_attempts, error, backoff
                );

                // Sleep before retry
                sleep(backoff).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_default_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_backoff, Duration::from_millis(500));
        assert_eq!(config.max_backoff, Duration::from_secs(60));
    }

    #[test]
    fn test_config_builder() {
        let config = RetryConfig::new()
            .with_max_attempts(5)
            .with_initial_backoff(Duration::from_secs(1))
            .with_max_backoff(Duration::from_secs(30));

        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_backoff, Duration::from_secs(1));
        assert_eq!(config.max_backoff, Duration::from_secs(30));
    }

    #[test]
    fn test_calculate_backoff() {
        let config = RetryConfig::new()
            .with_initial_backoff(Duration::from_secs(1))
            .with_backoff_multiplier(2.0);

        // First retry: 1s
        assert_eq!(config.calculate_backoff(0, None), Duration::from_secs(1));

        // Second retry: 2s
        assert_eq!(config.calculate_backoff(1, None), Duration::from_secs(2));

        // Third retry: 4s
        assert_eq!(config.calculate_backoff(2, None), Duration::from_secs(4));
    }

    #[test]
    fn test_respect_retry_after() {
        let config = RetryConfig::new().with_initial_backoff(Duration::from_secs(1));

        // When retry_after is provided, use it instead of exponential backoff
        let backoff = config.calculate_backoff(0, Some(10));
        assert_eq!(backoff, Duration::from_secs(10));
    }

    #[test]
    fn test_max_backoff_cap() {
        let config = RetryConfig::new()
            .with_initial_backoff(Duration::from_secs(1))
            .with_max_backoff(Duration::from_secs(5))
            .with_backoff_multiplier(10.0);

        // Even with large multiplier, should cap at max_backoff
        let backoff = config.calculate_backoff(10, None);
        assert!(backoff <= Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_retry_succeeds_first_attempt() {
        let config = RetryConfig::new();
        let call_count = Arc::new(AtomicU32::new(0));
        let count = call_count.clone();

        let result = retry_with_backoff(config, || {
            let count = count.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Ok::<_, Error>("success")
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_failures() {
        let config = RetryConfig::new().with_initial_backoff(Duration::from_millis(10));
        let call_count = Arc::new(AtomicU32::new(0));
        let count = call_count.clone();

        let result = retry_with_backoff(config, || {
            let count = count.clone();
            async move {
                let current = count.fetch_add(1, Ordering::SeqCst) + 1;
                if current < 3 {
                    Err(Error::Server {
                        status: 503,
                        message: "Service unavailable".into(),
                    })
                } else {
                    Ok::<_, Error>("success")
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_fails_non_retryable() {
        let config = RetryConfig::new();
        let call_count = Arc::new(AtomicU32::new(0));
        let count = call_count.clone();

        let result = retry_with_backoff(config, || {
            let count = count.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Err::<String, _>(Error::Authentication("Bad key".into()))
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // Should not retry auth errors
    }

    #[tokio::test]
    async fn test_retry_exhausts_attempts() {
        let config = RetryConfig::new()
            .with_max_attempts(2)
            .with_initial_backoff(Duration::from_millis(10));
        let call_count = Arc::new(AtomicU32::new(0));
        let count = call_count.clone();

        let result = retry_with_backoff(config, || {
            let count = count.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Err::<String, _>(Error::Server {
                    status: 500,
                    message: "Error".into(),
                })
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }
}
