//! Structured output example with adaptive thinking
//!
//! This example shows how to get guaranteed JSON output from Claude
//! using JSON schema, combined with adaptive thinking and other
//! v2.0+ features.
//!
//! Run with:
//! ```bash
//! cargo run --example structured_output
//! ```

use claude_sdk::{CacheTtl, EffortLevel, Message, MessagesRequest, Metadata, ServiceTier};
use serde_json::json;

fn main() {
    println!("Claude SDK - Structured Output Example");
    println!("==========================================\n");

    // Define a JSON schema for the output
    let schema = json!({
        "type": "object",
        "properties": {
            "summary": {
                "type": "string",
                "description": "Brief summary of the analysis"
            },
            "sentiment": {
                "type": "string",
                "enum": ["positive", "negative", "neutral", "mixed"]
            },
            "confidence": {
                "type": "number",
                "minimum": 0.0,
                "maximum": 1.0
            },
            "key_topics": {
                "type": "array",
                "items": {"type": "string"},
                "maxItems": 5
            },
            "recommendation": {
                "type": "string"
            }
        },
        "required": ["summary", "sentiment", "confidence", "key_topics"]
    });

    println!("JSON Schema:");
    println!("{}\n", serde_json::to_string_pretty(&schema).unwrap());

    // Build request with all the new v2.0+ features
    let request = MessagesRequest::new(
        "claude-sonnet-4-5-20250929",
        4096,
        vec![Message::user(
            "Analyze this product review: 'The new laptop is incredibly fast \
             and the battery lasts all day. However, the keyboard feels cheap \
             and the trackpad is too small. Overall decent value for the price.'",
        )],
    )
    .with_system("You are a product review analyst. Always provide structured analysis.")
    // Structured output: guarantees JSON matching the schema
    .with_json_schema(schema)
    // Adaptive thinking: Claude decides how much reasoning to use
    .with_adaptive_thinking()
    // Effort level: maximum quality
    .with_effort(EffortLevel::High)
    // Service tier: use priority capacity if available
    .with_service_tier(ServiceTier::Auto)
    // Metadata: track the user for abuse detection
    .with_metadata(Metadata {
        user_id: Some("analyst-42".into()),
    })
    // Container: persist state for follow-up analysis
    .with_container("analysis-session-001");

    // Suppress unused import warning -- CacheTtl is imported to demonstrate
    // it is available for cache TTL configuration alongside other v2+ types
    let _ = CacheTtl::OneHour;

    // Print the full request to show all features
    println!("Full Request JSON:");
    let json = serde_json::to_string_pretty(&request).unwrap();
    println!("{}\n", json);

    // Highlight the key fields
    let val: serde_json::Value = serde_json::to_value(&request).unwrap();

    println!("Key Features Used:");
    if val.get("output_config").is_some() {
        println!("  output_config.format -- JSON schema for structured output");
        println!("  output_config.effort -- High effort for maximum quality");
    }
    if val.get("thinking").is_some() {
        println!("  thinking -- Adaptive (Claude decides reasoning depth)");
    }
    if val.get("service_tier").is_some() {
        println!("  service_tier -- Auto (priority if available)");
    }
    if val.get("metadata").is_some() {
        println!("  metadata -- User tracking for abuse detection");
    }
    if val.get("container").is_some() {
        println!("  container -- Persistent execution context");
    }

    println!("\nTo send this request, set ANTHROPIC_API_KEY and use:");
    println!("  let response = client.send_message(request).await?;");
    println!("\nThe response will be guaranteed JSON matching the schema above.");
}
