//! Simple chat example using the Claude SDK
//!
//! This example demonstrates:
//! - Creating a Claude client
//! - Sending a simple message
//! - Receiving and displaying the response
//!
//! Run with:
//! ```bash
//! export ANTHROPIC_API_KEY="your-api-key"
//! cargo run --example simple_chat
//! ```

use claude_sdk::{ClaudeClient, Message, MessagesRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Get API key from environment
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    // Create the client
    let client = ClaudeClient::anthropic(api_key);

    println!("🤖 Claude SDK - Simple Chat Example");
    println!("=====================================\n");

    // Create a simple request using the latest Sonnet model
    let request = MessagesRequest::new(
        claude_sdk::models::CLAUDE_SONNET_4_5.anthropic_id,
        1024,
        vec![Message::user(
            "Hello! Can you explain what makes Rust a great language for systems programming?",
        )],
    );

    println!("Sending message to Claude...\n");

    // Send the message
    let response = client.send_message(request).await?;

    // Display the response
    println!("📝 Response:");
    println!("─────────────────────────────────────");
    for content in &response.content {
        match content {
            claude_sdk::ContentBlock::Text { text, .. } => {
                println!("{}", text);
            }
            _ => {}
        }
    }
    println!("─────────────────────────────────────\n");

    // Display usage information
    println!("📊 Token Usage:");
    println!("  Input tokens:  {}", response.usage.input_tokens);
    println!("  Output tokens: {}", response.usage.output_tokens);
    println!("  Total tokens:  {}", response.usage.input_tokens + response.usage.output_tokens);

    if let Some(stop_reason) = response.stop_reason {
        println!("  Stop reason:   {:?}", stop_reason);
    }

    Ok(())
}
