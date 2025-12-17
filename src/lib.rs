//! # Claude SDK for Rust
//!
//! [![Crates.io](https://img.shields.io/crates/v/claude-sdk.svg)](https://crates.io/crates/claude-sdk)
//! [![Documentation](https://docs.rs/claude-sdk/badge.svg)](https://docs.rs/claude-sdk)
//! [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
//!
//! A native Rust implementation of the Claude API client with full support for
//! streaming, tool execution, vision, batch processing, and more.
//!
//! ## Features
//!
//! - **Complete API Coverage**: Messages, streaming, tools, vision, batch processing
//! - **Multi-Platform**: Anthropic API and AWS Bedrock support
//! - **Type-Safe**: Comprehensive type definitions for all API structures
//! - **Async/Await**: Built on tokio for efficient async operations
//! - **Streaming**: Server-sent events (SSE) with typed event parsing
//! - **Tool Use**: Define tools and handle programmatic tool calls
//! - **Prompt Caching**: Cache system prompts and tools for cost savings
//! - **Extended Thinking**: Enable Claude's step-by-step reasoning
//! - **Token Counting**: Estimate token usage before API calls
//! - **Retry Logic**: Built-in exponential backoff for rate limits
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! claude-sdk = "1.0"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Basic usage:
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
//!         "claude-sonnet-4-5-20250929",
//!         1024,
//!         vec![Message::user("Hello, Claude!")],
//!     );
//!
//!     let response = client.send_message(request).await?;
//!     println!("Response: {:?}", response);
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Flags
//!
//! The SDK uses feature flags to control optional functionality:
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `anthropic` | Yes | Enable Anthropic API support |
//! | `bedrock` | No | Enable AWS Bedrock support |
//! | `repl` | No | Include interactive REPL binary |
//! | `full` | No | Enable all features |
//!
//! To enable AWS Bedrock support:
//!
//! ```toml
//! [dependencies]
//! claude-sdk = { version = "1.0", features = ["bedrock"] }
//! ```
//!
//! ## Streaming Responses
//!
//! Use streaming for real-time token generation:
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
//!     vec![Message::user("Write a haiku about Rust.")],
//! );
//!
//! let mut stream = client.send_streaming(request).await?;
//!
//! while let Some(event) = stream.next().await {
//!     match event? {
//!         StreamEvent::ContentBlockDelta { delta, .. } => {
//!             if let ContentDelta::TextDelta { text } = delta {
//!                 print!("{}", text);
//!             }
//!         }
//!         StreamEvent::MessageStop => println!("\n--- Done ---"),
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Tool Use
//!
//! Define and handle tool calls:
//!
//! ```rust,no_run
//! use claude_sdk::{ClaudeClient, MessagesRequest, Message, Tool, ContentBlock};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ClaudeClient::anthropic(std::env::var("ANTHROPIC_API_KEY")?);
//!
//! let weather_tool = Tool {
//!     name: "get_weather".into(),
//!     description: "Get weather for a location".into(),
//!     input_schema: json!({
//!         "type": "object",
//!         "properties": {
//!             "location": { "type": "string" }
//!         },
//!         "required": ["location"]
//!     }),
//!     disable_user_input: Some(true),
//!     input_examples: None,
//!     cache_control: None,
//! };
//!
//! let request = MessagesRequest::new(
//!     "claude-sonnet-4-5-20250929",
//!     1024,
//!     vec![Message::user("What's the weather in Tokyo?")],
//! )
//! .with_tools(vec![weather_tool]);
//!
//! let response = client.send_message(request).await?;
//!
//! // Handle tool use in response
//! for block in &response.content {
//!     if let ContentBlock::ToolUse { id, name, input, .. } = block {
//!         println!("Tool: {} ({})", name, id);
//!         println!("Input: {}", input);
//!         // Execute tool and return result...
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## AWS Bedrock
//!
//! Use Claude through AWS Bedrock (requires `bedrock` feature):
//!
//! ```rust,ignore
//! use claude_sdk::ClaudeClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Uses AWS credentials from environment/config
//! let client = ClaudeClient::bedrock("us-east-1").await?;
//!
//! // Use the same API as Anthropic
//! // ...
//! # Ok(())
//! # }
//! ```
//!
//! ## Modules
//!
//! - [`client`] - API client for Anthropic and AWS Bedrock
//! - [`types`] - Request/response types and content blocks
//! - [`streaming`] - SSE streaming types and event parsing
//! - [`conversation`] - Multi-turn conversation builder
//! - [`batch`] - Batch processing API for bulk operations
//! - [`files`] - Files API for document uploads
//! - [`models`] - Model constants and metadata
//! - [`tokens`] - Token counting utilities
//! - [`retry`] - Retry logic with exponential backoff
//! - [`error`] - Error types and result aliases
//! - [`prompts`] - Pre-built system prompts
//! - [`structured`] - Structured output helpers
//!
//! ## Model Selection
//!
//! Use model constants for type-safe model selection:
//!
//! ```rust
//! use claude_sdk::models::{CLAUDE_SONNET_4_5, CLAUDE_OPUS_4_5, CLAUDE_HAIKU_4_5};
//!
//! // Latest models
//! let model = CLAUDE_SONNET_4_5;
//! println!("Using: {} ({})", model.name, model.anthropic_id);
//! println!("Max tokens: {}", model.max_output_tokens);
//! println!("Supports vision: {}", model.supports_vision);
//! ```
//!
//! ## Error Handling
//!
//! The SDK provides typed errors with retry information:
//!
//! ```rust,no_run
//! use claude_sdk::{ClaudeClient, MessagesRequest, Message, Error};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ClaudeClient::anthropic(std::env::var("ANTHROPIC_API_KEY")?);
//! let request = MessagesRequest::new(
//!     "claude-sonnet-4-5-20250929",
//!     1024,
//!     vec![Message::user("Hello!")],
//! );
//!
//! match client.send_message(request).await {
//!     Ok(response) => println!("Success!"),
//!     Err(Error::RateLimit { retry_after, .. }) => {
//!         println!("Rate limited. Retry after: {:?}s", retry_after);
//!     }
//!     Err(Error::Api { status, message, .. }) => {
//!         println!("API error ({}): {}", status, message);
//!     }
//!     Err(e) => println!("Other error: {}", e),
//! }
//! # Ok(())
//! # }
//! ```

pub mod batch;
pub mod client;
pub mod conversation;
pub mod error;
pub mod files;
pub mod models;
pub mod prompts;
pub mod retry;
pub mod streaming;
pub mod structured;
pub mod tokens;
pub mod types;

// Re-export main types for convenience
pub use client::ClaudeClient;
pub use conversation::ConversationBuilder;
pub use error::{Error, Result};
pub use models::{BedrockRegion, Model};
pub use streaming::{ContentDelta, MessageDelta, StreamEvent};
pub use types::{
    ContentBlock, EffortLevel, Message, MessagesRequest, MessagesResponse, OutputConfig, Role,
    StopReason, Tool, ToolChoice, Usage,
};
