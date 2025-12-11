//! # Claude SDK for Rust
//!
//! A native Rust implementation of the Claude API client with streaming support,
//! tool execution, and programmatic tool calling.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use claude_sdk::{ClaudeClient, MessagesRequest, Message};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = ClaudeClient::anthropic(
//!         std::env::var("ANTHROPIC_API_KEY")?
//!     );
//!
//!     let request = MessagesRequest::new(
//!         "claude-3-5-sonnet-20241022",
//!         1024,
//!         vec![Message::user("Hello, Claude!")],
//!     );
//!
//!     let response = client.send_message(request).await?;
//!     println!("Response: {:?}", response);
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod conversation;
pub mod error;
pub mod models;
pub mod streaming;
pub mod types;

// Re-export main types for convenience
pub use client::ClaudeClient;
pub use conversation::ConversationBuilder;
pub use error::{Error, Result};
pub use models::{BedrockRegion, Model};
pub use streaming::{ContentDelta, MessageDelta, StreamEvent};
pub use types::{ContentBlock, Message, MessagesRequest, MessagesResponse, Role, Tool, Usage};
