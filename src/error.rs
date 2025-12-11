//! Error types for the Claude SDK

use thiserror::Error;

/// Result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for Claude SDK operations
#[derive(Error, Debug)]
pub enum Error {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// Failed to parse JSON response
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    /// API returned an error response
    #[error("API error ({status}): {message}")]
    Api {
        status: u16,
        message: String,
        error_type: Option<String>,
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded. Retry after: {retry_after:?}")]
    RateLimit {
        retry_after: Option<u64>,
        message: String,
    },

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Server error (5xx)
    #[error("Server error ({status}): {message}")]
    Server { status: u16, message: String },

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// SSE stream parsing error
    #[error("Stream parsing error: {0}")]
    StreamParse(String),
}

impl Error {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::RateLimit { .. } => true,
            Error::Server { status, .. } => *status >= 500,
            Error::Network(_) => true,
            _ => false,
        }
    }

    /// Get retry-after duration in seconds if available
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
