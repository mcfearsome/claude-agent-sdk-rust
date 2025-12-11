# Claude SDK for Rust

A native Rust implementation of the Claude API client with streaming support, tool execution, and programmatic tool calling.

## Features

- ✅ **Non-streaming API** - Send messages and get complete responses
- ✅ **Streaming API** - Stream responses with Server-Sent Events in real-time
- ✅ **Tool Use** - Define and execute tools with programmatic calling
- ✅ **Conversation Management** - Multi-turn conversations with ConversationBuilder
- ✅ **Retry Logic** - Exponential backoff for rate limits and transient errors
- 🚧 **Prompt Caching** - Types ready, integration in progress
- 🚧 **AWS Bedrock** - Model IDs ready, client implementation in progress
- 🦀 **Idiomatic Rust** - Type-safe, async/await, zero-cost abstractions
- 📦 **Standalone** - No FFI, no subprocesses, pure Rust

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
claude-sdk = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Basic Usage

```rust
use claude_sdk::{ClaudeClient, Message, MessagesRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the client
    let client = ClaudeClient::anthropic(
        std::env::var("ANTHROPIC_API_KEY")?
    );

    // Create a request
    let request = MessagesRequest::new(
        "claude-3-5-sonnet-20241022",
        1024,
        vec![Message::user("Hello, Claude!")],
    );

    // Send the message
    let response = client.send_message(request).await?;

    // Print the response
    for content in &response.content {
        if let claude_sdk::ContentBlock::Text { text, .. } = content {
            println!("{}", text);
        }
    }

    Ok(())
}
```

### Streaming Responses

```rust
use claude_sdk::{ClaudeClient, Message, MessagesRequest, StreamEvent};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClaudeClient::anthropic(
        std::env::var("ANTHROPIC_API_KEY")?
    );

    let request = MessagesRequest::new(
        claude_sdk::models::CLAUDE_SONNET_4_5.anthropic_id,
        1024,
        vec![Message::user("Tell me a story")],
    );

    let mut stream = client.send_streaming(request).await?;

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

    Ok(())
}
```

### Tool Use with Multi-turn Conversations

```rust
use claude_sdk::{ClaudeClient, ConversationBuilder, Tool, StreamEvent};
use futures::StreamExt;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClaudeClient::anthropic(
        std::env::var("ANTHROPIC_API_KEY")?
    );

    // Define a tool
    let weather_tool = Tool {
        name: "get_weather".into(),
        description: "Get weather for a location".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            },
            "required": ["location"]
        }),
        disable_user_input: Some(true), // Programmatic!
        cache_control: None,
    };

    // Build conversation with tool
    let mut conversation = ConversationBuilder::new()
        .with_system("You are a helpful assistant")
        .with_tool(weather_tool);

    conversation.add_user_message("What's the weather in NYC?");

    // First turn - Claude requests tool use
    let request = conversation.build(
        claude_sdk::models::CLAUDE_SONNET_4_5.anthropic_id,
        1024
    );
    let response = client.send_message(request).await?;

    // Extract tool use
    for content in &response.content {
        if let claude_sdk::ContentBlock::ToolUse { id, name, input, .. } = content {
            println!("Claude wants to use: {}", name);

            // Execute tool (your implementation here)
            let result = r#"{"temp": 72, "condition": "sunny"}"#;

            // Add tool result
            conversation.add_tool_result(id, result);
        }
    }

    // Second turn - Claude uses the tool result
    let request = conversation.build(
        claude_sdk::models::CLAUDE_SONNET_4_5.anthropic_id,
        1024
    );
    let response = client.send_message(request).await?;

    // Claude responds with the answer
    for content in &response.content {
        if let claude_sdk::ContentBlock::Text { text, .. } = content {
            println!("Claude: {}", text);
        }
    }

    Ok(())
}
```

### Retry Logic for Production

```rust
use claude_sdk::{ClaudeClient, MessagesRequest, Message};
use claude_sdk::retry::RetryConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClaudeClient::anthropic(
        std::env::var("ANTHROPIC_API_KEY")?
    );

    let request = MessagesRequest::new(
        claude_sdk::models::CLAUDE_SONNET_4_5.anthropic_id,
        1024,
        vec![Message::user("Hello!")],
    );

    // Configure retry behavior
    let retry_config = RetryConfig::new()
        .with_max_attempts(5)
        .with_initial_backoff(std::time::Duration::from_secs(1))
        .with_max_backoff(std::time::Duration::from_secs(60));

    // Automatically retries on rate limits and server errors
    let response = client.send_message_with_retry(request, retry_config).await?;

    Ok(())
}
```

## Examples

Run the examples with your API key:

```bash
export ANTHROPIC_API_KEY="your-api-key"
cargo run --example simple_chat
cargo run --example streaming_chat
cargo run --example tool_use
cargo run --example prompt_caching
```

Available examples:
- `simple_chat` - Basic message sending with complete responses
- `streaming_chat` - Real-time streaming responses with token display
- `tool_use` - Tool definitions, multi-turn conversations, programmatic tool calling
- `prompt_caching` - Cost reduction with prompt caching (90% savings on cached tokens)

## Supported Models

- `claude-3-5-sonnet-20241022` - Most capable, balanced performance
- `claude-3-5-haiku-20241022` - Fast and cost-effective
- `claude-3-opus-20240229` - Most powerful for complex tasks

## API Coverage

### ✅ Implemented

- [x] Basic message sending
- [x] Streaming responses with SSE
- [x] Tool definitions and tool use
- [x] Multi-turn conversations (ConversationBuilder)
- [x] Programmatic tool calling
- [x] Request/response types
- [x] Error handling
- [x] Authentication
- [x] Token usage tracking
- [x] Model registry with constraints
- [x] Bedrock regional/global endpoints
- [x] Prompt caching (CacheControl, cached system/tools, 90% cost reduction)
- [x] Retry logic with exponential backoff
- [x] Respects retry-after headers

### 🚧 In Progress

- [ ] Token counting (tiktoken-rs)

### 📋 Planned

- [ ] AWS Bedrock client implementation
- [ ] Interactive REPL
- [ ] More examples and integration tests

## Development Status

**Phase 1 (Foundation) - COMPLETE ✅**
- Non-streaming API
- Streaming API with SSE
- Model registry with all Claude 4.5/4.x/3.x models
- Comprehensive error handling
- CI/CD automation

**Phase 2 (Tools & Conversations) - COMPLETE ✅**
- Tool definitions and execution
- Multi-turn conversations with ConversationBuilder
- Programmatic tool calling
- Tool result handling
- Retry logic with exponential backoff
- Comprehensive error handling
- Prompt caching (90% cost reduction)

**Phase 3 (Advanced Features) - In Progress 🚧**
- Token counting
- AWS Bedrock client
- Interactive REPL
- More examples and integration tests

**Progress: 11 of 17 features complete (65%)**

See [.claude/system/features.json](.claude/system/features.json) for detailed feature tracking.

## Project Structure

```
claude-agent-sdk-rust/
├── src/
│   ├── lib.rs          # Public API
│   ├── client.rs       # HTTP client
│   ├── types.rs        # Request/response types
│   └── error.rs        # Error types
├── examples/
│   └── simple_chat.rs  # Example usage
└── .claude/
    └── system/         # Development tracking
```

## Development Setup

### Install Git Hooks

We use pre-commit hooks to ensure code quality. Install them with:

```bash
./scripts/install-hooks.sh
```

The pre-commit hook automatically runs:
- `cargo test` - All tests must pass
- `cargo clippy -- -D warnings` - No clippy warnings allowed
- `cargo fmt --check` - Code must be formatted
- `cargo doc` - Documentation must build

To bypass the hook (not recommended): `git commit --no-verify`

### Running Checks Manually

```bash
# Run all checks
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo doc --no-deps --all-features

# Auto-fix formatting
cargo fmt --all
```

### Developer Tools

**Automated Changelog Generation:**

Use Claude to generate changelog entries from git commits:

```bash
export ANTHROPIC_API_KEY="your-api-key"
cargo run --bin update-changelog
```

This analyzes commits since the last release and generates Keep a Changelog format entries.

## Contributing

This project is part of Colony Shell (F015 - Claude ADK integration) but can be used standalone.

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## Design Principles

1. **Native Rust** - No FFI, no subprocess, pure Rust
2. **Type-safe** - Leverage Rust's type system
3. **Async-first** - Built on tokio
4. **API-compliant** - Match official Claude API exactly
5. **Zero-copy** - Minimize allocations where possible

## License

MIT

## Resources

- [Claude API Documentation](https://docs.anthropic.com/en/api/messages)
- [Programmatic Tool Calling](https://platform.claude.com/docs/en/agents-and-tools/tool-use/programmatic-tool-calling)
- [API Streaming](https://docs.anthropic.com/en/api/messages-streaming)
