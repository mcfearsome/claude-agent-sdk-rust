//! Tool use example demonstrating programmatic tool calling
//!
//! This example shows:
//! - Defining tools with JSON schemas
//! - Using ConversationBuilder for multi-turn interactions
//! - Handling tool use requests from Claude
//! - Providing tool results
//! - Continuing the conversation with tool outputs
//!
//! Run with:
//! ```bash
//! export ANTHROPIC_API_KEY="your-api-key"
//! cargo run --example tool_use
//! ```

use claude_sdk::{ClaudeClient, ContentBlock, ConversationBuilder, StreamEvent, Tool};
use futures::StreamExt;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get API key
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    let client = ClaudeClient::anthropic(api_key);

    println!("🔧 Claude SDK - Tool Use Example");
    println!("=================================\n");

    // Define tools
    let get_weather_tool = Tool {
        name: "get_weather".into(),
        description: "Get the current weather for a given location".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name, e.g. 'San Francisco' or 'Paris'"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "Temperature unit"
                }
            },
            "required": ["location"]
        }),
        disable_user_input: Some(true), // Programmatic tool calling
        cache_control: None,
    };

    let calculate_tool = Tool {
        name: "calculate".into(),
        description: "Perform basic arithmetic calculations".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Mathematical expression, e.g. '2 + 2' or '10 * 5'"
                }
            },
            "required": ["expression"]
        }),
        disable_user_input: Some(true),
        cache_control: None,
    };

    // Build conversation with tools
    let mut conversation = ConversationBuilder::new()
        .with_system("You are a helpful assistant with access to weather and calculation tools.")
        .with_tool(get_weather_tool)
        .with_tool(calculate_tool);

    // Add user message
    conversation.add_user_message("What's the weather in San Francisco? Also, what's 15 * 24?");

    println!("👤 User: What's the weather in San Francisco? Also, what's 15 * 24?\n");

    // First turn - Claude will request tool use
    let request = conversation.build(claude_sdk::models::CLAUDE_SONNET_4_5.anthropic_id, 2048);
    let mut stream = client.send_streaming(request).await?;

    let mut assistant_response = Vec::new();
    let mut response_text = String::new();

    println!("🤖 Claude:");
    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::ContentBlockStart { content_block, .. } => {
                assistant_response.push(content_block.clone());

                match content_block {
                    ContentBlock::Text { text, .. } => {
                        print!("{}", text);
                        response_text.push_str(&text);
                    }
                    ContentBlock::ToolUse { name, input, .. } => {
                        println!("\n   🔧 Tool: {} ({})", name, input);
                    }
                    _ => {}
                }
            }

            StreamEvent::ContentBlockDelta { index, delta } => {
                if let Some(text) = delta.text() {
                    print!("{}", text);
                    response_text.push_str(text);

                    // Update the text in assistant_response
                    if let Some(ContentBlock::Text {
                        text: stored_text, ..
                    }) = assistant_response.get_mut(index)
                    {
                        stored_text.push_str(text);
                    }
                }
            }

            StreamEvent::MessageStop => break,
            _ => {}
        }
    }
    println!("\n");

    // Add assistant response to conversation
    conversation.add_assistant_with_blocks(assistant_response.clone());

    // Execute tools
    println!("🔨 Executing tools...\n");

    for content in &assistant_response {
        if let ContentBlock::ToolUse {
            id, name, input, ..
        } = content
        {
            let result = execute_tool(name, input)?;
            println!("   ✓ {}: {}", name, result);

            // Add tool result to conversation
            conversation.add_tool_result(id, result);
        }
    }
    println!();

    // Second turn - Claude will use the tool results
    let request = conversation.build(claude_sdk::models::CLAUDE_SONNET_4_5.anthropic_id, 1024);
    let mut stream = client.send_streaming(request).await?;

    println!("🤖 Claude (after tools):");
    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::ContentBlockDelta { delta, .. } => {
                if let Some(text) = delta.text() {
                    print!("{}", text);
                }
            }
            StreamEvent::MessageStop => break,
            _ => {}
        }
    }
    println!("\n");

    println!("📊 Conversation Summary:");
    println!("   Messages: {}", conversation.messages().len());
    println!("   Tools defined: {}", conversation.tools().len());

    Ok(())
}

/// Execute a tool and return the result
///
/// In a real application, this would call actual APIs or perform real operations.
/// For this example, we return mock data.
fn execute_tool(
    name: &str,
    input: &serde_json::Value,
) -> Result<String, Box<dyn std::error::Error>> {
    match name {
        "get_weather" => {
            let location = input["location"].as_str().unwrap_or("Unknown");
            let unit = input["unit"].as_str().unwrap_or("fahrenheit");

            // Mock weather data
            let (temp, temp_unit) = match unit {
                "celsius" => (22, "°C"),
                _ => (72, "°F"),
            };

            Ok(json!({
                "location": location,
                "temperature": temp,
                "unit": temp_unit,
                "condition": "Sunny",
                "humidity": 45,
                "wind_speed": 10
            })
            .to_string())
        }

        "calculate" => {
            let expression = input["expression"].as_str().unwrap_or("");

            // Simple calculation (in real app, use proper parser)
            let result = if expression.contains('+') {
                let parts: Vec<&str> = expression.split('+').collect();
                let a: i32 = parts[0].trim().parse().unwrap_or(0);
                let b: i32 = parts[1].trim().parse().unwrap_or(0);
                a + b
            } else if expression.contains('*') {
                let parts: Vec<&str> = expression.split('*').collect();
                let a: i32 = parts[0].trim().parse().unwrap_or(0);
                let b: i32 = parts[1].trim().parse().unwrap_or(0);
                a * b
            } else {
                0
            };

            Ok(json!({
                "expression": expression,
                "result": result
            })
            .to_string())
        }

        _ => Err(format!("Unknown tool: {}", name).into()),
    }
}
