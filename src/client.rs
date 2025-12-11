//! Claude API client implementation

use crate::error::{ApiErrorResponse, Error, Result};
use crate::streaming::StreamEvent;
use crate::types::{MessagesRequest, MessagesResponse};
use bytes::Bytes;
use eventsource_stream::Eventsource;
use futures::Stream;
use reqwest::{Client, StatusCode};
use std::pin::Pin;
use tracing::{debug, instrument};

/// API endpoint for Anthropic
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Current API version
const API_VERSION: &str = "2023-06-01";

/// Claude API client
///
/// This client can connect to either the Anthropic API directly or AWS Bedrock.
///
/// # Example
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
pub struct ClaudeClient {
    http: Client,
    api_key: String,
    api_url: String,
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
            api_key: api_key.into(),
            api_url: ANTHROPIC_API_URL.to_string(),
            api_version: API_VERSION.to_string(),
        }
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
        debug!("Sending message to Claude API");

        // Ensure stream is not set or is false
        let mut request = request;
        request.stream = Some(false);

        let response = self
            .http
            .post(&self.api_url)
            .header("x-api-key", &self.api_key)
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
        debug!("Sending streaming message to Claude API");

        // Enable streaming
        let mut request = request;
        request.stream = Some(true);

        let response = self
            .http
            .post(&self.api_url)
            .header("x-api-key", &self.api_key)
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
        let stream = futures::stream::try_unfold(event_stream, |mut es| async move {
            use eventsource_stream::Event;
            use futures::StreamExt;

            match es.next().await {
                Some(Ok(Event::Open)) => {
                    // Connection opened, continue to next event
                    Ok(Some((None, es)))
                }
                Some(Ok(Event::Message(msg))) => {
                    // Parse the event data
                    let event = match msg.event.as_str() {
                        "ping" => Some(StreamEvent::Ping),
                        "error" => {
                            let error: crate::streaming::StreamError =
                                serde_json::from_str(&msg.data)
                                    .map_err(|e| Error::StreamParse(e.to_string()))?;
                            Some(StreamEvent::Error { error })
                        }
                        _ => {
                            // All other events follow the standard format
                            serde_json::from_str::<StreamEvent>(&msg.data)
                                .map_err(|e| Error::StreamParse(format!("Failed to parse event {}: {}", msg.event, e)))?
                                .into()
                        }
                    };

                    Ok(Some((event, es)))
                }
                Some(Err(e)) => Err(Error::StreamParse(e.to_string())),
                None => Ok(None), // Stream ended
            }
        });

        // Filter out None values and flatten the stream
        let filtered_stream = futures::stream::try_unfold(stream, |mut s| async move {
            use futures::StreamExt;

            loop {
                match s.try_next().await? {
                    Some(Some(event)) => return Ok(Some((event, s))),
                    Some(None) => continue, // Skip None events
                    None => return Ok(None), // Stream ended
                }
            }
        });

        Ok(Box::pin(filtered_stream))
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ClaudeClient::anthropic("test-key");
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.api_url, ANTHROPIC_API_URL);
        assert_eq!(client.api_version, API_VERSION);
    }
}
