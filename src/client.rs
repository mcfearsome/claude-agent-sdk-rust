//! Claude API client implementation

use crate::error::{ApiErrorResponse, Error, Result};
use crate::streaming::StreamEvent;
use crate::types::{MessagesRequest, MessagesResponse};
use eventsource_stream::Eventsource;
use futures::{Stream, StreamExt, TryStreamExt};
use reqwest::{Client, StatusCode};
use std::pin::Pin;
use tracing::{debug, instrument};

#[cfg(feature = "bedrock")]
use aws_sdk_bedrockruntime::Client as BedrockClient;

/// API endpoint for Anthropic
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Current API version
const API_VERSION: &str = "2023-06-01";

/// Backend for Claude API
pub enum ClaudeBackend {
    /// Anthropic API with API key
    Anthropic { api_key: String },

    /// AWS Bedrock with Bedrock runtime client
    #[cfg(feature = "bedrock")]
    Bedrock {
        region: String,
        bedrock_client: BedrockClient,
    },
}

/// Claude API client
///
/// This client can connect to either the Anthropic API directly or AWS Bedrock.
///
/// # Example - Anthropic API
///
/// ```rust,no_run
/// use claude_sdk::ClaudeClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = ClaudeClient::anthropic(
///         std::env::var("ANTHROPIC_API_KEY")?
///     );
///     Ok(())
/// }
/// ```
///
/// # Example - AWS Bedrock
///
/// ```rust,no_run
/// # #[cfg(feature = "bedrock")]
/// use claude_sdk::ClaudeClient;
///
/// # #[cfg(feature = "bedrock")]
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = ClaudeClient::bedrock("us-east-1").await?;
///     Ok(())
/// }
/// ```
pub struct ClaudeClient {
    http: Client,
    backend: ClaudeBackend,
    api_version: String,
}

impl ClaudeClient {
    /// Create a new client for the Anthropic API
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use claude_sdk::ClaudeClient;
    ///
    /// let client = ClaudeClient::anthropic("your-api-key");
    /// ```
    pub fn anthropic(api_key: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            backend: ClaudeBackend::Anthropic {
                api_key: api_key.into(),
            },
            api_version: API_VERSION.to_string(),
        }
    }

    /// Create a new client for AWS Bedrock
    ///
    /// This loads AWS credentials from the environment (AWS_PROFILE, AWS_ACCESS_KEY_ID, etc.)
    /// using the standard AWS credential chain.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "bedrock")]
    /// use claude_sdk::ClaudeClient;
    ///
    /// # #[cfg(feature = "bedrock")]
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Uses AWS_PROFILE or default credential chain
    ///     let client = ClaudeClient::bedrock("us-east-1").await?;
    ///     Ok(())
    /// }
    /// ```
    #[cfg(feature = "bedrock")]
    pub async fn bedrock(region: impl Into<String>) -> Result<Self> {
        let region = region.into();

        // Load AWS config with specified region
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let bedrock_config = aws_sdk_bedrockruntime::config::Builder::from(&config)
            .region(aws_sdk_bedrockruntime::config::Region::new(region.clone()))
            .build();

        // Create Bedrock runtime client
        let bedrock_client = BedrockClient::from_conf(bedrock_config);

        Ok(Self {
            http: Client::new(),
            backend: ClaudeBackend::Bedrock {
                region,
                bedrock_client,
            },
            api_version: API_VERSION.to_string(),
        })
    }

    /// Send a message and get a complete response
    ///
    /// This is the non-streaming API. For streaming responses, use `send_streaming()`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use claude_sdk::{ClaudeClient, MessagesRequest, Message};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClaudeClient::anthropic("your-api-key");
    ///
    /// let request = MessagesRequest::new(
    ///     "claude-3-5-sonnet-20241022",
    ///     1024,
    ///     vec![Message::user("Hello, Claude!")],
    /// );
    ///
    /// let response = client.send_message(request).await?;
    /// println!("Response: {:?}", response.content);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, request), fields(model = %request.model))]
    pub async fn send_message(&self, request: MessagesRequest) -> Result<MessagesResponse> {
        match &self.backend {
            ClaudeBackend::Anthropic { .. } => self.send_anthropic(request).await,
            #[cfg(feature = "bedrock")]
            ClaudeBackend::Bedrock { .. } => self.send_bedrock(request).await,
        }
    }

    /// Send message to Anthropic API
    async fn send_anthropic(&self, request: MessagesRequest) -> Result<MessagesResponse> {
        let api_key = match &self.backend {
            ClaudeBackend::Anthropic { api_key } => api_key,
            #[allow(unreachable_patterns)]
            _ => unreachable!("send_anthropic called with non-Anthropic backend"),
        };

        debug!("Sending message to Anthropic API");

        // Ensure stream is not set or is false
        let mut request = request;
        request.stream = Some(false);

        let response = self
            .http
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", api_key)
            .header("anthropic-version", &self.api_version)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        debug!("Received response with status: {}", status);

        // Handle different status codes
        match status {
            StatusCode::OK => {
                let messages_response: MessagesResponse = response.json().await?;
                Ok(messages_response)
            }
            StatusCode::TOO_MANY_REQUESTS => {
                // Parse retry-after header if present
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse().ok());

                let error_body = response.text().await.unwrap_or_default();
                Err(Error::RateLimit {
                    retry_after,
                    message: error_body,
                })
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                let error_body = response.text().await.unwrap_or_default();
                Err(Error::Authentication(error_body))
            }
            StatusCode::BAD_REQUEST => {
                // Try to parse structured error
                let error_text = response.text().await.unwrap_or_default();
                if let Ok(api_error) = serde_json::from_str::<ApiErrorResponse>(&error_text) {
                    Err(Error::Api {
                        status: status.as_u16(),
                        message: api_error.error.message,
                        error_type: Some(api_error.error.error_type),
                    })
                } else {
                    Err(Error::InvalidRequest(error_text))
                }
            }
            _ if status.is_server_error() => {
                let error_body = response.text().await.unwrap_or_default();
                Err(Error::Server {
                    status: status.as_u16(),
                    message: error_body,
                })
            }
            _ => {
                let error_body = response.text().await.unwrap_or_default();
                Err(Error::Api {
                    status: status.as_u16(),
                    message: error_body,
                    error_type: None,
                })
            }
        }
    }

    /// Send message to AWS Bedrock
    #[cfg(feature = "bedrock")]
    async fn send_bedrock(&self, request: MessagesRequest) -> Result<MessagesResponse> {
        let (bedrock_client, model_id) = match &self.backend {
            ClaudeBackend::Bedrock { bedrock_client, .. } => {
                let model_id = self.get_bedrock_model_id(&request.model)?;
                (bedrock_client, model_id)
            }
            _ => unreachable!("send_bedrock called with non-Bedrock backend"),
        };

        debug!("Sending message to AWS Bedrock");

        // Serialize request to JSON
        let body = serde_json::to_string(&request)?;

        // Use Bedrock runtime client
        let response = bedrock_client
            .invoke_model()
            .model_id(&model_id)
            .content_type("application/json")
            .body(aws_sdk_bedrockruntime::primitives::Blob::new(
                body.as_bytes(),
            ))
            .send()
            .await
            .map_err(|e| Error::Network(format!("Bedrock API call failed: {}", e)))?;

        // Parse response body
        let response_bytes = response.body().as_ref();
        let messages_response: MessagesResponse = serde_json::from_slice(response_bytes)?;

        Ok(messages_response)
    }

    /// Get Bedrock model ID for a given model string
    #[cfg(feature = "bedrock")]
    fn get_bedrock_model_id(&self, model: &str) -> Result<String> {
        // If already a Bedrock ID, use as-is
        if model.starts_with("anthropic.")
            || model.starts_with("global.")
            || model.starts_with("us.")
            || model.starts_with("eu.")
            || model.starts_with("ap.")
        {
            return Ok(model.to_string());
        }

        // Try to find the model and get its Bedrock ID
        if let Some(model_info) = crate::models::get_model_by_anthropic_id(model) {
            if let Some(bedrock_id) = model_info.bedrock_id {
                return Ok(bedrock_id.to_string());
            }
        }

        // Fallback: assume it's a valid ID
        Ok(model.to_string())
    }

    /// Send a message and stream the response
    ///
    /// Returns a stream of events as Claude generates its response.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use claude_sdk::{ClaudeClient, MessagesRequest, Message, StreamEvent};
    /// use futures::StreamExt;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClaudeClient::anthropic("your-api-key");
    ///
    /// let request = MessagesRequest::new(
    ///     claude_sdk::models::CLAUDE_SONNET_4_5.anthropic_id,
    ///     1024,
    ///     vec![Message::user("Tell me a story")],
    /// );
    ///
    /// let mut stream = client.send_streaming(request).await?;
    ///
    /// while let Some(event) = stream.next().await {
    ///     match event? {
    ///         StreamEvent::ContentBlockDelta { delta, .. } => {
    ///             if let Some(text) = delta.text() {
    ///                 print!("{}", text);
    ///             }
    ///         }
    ///         StreamEvent::MessageStop => break,
    ///         _ => {}
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, request), fields(model = %request.model))]
    pub async fn send_streaming(
        &self,
        request: MessagesRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        match &self.backend {
            ClaudeBackend::Anthropic { .. } => self.send_streaming_anthropic(request).await,
            #[cfg(feature = "bedrock")]
            ClaudeBackend::Bedrock { .. } => self.send_streaming_bedrock(request).await,
        }
    }

    /// Send streaming message to Anthropic API
    async fn send_streaming_anthropic(
        &self,
        request: MessagesRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        let api_key = match &self.backend {
            ClaudeBackend::Anthropic { api_key } => api_key,
            #[allow(unreachable_patterns)]
            _ => unreachable!("send_streaming_anthropic called with non-Anthropic backend"),
        };

        debug!("Sending streaming message to Anthropic API");

        // Enable streaming
        let mut request = request;
        request.stream = Some(true);

        let response = self
            .http
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", api_key)
            .header("anthropic-version", &self.api_version)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        debug!("Received streaming response with status: {}", status);

        // Handle non-OK status codes
        if !status.is_success() {
            return Err(self.handle_error_response(status, response).await);
        }

        // Convert the response into an SSE stream
        let byte_stream = response.bytes_stream();
        let event_stream = byte_stream.eventsource();

        // Map SSE events to our StreamEvent type
        let stream = event_stream.map(|result| {
            let event = result.map_err(|e| Error::StreamParse(e.to_string()))?;

            // Skip empty data
            if event.data.is_empty() {
                return Ok(None);
            }

            // Parse based on event type
            let stream_event = match event.event.as_str() {
                "ping" => Some(StreamEvent::Ping),
                "error" => {
                    let error: crate::streaming::StreamError = serde_json::from_str(&event.data)
                        .map_err(|e| Error::StreamParse(e.to_string()))?;
                    Some(StreamEvent::Error { error })
                }
                _ => {
                    // All other events (message_start, content_block_start, etc.)
                    // follow the standard format with type field
                    Some(
                        serde_json::from_str::<StreamEvent>(&event.data).map_err(|e| {
                            Error::StreamParse(format!(
                                "Failed to parse event '{}': {}",
                                event.event, e
                            ))
                        })?,
                    )
                }
            };

            Ok(stream_event)
        });

        // Filter out None values
        let filtered_stream = stream.try_filter_map(|opt| async move { Ok(opt) });

        Ok(Box::pin(filtered_stream))
    }

    /// Send streaming message to AWS Bedrock
    #[cfg(feature = "bedrock")]
    async fn send_streaming_bedrock(
        &self,
        request: MessagesRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        let (bedrock_client, model_id) = match &self.backend {
            ClaudeBackend::Bedrock { bedrock_client, .. } => {
                let model_id = self.get_bedrock_model_id(&request.model)?;
                (bedrock_client, model_id)
            }
            _ => unreachable!("send_streaming_bedrock called with non-Bedrock backend"),
        };

        debug!("Sending streaming message to AWS Bedrock");

        // Enable streaming
        let mut request = request;
        request.stream = Some(true);

        // Serialize request to JSON
        let body = serde_json::to_string(&request)?;

        // Use Bedrock runtime client with streaming
        let response = bedrock_client
            .invoke_model_with_response_stream()
            .model_id(&model_id)
            .content_type("application/json")
            .body(aws_sdk_bedrockruntime::primitives::Blob::new(
                body.as_bytes(),
            ))
            .send()
            .await
            .map_err(|e| Error::Network(format!("Bedrock streaming API call failed: {}", e)))?;

        // Convert Bedrock EventReceiver to a stream
        let mut event_stream = response.body;

        // Create a stream by polling the EventReceiver
        let stream = async_stream::stream! {
            loop {
                match event_stream.recv().await {
                    Ok(Some(event)) => {
                        // Parse the event based on Bedrock's format
                        if let aws_sdk_bedrockruntime::types::ResponseStream::Chunk(payload) = event {
                            let bytes = payload.bytes().ok_or_else(|| {
                                Error::StreamParse("Bedrock chunk missing bytes".into())
                            })?;

                            let json_str = std::str::from_utf8(bytes.as_ref())
                                .map_err(|e| Error::StreamParse(format!("Invalid UTF-8: {}", e)))?;

                            // Parse as StreamEvent
                            let stream_event: StreamEvent = serde_json::from_str(json_str)
                                .map_err(|e| Error::StreamParse(format!("Failed to parse Bedrock event: {}", e)))?;

                            yield Ok(stream_event);
                        }
                        // Skip other event types
                    }
                    Ok(None) => break, // Stream ended
                    Err(e) => {
                        yield Err(Error::StreamParse(format!("Bedrock stream error: {}", e)));
                        break;
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    /// Helper to handle error responses
    async fn handle_error_response(
        &self,
        status: StatusCode,
        response: reqwest::Response,
    ) -> Error {
        match status {
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse().ok());

                let error_body = response.text().await.unwrap_or_default();
                Error::RateLimit {
                    retry_after,
                    message: error_body,
                }
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                let error_body = response.text().await.unwrap_or_default();
                Error::Authentication(error_body)
            }
            StatusCode::BAD_REQUEST => {
                let error_text = response.text().await.unwrap_or_default();
                if let Ok(api_error) = serde_json::from_str::<ApiErrorResponse>(&error_text) {
                    Error::Api {
                        status: status.as_u16(),
                        message: api_error.error.message,
                        error_type: Some(api_error.error.error_type),
                    }
                } else {
                    Error::InvalidRequest(error_text)
                }
            }
            _ if status.is_server_error() => {
                let error_body = response.text().await.unwrap_or_default();
                Error::Server {
                    status: status.as_u16(),
                    message: error_body,
                }
            }
            _ => {
                let error_body = response.text().await.unwrap_or_default();
                Error::Api {
                    status: status.as_u16(),
                    message: error_body,
                    error_type: None,
                }
            }
        }
    }

    /// Send a message with automatic retry on transient failures
    ///
    /// This method automatically retries on rate limits (429) and server errors (5xx)
    /// using exponential backoff. Use the provided `RetryConfig` to customize behavior.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use claude_sdk::{ClaudeClient, MessagesRequest, Message};
    /// use claude_sdk::retry::RetryConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClaudeClient::anthropic("your-api-key");
    ///
    /// let request = MessagesRequest::new(
    ///     claude_sdk::models::CLAUDE_SONNET_4_5.anthropic_id,
    ///     1024,
    ///     vec![Message::user("Hello!")],
    /// );
    ///
    /// let config = RetryConfig::new().with_max_attempts(5);
    /// let response = client.send_message_with_retry(request, config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_message_with_retry(
        &self,
        request: MessagesRequest,
        config: crate::retry::RetryConfig,
    ) -> Result<MessagesResponse> {
        crate::retry::retry_with_backoff(config, || async {
            self.send_message(request.clone()).await
        })
        .await
    }

    /// Send a streaming message with automatic retry on transient failures
    ///
    /// Note: Retries create a new stream, so partial results from failed attempts are lost.
    pub async fn send_streaming_with_retry(
        &self,
        request: MessagesRequest,
        config: crate::retry::RetryConfig,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        crate::retry::retry_with_backoff(config, || async {
            self.send_streaming(request.clone()).await
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation_anthropic() {
        let client = ClaudeClient::anthropic("test-key");

        match &client.backend {
            ClaudeBackend::Anthropic { api_key } => {
                assert_eq!(api_key, "test-key");
            }
            #[allow(unreachable_patterns)]
            _ => panic!("Expected Anthropic backend"),
        }

        assert_eq!(client.api_version, API_VERSION);
    }

    #[tokio::test]
    #[cfg(feature = "bedrock")]
    #[ignore] // Requires AWS credentials
    async fn test_client_creation_bedrock() {
        // This test only runs with: cargo test -- --ignored
        // and requires AWS credentials to be configured
        let result = ClaudeClient::bedrock("us-east-1").await;

        if let Ok(client) = result {
            match &client.backend {
                ClaudeBackend::Bedrock { region, .. } => {
                    assert_eq!(region, "us-east-1");
                }
                _ => panic!("Expected Bedrock backend"),
            }
        }
        // If credentials aren't available, test is skipped
    }
}
