//! Error types for the Claude SDK.
//!
//! This module provides comprehensive error handling for all Claude API operations,
//! with support for retryable errors and rate limit information.
//!
//! # Error Types
//!
//! The main error enum covers all possible failure modes:
//!
//! | Error | Description | Retryable |
//! |-------|-------------|-----------|
//! | [`Error::Api`] | API returned an error response | No |
//! | [`Error::RateLimit`] | Rate limit exceeded (429) | Yes |
//! | [`Error::Server`] | Server error (5xx) | Yes |
//! | [`Error::Network`] | Connection/network failure | Yes |
//! | [`Error::Authentication`] | Invalid API key | No |
//! | [`Error::InvalidRequest`] | Malformed request | No |
//! | [`Error::Http`] | HTTP client error | Depends |
//! | [`Error::Json`] | JSON serialization error | No |
//! | [`Error::StreamParse`] | SSE parsing error | No |
//!
//! # Example: Basic Error Handling
//!
//! ```rust,no_run
//! use claude_sdk::{ClaudeClient, MessagesRequest, Message, Error};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ClaudeClient::anthropic(std::env::var("ANTHROPIC_API_KEY")?);
//!
//! let request = MessagesRequest::new(
//!     "claude-sonnet-4-5-20250929",
//!     1024,
//!     vec![Message::user("Hello!")],
//! );
//!
//! match client.send_message(request).await {
//!     Ok(response) => {
//!         println!("Success: {:?}", response.content);
//!     }
//!     Err(Error::RateLimit { retry_after, message }) => {
//!         println!("Rate limited: {}", message);
//!         if let Some(seconds) = retry_after {
//!             println!("Retry after {} seconds", seconds);
//!         }
//!     }
//!     Err(Error::Authentication(msg)) => {
//!         println!("Auth failed: {} - check your API key", msg);
//!     }
//!     Err(Error::Api { status, message, .. }) => {
//!         println!("API error ({}): {}", status, message);
//!     }
//!     Err(e) if e.is_retryable() => {
//!         println!("Retryable error: {}", e);
//!     }
//!     Err(e) => {
//!         println!("Fatal error: {}", e);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Retry with Backoff
//!
//! ```rust,no_run
//! use claude_sdk::{ClaudeClient, MessagesRequest, Message, Error};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ClaudeClient::anthropic(std::env::var("ANTHROPIC_API_KEY")?);
//! let request = MessagesRequest::new(
//!     "claude-sonnet-4-5-20250929",
//!     1024,
//!     vec![Message::user("Hello!")],
//! );
//!
//! // Simple retry loop
//! let mut attempts = 0;
//! let max_attempts = 3;
//!
//! loop {
//!     match client.send_message(request.clone()).await {
//!         Ok(response) => {
//!             println!("Success!");
//!             break;
//!         }
//!         Err(ref e) if e.is_retryable() && attempts < max_attempts => {
//!             attempts += 1;
//!             let delay = e.retry_after().unwrap_or(2u64.pow(attempts));
//!             println!("Retrying in {}s (attempt {})", delay, attempts);
//!             tokio::time::sleep(Duration::from_secs(delay)).await;
//!         }
//!         Err(e) => return Err(e.into()),
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Using the Built-in Retry Logic
//!
//! For production use, prefer the `send_message_with_retry` method on `ClaudeClient` or
//! the [`retry`](crate::retry) module:
//!
//! ```rust,no_run
//! use claude_sdk::{ClaudeClient, MessagesRequest, Message};
//! use claude_sdk::retry::RetryConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ClaudeClient::anthropic(std::env::var("ANTHROPIC_API_KEY")?);
//!
//! let request = MessagesRequest::new(
//!     "claude-sonnet-4-5-20250929",
//!     1024,
//!     vec![Message::user("Hello!")],
//! );
//!
//! let config = RetryConfig::default()
//!     .with_max_retries(3)
//!     .with_initial_delay_ms(1000);
//!
//! let response = client.send_message_with_retry(request, &config).await?;
//! # Ok(())
//! # }
//! ```

use thiserror::Error;

/// Result type alias using the SDK's error type.
///
/// This is the standard return type for all fallible SDK operations.
///
/// # Example
///
/// ```rust
/// use claude_sdk::error::Result;
/// use claude_sdk::MessagesResponse;
///
/// async fn my_function() -> Result<MessagesResponse> {
///     // ... implementation
///     # todo!()
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for Claude SDK operations.
///
/// This enum covers all possible error conditions when using the SDK.
/// Use [`Error::is_retryable`] to check if an error can be retried.
///
/// # Example
///
/// ```rust,no_run
/// use claude_sdk::Error;
///
/// fn handle_error(err: Error) {
///     if err.is_retryable() {
///         println!("Can retry after {:?}s", err.retry_after());
///     } else {
///         println!("Fatal error: {}", err);
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum Error {
    /// HTTP request failed.
    ///
    /// This wraps errors from the underlying HTTP client (reqwest).
    /// May be retryable depending on the specific error.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// Failed to parse JSON response.
    ///
    /// This typically indicates a bug or API contract change.
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    /// API returned an error response.
    ///
    /// Contains the HTTP status code and error message from the API.
    /// Check `error_type` for specific error categories like "invalid_request_error".
    #[error("API error ({status}): {message}")]
    Api {
        /// HTTP status code
        status: u16,
        /// Error message from the API
        message: String,
        /// Error type (e.g., "invalid_request_error", "authentication_error")
        error_type: Option<String>,
    },

    /// Rate limit exceeded (HTTP 429).
    ///
    /// The API has rate-limited your request. Check `retry_after` for the
    /// recommended wait time before retrying.
    #[error("Rate limit exceeded. Retry after: {retry_after:?}")]
    RateLimit {
        /// Seconds to wait before retrying (from Retry-After header)
        retry_after: Option<u64>,
        /// Error message from the API
        message: String,
    },

    /// Invalid request.
    ///
    /// The request was malformed or missing required fields.
    /// This is not retryable - fix the request before trying again.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Authentication failed.
    ///
    /// The API key is invalid, expired, or missing required permissions.
    /// Not retryable - verify your API key configuration.
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Server error (HTTP 5xx).
    ///
    /// The API server encountered an internal error. These are typically
    /// transient and can be retried with exponential backoff.
    #[error("Server error ({status}): {message}")]
    Server {
        /// HTTP status code (500-599)
        status: u16,
        /// Error message
        message: String,
    },

    /// Network error.
    ///
    /// Connection failed, timed out, or was interrupted.
    /// Usually retryable after a brief delay.
    #[error("Network error: {0}")]
    Network(String),

    /// SSE stream parsing error.
    ///
    /// Failed to parse a server-sent event during streaming.
    /// May indicate a malformed response or connection issue.
    #[error("Stream parsing error: {0}")]
    StreamParse(String),
}

impl Error {
    /// Check if this error is retryable.
    ///
    /// Returns `true` for transient errors that may succeed on retry:
    /// - Rate limits (with backoff)
    /// - Server errors (5xx)
    /// - Network errors
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::Error;
    ///
    /// fn should_retry(err: &Error) -> bool {
    ///     err.is_retryable()
    /// }
    /// ```
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::RateLimit { .. } => true,
            Error::Server { status, .. } => *status >= 500,
            Error::Network(_) => true,
            _ => false,
        }
    }

    /// Get retry-after duration in seconds if available.
    ///
    /// Only returns a value for [`Error::RateLimit`] errors that include
    /// a Retry-After header from the API.
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::Error;
    /// use std::time::Duration;
    ///
    /// async fn wait_and_retry(err: &Error) {
    ///     if let Some(seconds) = err.retry_after() {
    ///         tokio::time::sleep(Duration::from_secs(seconds)).await;
    ///     }
    /// }
    /// ```
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Error::RateLimit { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

/// API error response structure
#[derive(Debug, serde::Deserialize)]
pub struct ApiErrorResponse {
    #[serde(rename = "type")]
    pub error_type: String,
    pub error: ApiErrorDetail,
}

#[derive(Debug, serde::Deserialize)]
pub struct ApiErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}
