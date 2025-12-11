# Claude SDK for Rust

A native Rust implementation of the Claude API client with streaming support, tool execution, and programmatic tool calling.

## Features

- ✅ **Non-streaming API** - Send messages and get complete responses
- 🚧 **Streaming API** - Stream responses with Server-Sent Events (coming soon)
- 🚧 **Tool Use** - Define and execute tools with programmatic calling (coming soon)
- 🚧 **Prompt Caching** - Reduce costs with prompt caching support (coming soon)
- 🚧 **AWS Bedrock** - Support for Claude models via AWS Bedrock (coming soon)
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

## Examples

Run the examples with your API key:

```bash
export ANTHROPIC_API_KEY="your-api-key"
cargo run --example simple_chat
```

Available examples:
- `simple_chat` - Basic message sending
- `streaming_chat` - Streaming responses (coming soon)

## Supported Models

- `claude-3-5-sonnet-20241022` - Most capable, balanced performance
- `claude-3-5-haiku-20241022` - Fast and cost-effective
- `claude-3-opus-20240229` - Most powerful for complex tasks

## API Coverage

### ✅ Implemented

- [x] Basic message sending
- [x] Request/response types
- [x] Error handling
- [x] Authentication
- [x] Token usage tracking

### 🚧 In Progress

- [ ] Streaming responses (SSE)
- [ ] Tool definitions
- [ ] Tool use and results
- [ ] Multi-turn conversations
- [ ] Prompt caching

### 📋 Planned

- [ ] AWS Bedrock support
- [ ] Token counting
- [ ] Retry logic with backoff
- [ ] Interactive REPL

## Development Status

This is an early-stage project. The basic non-streaming API is functional, but many features are still in development. See [.claude/system/features.json](.claude/system/features.json) for detailed feature tracking.

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

## Contributing

This project is part of Colony Shell (F015 - Claude ADK integration) but can be used standalone.

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
