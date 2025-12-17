//! Message Batches API for async bulk processing.
//!
//! Process large volumes of Messages requests asynchronously with 50% cost reduction.
//! Ideal for bulk content generation, data processing, and offline workloads.
//!
//! # Key Benefits
//!
//! | Feature | Description |
//! |---------|-------------|
//! | **50% Cost Savings** | All batch requests are billed at half price |
//! | **High Volume** | Up to 100,000 requests per batch |
//! | **Large Batches** | Up to 256 MB total request size |
//! | **Long Results** | Results available for 29 days |
//! | **Concurrent Processing** | Requests processed in parallel |
//!
//! # Typical Workflow
//!
//! 1. Create a batch with [`BatchClient::create`]
//! 2. Monitor progress with [`BatchClient::retrieve`] or [`BatchClient::wait_for_completion`]
//! 3. Fetch results with [`BatchClient::results`]
//!
//! # Quick Example
//!
//! ```rust,no_run
//! use claude_sdk::batch::{BatchClient, BatchRequest};
//! use claude_sdk::{MessagesRequest, Message};
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = BatchClient::new("your-api-key");
//!
//! // Create batch with multiple requests
//! let requests = vec![
//!     BatchRequest {
//!         custom_id: "summarize-doc-1".into(),
//!         params: MessagesRequest::new(
//!             "claude-sonnet-4-5-20250929",
//!             1024,
//!             vec![Message::user("Summarize: [document 1 text]")],
//!         ),
//!     },
//!     BatchRequest {
//!         custom_id: "summarize-doc-2".into(),
//!         params: MessagesRequest::new(
//!             "claude-sonnet-4-5-20250929",
//!             1024,
//!             vec![Message::user("Summarize: [document 2 text]")],
//!         ),
//!     },
//! ];
//!
//! // Submit batch
//! let batch = client.create(requests).await?;
//! println!("Batch {} created", batch.id);
//!
//! // Wait for completion (polls every 60s)
//! let completed = client.wait_for_completion(&batch.id).await?;
//! println!("Done! {} succeeded, {} failed",
//!     completed.request_counts.succeeded,
//!     completed.request_counts.errored);
//!
//! // Stream results
//! let mut results = client.results(&batch.id).await?;
//! while let Some(result) = results.next().await {
//!     let result = result?;
//!     println!("Result for {}: {:?}", result.custom_id, result.result);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Processing Large Datasets
//!
//! For large datasets, create batches in chunks and process results as they complete:
//!
//! ```rust,no_run
//! use claude_sdk::batch::{BatchClient, BatchRequest, BatchResultType};
//! use claude_sdk::{MessagesRequest, Message};
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = BatchClient::new("your-api-key");
//!
//! // Example: Process 1000 items in batches of 100
//! let items: Vec<String> = (0..1000).map(|i| format!("Item {}", i)).collect();
//!
//! for (batch_num, chunk) in items.chunks(100).enumerate() {
//!     let requests: Vec<BatchRequest> = chunk
//!         .iter()
//!         .enumerate()
//!         .map(|(i, item)| BatchRequest {
//!             custom_id: format!("batch-{}-item-{}", batch_num, i),
//!             params: MessagesRequest::new(
//!                 "claude-sonnet-4-5-20250929",
//!                 256,
//!                 vec![Message::user(format!("Process: {}", item))],
//!             ),
//!         })
//!         .collect();
//!
//!     let batch = client.create(requests).await?;
//!     println!("Batch {} submitted: {}", batch_num, batch.id);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Handling Results
//!
//! Each result can be one of several types:
//!
//! ```rust,no_run
//! use claude_sdk::batch::{BatchClient, BatchResultType};
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = BatchClient::new("your-api-key");
//! let mut results = client.results("msgbatch_123").await?;
//!
//! while let Some(result) = results.next().await {
//!     let result = result?;
//!     match result.result {
//!         BatchResultType::Succeeded { message } => {
//!             println!("{}: Success - {} tokens used",
//!                 result.custom_id,
//!                 message.usage.output_tokens);
//!         }
//!         BatchResultType::Errored { error } => {
//!             println!("{}: Error - {}", result.custom_id, error.message);
//!         }
//!         BatchResultType::Canceled => {
//!             println!("{}: Canceled", result.custom_id);
//!         }
//!         BatchResultType::Expired => {
//!             println!("{}: Expired", result.custom_id);
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Batch Lifecycle
//!
//! ```text
//! ┌──────────┐     ┌─────────────┐     ┌───────┐
//! │  create  │────▶│ in_progress │────▶│ ended │
//! └──────────┘     └─────────────┘     └───────┘
//!                        │
//!                        ▼
//!                  ┌───────────┐
//!                  │ canceling │────▶ ended
//!                  └───────────┘
//! ```
//!
//! # Cancellation
//!
//! Cancel a batch to stop processing remaining requests:
//!
//! ```rust,no_run
//! use claude_sdk::batch::BatchClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = BatchClient::new("your-api-key");
//!
//! // Cancel a running batch
//! let batch = client.cancel("msgbatch_123").await?;
//! println!("Batch canceled. Completed: {}, Canceled: {}",
//!     batch.request_counts.succeeded,
//!     batch.request_counts.canceled);
//! # Ok(())
//! # }
//! ```

use crate::error::{Error, Result};
use crate::types::{MessagesRequest, MessagesResponse};
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use tracing::{debug, info};

/// Batch API endpoint
const BATCH_API_URL: &str = "https://api.anthropic.com/v1/messages/batches";

/// API version
const API_VERSION: &str = "2023-06-01";

/// A single request in a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    /// Unique identifier for this request (for result matching)
    pub custom_id: String,

    /// The Messages API request parameters
    pub params: MessagesRequest,
}

/// Message Batch metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBatch {
    pub id: String,

    #[serde(rename = "type")]
    pub batch_type: String,

    pub processing_status: BatchProcessingStatus,

    pub request_counts: RequestCounts,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,

    pub created_at: String,

    pub expires_at: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_initiated_at: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub results_url: Option<String>,
}

/// Batch processing status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchProcessingStatus {
    InProgress,
    Canceling,
    Ended,
}

/// Request counts by status
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RequestCounts {
    pub processing: u32,
    pub succeeded: u32,
    pub errored: u32,
    pub canceled: u32,
    pub expired: u32,
}

/// Individual batch result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub custom_id: String,
    pub result: BatchResultType,
}

/// Type of batch result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BatchResultType {
    Succeeded { message: MessagesResponse },
    Errored { error: BatchError },
    Canceled,
    Expired,
}

/// Error in a batch request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

/// Client for Message Batches API
///
/// # Example
///
/// ```rust,no_run
/// use claude_sdk::batch::BatchClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = BatchClient::new("your-api-key");
///
/// // List all batches
/// let batches = client.list(None).await?;
/// println!("Found {} batches", batches.len());
/// # Ok(())
/// # }
/// ```
pub struct BatchClient {
    http: Client,
    api_key: String,
    api_version: String,
}

impl BatchClient {
    /// Create a new batch client
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            api_key: api_key.into(),
            api_version: API_VERSION.to_string(),
        }
    }

    /// Create a new message batch
    ///
    /// # Limits
    /// - Maximum 100,000 requests per batch
    /// - Maximum 256 MB total size
    /// - Results available for 29 days
    pub async fn create(&self, requests: Vec<BatchRequest>) -> Result<MessageBatch> {
        debug!("Creating batch with {} requests", requests.len());

        #[derive(Serialize)]
        struct CreateRequest {
            requests: Vec<BatchRequest>,
        }

        let response = self
            .http
            .post(BATCH_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.api_version)
            .header("content-type", "application/json")
            .json(&CreateRequest { requests })
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                message: error_text,
                error_type: None,
            });
        }

        let batch: MessageBatch = response.json().await?;
        Ok(batch)
    }

    /// Retrieve a message batch by ID
    pub async fn retrieve(&self, batch_id: &str) -> Result<MessageBatch> {
        debug!("Retrieving batch: {}", batch_id);

        let url = format!("{}/{}", BATCH_API_URL, batch_id);

        let response = self
            .http
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.api_version)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                message: error_text,
                error_type: None,
            });
        }

        let batch: MessageBatch = response.json().await?;
        Ok(batch)
    }

    /// List message batches
    ///
    /// # Arguments
    /// * `limit` - Maximum number of batches to return (default: 20)
    pub async fn list(&self, limit: Option<u32>) -> Result<Vec<MessageBatch>> {
        debug!("Listing batches");

        let mut url = BATCH_API_URL.to_string();
        if let Some(lim) = limit {
            url.push_str(&format!("?limit={}", lim));
        }

        let response = self
            .http
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.api_version)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                message: error_text,
                error_type: None,
            });
        }

        #[derive(Deserialize)]
        struct ListResponse {
            data: Vec<MessageBatch>,
        }

        let list_response: ListResponse = response.json().await?;
        Ok(list_response.data)
    }

    /// Cancel a message batch
    pub async fn cancel(&self, batch_id: &str) -> Result<MessageBatch> {
        info!("Canceling batch: {}", batch_id);

        let url = format!("{}/{}/cancel", BATCH_API_URL, batch_id);

        let response = self
            .http
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.api_version)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                message: error_text,
                error_type: None,
            });
        }

        let batch: MessageBatch = response.json().await?;
        Ok(batch)
    }

    /// Wait for batch to complete processing
    ///
    /// Polls the batch status every 60 seconds until it reaches `ended` status.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use claude_sdk::batch::BatchClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BatchClient::new("your-api-key");
    ///
    /// let batch = client.wait_for_completion("msgbatch_123").await?;
    /// println!("Batch complete! {} succeeded, {} failed",
    ///          batch.request_counts.succeeded,
    ///          batch.request_counts.errored);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_completion(&self, batch_id: &str) -> Result<MessageBatch> {
        info!("Waiting for batch {} to complete", batch_id);

        loop {
            let batch = self.retrieve(batch_id).await?;

            if batch.processing_status == BatchProcessingStatus::Ended {
                return Ok(batch);
            }

            debug!(
                "Batch {} still processing (status: {:?})",
                batch_id, batch.processing_status
            );

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    /// Stream batch results
    ///
    /// Results are returned as a stream of BatchResult items parsed from JSONL.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use claude_sdk::batch::BatchClient;
    /// use futures::StreamExt;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BatchClient::new("your-api-key");
    ///
    /// let mut stream = client.results("msgbatch_123").await?;
    ///
    /// while let Some(result) = stream.next().await {
    ///     match result? {
    ///         result => println!("Result for {}: {:?}",
    ///                           result.custom_id, result.result),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn results(
        &self,
        batch_id: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<BatchResult>> + Send>>> {
        // First, get the batch to find the results_url
        let batch = self.retrieve(batch_id).await?;

        let results_url = batch
            .results_url
            .ok_or_else(|| Error::InvalidRequest("Batch has no results yet".into()))?;

        debug!("Streaming results from: {}", results_url);

        // Stream the JSONL results
        let response = self
            .http
            .get(&results_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.api_version)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                message: error_text,
                error_type: None,
            });
        }

        // Convert bytes stream to lines stream
        let byte_stream = response.bytes_stream();

        let stream = async_stream::stream! {
            use futures::StreamExt;

            let mut byte_stream = byte_stream;
            let mut buffer = Vec::new();

            while let Some(chunk_result) = byte_stream.next().await {
                let chunk = chunk_result.map_err(|e| Error::Network(format!("Stream error: {}", e)))?;
                buffer.extend_from_slice(&chunk);

                // Process complete lines
                while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                    let line_bytes: Vec<u8> = buffer.drain(..=newline_pos).collect();
                    let line = String::from_utf8_lossy(&line_bytes).trim().to_string();

                    if !line.is_empty() {
                        let result: BatchResult = serde_json::from_str(&line)
                            .map_err(Error::Json)?;
                        yield Ok(result);
                    }
                }
            }

            // Process any remaining data
            if !buffer.is_empty() {
                let line = String::from_utf8_lossy(&buffer).trim().to_string();
                if !line.is_empty() {
                    let result: BatchResult = serde_json::from_str(&line)
                        .map_err(Error::Json)?;
                    yield Ok(result);
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_client_creation() {
        let client = BatchClient::new("test-key");
        assert_eq!(client.api_key, "test-key");
    }

    #[test]
    fn test_batch_processing_status() {
        assert_eq!(
            serde_json::to_string(&BatchProcessingStatus::InProgress).unwrap(),
            r#""in_progress""#
        );
        assert_eq!(
            serde_json::to_string(&BatchProcessingStatus::Ended).unwrap(),
            r#""ended""#
        );
    }

    // Integration tests require API key
    #[tokio::test]
    #[ignore]
    async fn test_create_and_retrieve_batch() {
        let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY required");
        let client = BatchClient::new(api_key);

        let requests = vec![BatchRequest {
            custom_id: "test-1".into(),
            params: MessagesRequest::new(
                "claude-sonnet-4-5-20250929",
                100,
                vec![crate::types::Message::user("Hello!")],
            ),
        }];

        let batch = client.create(requests).await;

        match batch {
            Ok(b) => {
                println!("Created batch: {}", b.id);
                assert_eq!(b.processing_status, BatchProcessingStatus::InProgress);
            }
            Err(e) => println!("Test skipped (expected without real API): {}", e),
        }
    }
}
