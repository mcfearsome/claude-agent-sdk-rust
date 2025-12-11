//! Streaming chat example using the Claude SDK
//!
//! This example demonstrates:
//! - Creating a Claude client
//! - Sending a message with streaming enabled
//! - Processing stream events in real-time
//! - Displaying text as it arrives
//!
//! Run with:
//! ```bash
//! export ANTHROPIC_API_KEY="your-api-key"
//! cargo run --example streaming_chat
//! ```

use claude_sdk::{ClaudeClient, Message, MessagesRequest, StreamEvent};
use futures::StreamExt;
use std::io::{self, Write};
use tracing::debug;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Get API key from environment
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    // Create the client
    let client = ClaudeClient::anthropic(api_key);

    println!("🤖 Claude SDK - Streaming Chat Example");
    println!("========================================\n");

    // Create a streaming request
    let request = MessagesRequest::new(
        claude_sdk::models::CLAUDE_SONNET_4_5.anthropic_id,
        1024,
        vec![Message::user(
            "Tell me a short story about a robot learning to feel emotions. Keep it under 200 words.",
        )],
    );

    println!("Sending streaming message to Claude...\n");
    println!("📝 Streaming response:");
    println!("─────────────────────────────────────");

    // Send the streaming request
    let mut stream = client.send_streaming(request).await?;

    let mut total_text = String::new();
    let mut input_tokens = 0;
    let mut output_tokens = 0;

    // Process events as they arrive
    while let Some(event_result) = stream.next().await {
        match event_result? {
            StreamEvent::MessageStart { message } => {
                input_tokens = message.usage.input_tokens;
                debug!("Message started: {}", message.id);
            }

            StreamEvent::ContentBlockStart {
                index,
                content_block,
            } => {
                debug!("Content block {} started: {:?}", index, content_block);
            }

            StreamEvent::ContentBlockDelta { delta, .. } => {
                if let Some(text) = delta.text() {
                    print!("{}", text);
                    io::stdout().flush()?;
                    total_text.push_str(text);
                }
            }

            StreamEvent::ContentBlockStop { index } => {
                debug!("Content block {} stopped", index);
            }

            StreamEvent::MessageDelta { delta, usage } => {
                debug!("Message delta: {:?}", delta);
                output_tokens = usage.output_tokens;

                if let Some(cache_read) = usage.cache_read_input_tokens {
                    println!("\n[Cache hit: {} tokens]", cache_read);
                }
            }

            StreamEvent::MessageStop => {
                println!();
                debug!("Message stopped");
                break;
            }

            StreamEvent::Ping => {
                debug!("Received ping");
            }

            StreamEvent::Error { error } => {
                eprintln!("\n❌ Stream error: {}", error.message);
                return Err(error.message.into());
            }
        }
    }

    println!("─────────────────────────────────────\n");

    // Display statistics
    println!("📊 Statistics:");
    println!("  Characters streamed: {}", total_text.len());
    println!("  Input tokens:  {}", input_tokens);
    println!("  Output tokens: {}", output_tokens);
    println!("  Total tokens:  {}", input_tokens + output_tokens);

    // Estimate cost
    let cost = claude_sdk::models::CLAUDE_SONNET_4_5.estimate_cost(input_tokens, output_tokens);
    println!("  Estimated cost: ${:.4}", cost);

    Ok(())
}
