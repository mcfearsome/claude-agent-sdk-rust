//! Prompt caching example demonstrating cost reduction
//!
//! This example shows how to use prompt caching to reduce costs when making
//! repeated requests with the same context (system prompt, tools, documents).
//!
//! Prompt caching reduces input token costs by ~90% for cached content.
//! Cache duration: ~5 minutes (ephemeral cache)
//!
//! Run with:
//! ```bash
//! export ANTHROPIC_API_KEY="your-api-key"
//! cargo run --example prompt_caching
//! ```

use claude_sdk::{ClaudeClient, ConversationBuilder, Tool};
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    let client = ClaudeClient::anthropic(api_key);

    println!("💰 Claude SDK - Prompt Caching Example");
    println!("=======================================\n");
    println!("Demonstrating cost reduction via prompt caching\n");

    // Define tools (these will be cached)
    let search_tool = Tool {
        name: "search_code".into(),
        description: "Search through a codebase for specific patterns".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "file_pattern": {
                    "type": "string",
                    "description": "File pattern to search in"
                }
            },
            "required": ["query"]
        }),
        disable_user_input: Some(true),
        input_examples: None,
        cache_control: None, // Will be set by with_cached_tool()
    };

    let read_file_tool = Tool {
        name: "read_file".into(),
        description: "Read contents of a file".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path"
                }
            },
            "required": ["path"]
        }),
        disable_user_input: Some(true),
        input_examples: None,
        cache_control: None,
    };

    // Build conversation with cached system and tools
    let mut conversation = ConversationBuilder::new()
        .with_cached_system(
            "You are an expert code assistant. You have access to tools for \
             searching code and reading files. Always use these tools to provide \
             accurate answers about the codebase.",
        )
        .with_cached_tool(search_tool)
        .with_cached_tool(read_file_tool);

    println!("🔧 Configured:");
    println!("   ✓ Cached system prompt");
    println!("   ✓ Cached 2 tool definitions\n");

    // First request - will write to cache
    println!("📤 Request 1: Writing to cache...");
    conversation.add_user_message("How do I search for TODO comments?");

    let request1 = conversation.build(claude_sdk::models::CLAUDE_SONNET_4_5.anthropic_id, 512);
    let response1 = client.send_message(request1).await?;

    println!("   Input tokens:  {}", response1.usage.input_tokens);
    println!("   Output tokens: {}", response1.usage.output_tokens);

    if let Some(cache_write) = response1.usage.cache_creation_input_tokens {
        println!("   💾 Cache write: {} tokens", cache_write);
    }

    // Extract response text
    for content in &response1.content {
        if let claude_sdk::ContentBlock::Text { text, .. } = content {
            println!(
                "\n   Response: {}\n",
                text.chars().take(100).collect::<String>()
            );
            if text.len() > 100 {
                println!("   [...]\n");
            }
        }
    }

    // Wait a moment to simulate user thinking
    println!("⏳ Waiting 2 seconds...\n");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Second request - will hit cache!
    println!("📤 Request 2: Reading from cache...");
    conversation.add_user_message("And how do I read the src/lib.rs file?");

    let request2 = conversation.build(claude_sdk::models::CLAUDE_SONNET_4_5.anthropic_id, 512);
    let response2 = client.send_message(request2).await?;

    println!("   Input tokens:  {}", response2.usage.input_tokens);
    println!("   Output tokens: {}", response2.usage.output_tokens);

    if let Some(cache_read) = response2.usage.cache_read_input_tokens {
        println!("   ✨ Cache HIT: {} tokens (90% cheaper!)", cache_read);
    }

    // Extract response text
    for content in &response2.content {
        if let claude_sdk::ContentBlock::Text { text, .. } = content {
            println!(
                "\n   Response: {}\n",
                text.chars().take(100).collect::<String>()
            );
            if text.len() > 100 {
                println!("   [...]\n");
            }
        }
    }

    // Calculate cost comparison
    println!("💵 Cost Analysis:");

    let model = claude_sdk::models::CLAUDE_SONNET_4_5;

    // Request 1 costs
    let cost1_input = response1.usage.input_tokens as f64 / 1_000_000.0 * model.cost_per_mtok_input;
    let cost1_cache = response1.usage.cache_creation_input_tokens.unwrap_or(0) as f64 / 1_000_000.0
        * model.cost_per_mtok_input
        * 1.25; // Cache writes cost 25% more
    let cost1_output =
        response1.usage.output_tokens as f64 / 1_000_000.0 * model.cost_per_mtok_output;
    let cost1_total = cost1_input + cost1_cache + cost1_output;

    println!("   Request 1: ${:.6}", cost1_total);

    // Request 2 costs (with cache hit)
    let cost2_input = response2.usage.input_tokens as f64 / 1_000_000.0 * model.cost_per_mtok_input;
    let cost2_cache_read = response2.usage.cache_read_input_tokens.unwrap_or(0) as f64
        / 1_000_000.0
        * model.cost_per_mtok_input
        * 0.1; // Cache reads cost 90% less
    let cost2_output =
        response2.usage.output_tokens as f64 / 1_000_000.0 * model.cost_per_mtok_output;
    let cost2_total = cost2_input + cost2_cache_read + cost2_output;

    println!("   Request 2: ${:.6}", cost2_total);

    if response2.usage.cache_read_input_tokens.is_some() {
        let savings = cost1_total - cost2_total;
        let savings_pct = (savings / cost1_total) * 100.0;
        println!("\n   💰 Savings: ${:.6} ({:.1}%)", savings, savings_pct);
    }

    println!("\n✨ Best Practices:");
    println!("   • Cache stable content: system prompts, tool definitions, large documents");
    println!("   • Cache duration: ~5 minutes");
    println!("   • Cost: Cache writes +25%, cache reads -90%");
    println!("   • Break-even: 2 requests with cache hit saves money");

    Ok(())
}
