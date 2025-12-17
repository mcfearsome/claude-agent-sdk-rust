//! Streaming support for Claude API.
//!
//! This module handles Server-Sent Events (SSE) streaming from the Claude API,
//! providing real-time token generation and typed event parsing.
//!
//! # Streaming vs Non-Streaming
//!
//! | Aspect | Streaming | Non-Streaming |
//! |--------|-----------|---------------|
//! | Latency | First token immediately | Wait for full response |
//! | UX | Progressive display | All-at-once |
//! | Memory | Constant | Grows with response |
//! | Use Case | Chat interfaces | Background processing |
//!
//! # Event Types
//!
//! The stream emits events in this order:
//!
//! 1. [`StreamEvent::MessageStart`] - Stream begins, provides message metadata
//! 2. [`StreamEvent::ContentBlockStart`] - A new content block (text/tool/thinking) begins
//! 3. [`StreamEvent::ContentBlockDelta`] - Incremental content updates
//! 4. [`StreamEvent::ContentBlockStop`] - Content block complete
//! 5. [`StreamEvent::MessageDelta`] - Final usage stats and stop reason
//! 6. [`StreamEvent::MessageStop`] - Stream complete
//!
//! # Basic Streaming Example
//!
//! ```rust,no_run
//! use claude_sdk::{ClaudeClient, MessagesRequest, Message, StreamEvent, ContentDelta};
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ClaudeClient::anthropic(std::env::var("ANTHROPIC_API_KEY")?);
//!
//! let request = MessagesRequest::new(
//!     "claude-sonnet-4-5-20250929",
//!     1024,
//!     vec![Message::user("Write a poem about Rust.")],
//! );
//!
//! let mut stream = client.send_streaming(request).await?;
//!
//! while let Some(event) = stream.next().await {
//!     match event? {
//!         StreamEvent::ContentBlockDelta { delta, .. } => {
//!             if let ContentDelta::TextDelta { text } = delta {
//!                 print!("{}", text);  // Stream to console
//!             }
//!         }
//!         StreamEvent::MessageDelta { usage, .. } => {
//!             println!("\n[Used {} output tokens]", usage.output_tokens);
//!         }
//!         StreamEvent::MessageStop => break,
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Handling Tool Use in Streams
//!
//! Tool calls are streamed as JSON deltas that need to be accumulated:
//!
//! ```rust,no_run
//! use claude_sdk::{StreamEvent, ContentDelta};
//!
//! # fn example(event: StreamEvent) {
//! match event {
//!     StreamEvent::ContentBlockStart { content_block, .. } => {
//!         // Tool use block starting - capture the tool name
//!     }
//!     StreamEvent::ContentBlockDelta { delta, .. } => {
//!         if let ContentDelta::InputJsonDelta { partial_json } = delta {
//!             // Accumulate JSON fragments
//!             // parse when ContentBlockStop is received
//!         }
//!     }
//!     StreamEvent::ContentBlockStop { .. } => {
//!         // Tool JSON complete - now parse and execute
//!     }
//!     _ => {}
//! }
//! # }
//! ```
//!
//! # Extended Thinking in Streams
//!
//! When extended thinking is enabled, thinking blocks are streamed first:
//!
//! ```rust,no_run
//! use claude_sdk::{StreamEvent, ContentDelta};
//!
//! # fn example(event: StreamEvent) {
//! match event {
//!     StreamEvent::ContentBlockDelta { delta, .. } => {
//!         match delta {
//!             ContentDelta::ThinkingDelta { thinking } => {
//!                 // Claude's reasoning process
//!                 print!("[thinking] {}", thinking);
//!             }
//!             ContentDelta::TextDelta { text } => {
//!                 // Final response
//!                 print!("{}", text);
//!             }
//!             _ => {}
//!         }
//!     }
//!     _ => {}
//! }
//! # }
//! ```

use crate::types::{ContentBlock, Role, StopReason, Usage};
use serde::{Deserialize, Serialize};

/// Events emitted during streaming responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// Stream started - provides message metadata
    MessageStart { message: MessageMetadata },

    /// A content block has started
    ContentBlockStart {
        index: usize,
        content_block: ContentBlock,
    },

    /// Delta update to a content block
    ContentBlockDelta { index: usize, delta: ContentDelta },

    /// A content block has finished
    ContentBlockStop { index: usize },

    /// Delta update to the message (includes final usage)
    MessageDelta { delta: MessageDelta, usage: Usage },

    /// Stream has ended
    MessageStop,

    /// Ping event (keepalive)
    Ping,

    /// Error event
    Error { error: StreamError },
}

/// Message metadata provided at stream start
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub id: String,

    #[serde(rename = "type")]
    pub message_type: String,

    pub role: Role,

    #[serde(default)]
    pub content: Vec<ContentBlock>,

    pub model: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,

    pub usage: Usage,
}

/// Delta update to content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentDelta {
    /// Text delta
    TextDelta { text: String },

    /// Input JSON delta for tool use
    InputJsonDelta { partial_json: String },

    /// Thinking delta (extended thinking)
    ThinkingDelta { thinking: String },

    /// Signature delta for thinking blocks
    SignatureDelta { signature: String },
}

impl ContentDelta {
    /// Extract text if this is a text delta
    pub fn text(&self) -> Option<&str> {
        match self {
            ContentDelta::TextDelta { text } => Some(text),
            _ => None,
        }
    }

    /// Extract partial JSON if this is an input JSON delta
    pub fn partial_json(&self) -> Option<&str> {
        match self {
            ContentDelta::InputJsonDelta { partial_json } => Some(partial_json),
            _ => None,
        }
    }

    /// Extract thinking if this is a thinking delta
    pub fn thinking(&self) -> Option<&str> {
        match self {
            ContentDelta::ThinkingDelta { thinking } => Some(thinking),
            _ => None,
        }
    }

    /// Extract signature if this is a signature delta
    pub fn signature(&self) -> Option<&str> {
        match self {
            ContentDelta::SignatureDelta { signature } => Some(signature),
            _ => None,
        }
    }
}

/// Delta update to the message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
}

/// Error in the stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamError {
    #[serde(rename = "type")]
    pub error_type: String,

    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_message_start() {
        let json = r#"{
            "type": "message_start",
            "message": {
                "id": "msg_123",
                "type": "message",
                "role": "assistant",
                "content": [],
                "model": "claude-sonnet-4-5-20250929",
                "stop_reason": null,
                "stop_sequence": null,
                "usage": {
                    "input_tokens": 10,
                    "output_tokens": 0
                }
            }
        }"#;

        let event: StreamEvent = serde_json::from_str(json).unwrap();
        match event {
            StreamEvent::MessageStart { message } => {
                assert_eq!(message.id, "msg_123");
                assert_eq!(message.role, Role::Assistant);
            }
            _ => panic!("Expected MessageStart"),
        }
    }

    #[test]
    fn test_parse_content_block_delta() {
        let json = r#"{
            "type": "content_block_delta",
            "index": 0,
            "delta": {
                "type": "text_delta",
                "text": "Hello"
            }
        }"#;

        let event: StreamEvent = serde_json::from_str(json).unwrap();
        match event {
            StreamEvent::ContentBlockDelta { index, delta } => {
                assert_eq!(index, 0);
                assert_eq!(delta.text(), Some("Hello"));
            }
            _ => panic!("Expected ContentBlockDelta"),
        }
    }

    #[test]
    fn test_content_delta_helpers() {
        let text_delta = ContentDelta::TextDelta {
            text: "test".to_string(),
        };
        assert_eq!(text_delta.text(), Some("test"));
        assert_eq!(text_delta.partial_json(), None);

        let json_delta = ContentDelta::InputJsonDelta {
            partial_json: r#"{"key":"#.to_string(),
        };
        assert_eq!(json_delta.text(), None);
        assert_eq!(json_delta.partial_json(), Some(r#"{"key":"#));
    }
}
