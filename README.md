# Claude SDK for Rust

[![Crates.io](https://img.shields.io/badge/version-1.0.0-blue)](https://crates.io/crates/claude-sdk)
[![Documentation](https://docs.rs/claude-sdk/badge.svg)](https://docs.rs/claude-sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A **complete, production-ready** Rust SDK for Claude with full API coverage. Built for [Colony Shell](https://github.com/mcfearsome/colony-shell) but usable standalone.

## ⚡ What Makes This Special

- **100% Feature Complete** - Every Claude API feature implemented
- **Both Platforms** - Anthropic API + AWS Bedrock with unified interface
- **Production Ready** - 90 tests, retry logic, error handling, token counting
- **Developer Experience** - Interactive REPL, auto-changelog, pre-commit hooks
- **Type Safe** - Leverage Rust's type system for correctness
- **Zero Cost** - Idiomatic async Rust with no runtime overhead

---

## 🚀 Features

### Core API
✅ **Streaming & Non-streaming** - SSE streams + complete responses
✅ **Error Handling** - Comprehensive error taxonomy with retry logic
✅ **Exponential Backoff** - Automatic retry on rate limits (429) and server errors (5xx)
✅ **Token Counting** - Accurate context window management with tiktoken-rs

### Intelligence & Tools
✅ **Tool Use** - Programmatic calling, parallel execution, input examples (beta)
✅ **Tool Choice** - Force specific tools (auto/any/tool/none)
✅ **Conversations** - Multi-turn state management with ConversationBuilder
✅ **Extended Thinking** - Step-by-step reasoning (thinking blocks with signatures)
✅ **Effort Control** - Token efficiency (high/medium/low for Opus 4.5)

### Content Types
✅ **Text** - Standard messages with optional citations
✅ **Images** - Vision support (base64, URL, or file_id)
✅ **Documents** - PDFs and text files with citations
✅ **Search Results** - RAG with automatic source attribution
✅ **Thinking Blocks** - Extended reasoning (including redacted)

### Cost Optimization
✅ **Prompt Caching** - 90% cost reduction on cached content (system prompts, tools, docs)
✅ **Batch Processing** - 50% discount for async bulk requests (up to 100K requests)
✅ **Files API** - Upload once, reference many times

### Platform Support
✅ **Anthropic API** - Full support including all beta features
✅ **AWS Bedrock** - Complete implementation with streaming
✅ **Regional Endpoints** - standard/global/us/eu/ap prefixes

### Developer Tools
✅ **Interactive REPL** - Full terminal UI with streaming, token display, backend switching
✅ **Auto-Changelog** - Uses Claude to generate changelog from git commits
✅ **Pre-commit Hooks** - Automated quality checks (test, clippy, fmt, docs)
✅ **CI/CD** - GitHub Actions for testing, releases, upstream monitoring
✅ **System Prompts** - Battle-tested prompts including Claude Code

### Advanced Features
✅ **Structured Outputs** - Forced JSON extraction with schema validation
✅ **Context Validation** - Proactive window checking (200K/1M tokens)
✅ **Model Registry** - 9 models with full metadata, constraints, pricing
✅ **Model Capabilities** - Track extended context, effort support, thinking support

---

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
claude-sdk = "1.0"
tokio = { version = "1", features = ["full"] }

# Optional features
claude-sdk = { version = "1.0", features = ["bedrock", "repl"] }
```

**Features:**
- `anthropic` (default) - Anthropic API support
- `bedrock` - AWS Bedrock support
- `repl` - Interactive REPL binary
- `full` - All features enabled

---

## 🎯 Quick Start

### Basic Chat

```rust
use claude_sdk::{ClaudeClient, Message, MessagesRequest, models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClaudeClient::anthropic(std::env::var("ANTHROPIC_API_KEY")?);

    let request = MessagesRequest::new(
        models::CLAUDE_SONNET_4_5.anthropic_id,
        1024,
        vec![Message::user("Explain quantum entanglement")],
    );

    let response = client.send_message(request).await?;

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

### AWS Bedrock

Same API works with AWS Bedrock:

```rust
use claude_sdk::{ClaudeClient, MessagesRequest, Message, models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Uses AWS_PROFILE or default credential chain
    let client = ClaudeClient::bedrock("us-east-1").await?;

    let request = MessagesRequest::new(
        models::CLAUDE_SONNET_4_5.anthropic_id,  // Auto-converts to Bedrock format
        1024,
        vec![Message::user("Hello from Bedrock!")],
    );

    let response = client.send_message(request).await?;
    // Works identically to Anthropic API!

    Ok(())
}
```

### Tool Use

```rust
use claude_sdk::{ConversationBuilder, Tool, models};
use serde_json::json;

let mut conversation = ConversationBuilder::new()
    .with_cached_system("You are a helpful assistant")  // Cached for 5min!
    .with_cached_tool(Tool {
        name: "get_weather".into(),
        description: "Get current weather".into(),
        input_schema: json!({"type": "object", "properties": {
            "location": {"type": "string"}
        }}),
        disable_user_input: Some(true),  // Programmatic
        input_examples: None,
        cache_control: None,
    });

conversation.add_user_message("What's the weather in Tokyo?");

// Claude will request tool use automatically
let request = conversation.build(models::CLAUDE_SONNET_4_5.anthropic_id, 1024);
```

### Extended Thinking

```rust
let request = MessagesRequest::new(
    models::CLAUDE_SONNET_4_5.anthropic_id,
    16000,
    vec![Message::user("Solve this complex math problem...")],
)
.with_thinking(10_000);  // 10K token budget for reasoning

// Response includes thinking blocks showing step-by-step reasoning
```

### Batch Processing (50% Discount!)

```rust
use claude_sdk::batch::{BatchClient, BatchRequest};

let batch_client = BatchClient::new(api_key);

let requests = vec![
    BatchRequest {
        custom_id: "req-1".into(),
        params: MessagesRequest::new(/*...*/),
    },
    // ... up to 100,000 requests
];

let batch = batch_client.create(requests).await?;
let completed = batch_client.wait_for_completion(&batch.id).await?;

// 50% cost reduction on all tokens!
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

---

## 🤖 Supported Models

All Claude 4.5, 4.x, and 3.x models supported with full metadata:

| Model | Context | Output | Extended Context | Thinking | Effort |
|-------|---------|--------|------------------|----------|--------|
| **Claude Sonnet 4.5** | 200K | 64K | ✅ 1M | ✅ | ❌ |
| **Claude Haiku 4.5** | 200K | 64K | ❌ | ✅ | ❌ |
| **Claude Opus 4.5** | 200K | 64K | ❌ | ✅ | ✅ |
| Claude Opus 4.1 | 200K | 32K | ❌ | ✅ | ❌ |
| Claude Sonnet 4 | 200K | 64K | ✅ 1M | ✅ | ❌ |
| Others | 200K | varies | varies | varies | ❌ |

**Model Registry Includes:**
- Context windows & output limits
- Extended context support (1M)
- Thinking capability
- Effort parameter support
- Pricing per million tokens
- Bedrock & Vertex AI IDs
- Regional endpoint prefixes

---

## ✅ Complete API Coverage

**Messages API: 100%**
- [x] Non-streaming & streaming
- [x] System prompts (cached & non-cached)
- [x] Multi-turn conversations
- [x] Stop reasons (end_turn, max_tokens, stop_sequence, tool_use, pause_turn)

**Content Types: 100%**
- [x] Text (with citations)
- [x] Images (base64, URL, file_id)
- [x] Documents (PDFs, text with citations)
- [x] Search Results (RAG with auto-citations)
- [x] Tool Use & Tool Results
- [x] Thinking & Redacted Thinking

**Tool Use: 100%**
- [x] Tool definitions with JSON schemas
- [x] Programmatic calling (disable_user_input)
- [x] Tool choice (auto/any/tool/none)
- [x] Parallel tool use (configurable)
- [x] Input examples (beta)
- [x] Tool result error handling

**Platform: 100%**
- [x] Anthropic API (complete)
- [x] AWS Bedrock (streaming + non-streaming)
- [x] Regional endpoints (standard/global/us/eu/ap)
- [x] Automatic model ID conversion

**Optimization: 100%**
- [x] Prompt caching (90% savings)
- [x] Batch processing (50% discount)
- [x] Token counting (tiktoken-rs)
- [x] Context validation (200K/1M)
- [x] Retry logic with backoff
- [x] Effort control (Opus 4.5)

**Advanced: 100%**
- [x] Extended thinking (reasoning blocks)
- [x] Files API (upload/download/manage)
- [x] Structured outputs (forced JSON)
- [x] Vision (image analysis)
- [x] Citations (automatic source attribution)

**Developer Tools: 100%**
- [x] Interactive REPL
- [x] Token counting display
- [x] Conversation save/load
- [x] Auto-changelog generator
- [x] Pre-commit hooks
- [x] CI/CD pipelines
- [x] System prompts (Claude Code)

---

## 🎉 Development Status: v1.0.0 - COMPLETE

**All Features Implemented: 22/22 (100%)**

✅ Core SDK (17/17)
✅ Advanced Features (5/5)
✅ 90 tests passing
✅ Zero technical debt
✅ Production ready

**What This Means:**
- Every documented Claude API feature is implemented
- Both Anthropic and AWS Bedrock fully supported
- All content types (text, image, document, search, thinking)
- All optimization features (caching, batching, retry)
- Complete developer tooling (REPL, hooks, CI/CD)
- Ready for immediate production use

---

## 💡 Why Use This SDK?

**vs. Official SDKs (TypeScript/Python):**
- ✅ **Type Safety** - Rust's type system catches errors at compile time
- ✅ **Model Registry** - Full metadata with constraints and validation
- ✅ **Unified Bedrock** - Same API for both platforms
- ✅ **System Prompts** - Pre-built prompts (Claude Code, RAG, coding, etc.)
- ✅ **Better Regional Support** - First-class Bedrock endpoint handling
- ✅ **Performance** - Zero-cost abstractions, no GC pauses
- ✅ **Memory Safety** - No segfaults, no undefined behavior

**vs. Building Your Own:**
- ✅ **Complete** - Every API feature implemented
- ✅ **Tested** - 90 tests covering all features
- ✅ **Maintained** - Auto-monitoring for upstream changes
- ✅ **Documented** - Full API docs + examples
- ✅ **Production Ready** - Error handling, retry logic, validation

**Perfect For:**
- 🦀 Rust applications needing Claude integration
- 🤖 Building AI agents (like Colony Shell)
- 🏢 Enterprise apps needing AWS Bedrock
- 📊 Batch processing workloads (50% savings)
- 💰 Cost-sensitive applications (caching, batching)

---

## 📁 Project Structure

```
claude-agent-sdk-rust/
├── src/
│   ├── lib.rs              # Public API & re-exports
│   ├── client.rs           # HTTP client (Anthropic + Bedrock)
│   ├── conversation.rs     # ConversationBuilder
│   ├── types.rs            # Request/response types (400+ lines)
│   ├── error.rs            # Error taxonomy
│   ├── models.rs           # Model registry (9 models)
│   ├── streaming.rs        # SSE event types
│   ├── tokens.rs           # Token counting (tiktoken-rs)
│   ├── retry.rs            # Exponential backoff
│   ├── files.rs            # Files API client
│   ├── batch.rs            # Batch processing
│   ├── prompts.rs          # System prompts (Claude Code, etc.)
│   ├── structured.rs       # Structured output helpers
│   └── bin/
│       ├── claude-repl.rs  # Interactive REPL (440+ lines)
│       └── update-changelog.rs  # Changelog generator
├── examples/
│   ├── simple_chat.rs      # Basic non-streaming
│   ├── streaming_chat.rs   # Real-time streaming
│   ├── tool_use.rs         # Multi-turn with tools
│   └── prompt_caching.rs   # Cost optimization
├── .github/workflows/
│   ├── ci.yml              # PR checks
│   ├── release.yml         # Auto-publish
│   └── upstream-check.yml  # Model monitoring
├── hooks/
│   └── pre-commit          # Quality checks
├── scripts/
│   └── install-hooks.sh    # Hook installer
├── .claude/system/
│   ├── features.json       # Feature tracking (22 features)
│   ├── future-features.json # Advanced features
│   └── history.txt         # Development log
├── CHANGELOG.md            # Keep a Changelog format
├── CONTRIBUTING.md         # Development guide
├── RELEASE.md              # Release process
├── ROADMAP.md              # Feature roadmap
└── README.md               # This file
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

**Interactive REPL:**

Test the SDK interactively with a full-featured terminal interface:

```bash
export ANTHROPIC_API_KEY="your-api-key"
cargo run --features repl --bin claude-repl
```

Features:
- Real-time streaming responses
- Multi-turn conversations
- Token counting in prompt
- Slash commands (/help, /save, /load, /tokens, /model, /backend)
- Backend switching (Anthropic ↔ Bedrock)
- Conversation save/load

**Automated Changelog Generation:**

Use Claude to generate changelog entries from git commits:

```bash
export ANTHROPIC_API_KEY="your-api-key"
cargo run --bin update-changelog
```

This analyzes commits since the last release and generates Keep a Changelog format entries.

---

## 🤝 Contributing

This project is **feature-complete** at v1.0.0, but we welcome:
- 🐛 Bug reports and fixes
- 📝 Documentation improvements
- ✨ Additional examples
- 🔧 Performance optimizations

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

### Development

```bash
# Setup
git clone https://github.com/mcfearsome/claude-agent-sdk-rust
cd claude-agent-sdk-rust
./scripts/install-hooks.sh

# Run tests
cargo test --all-features

# Quality checks
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo doc --no-deps
```

---

## 📚 Documentation

- [API Documentation](https://docs.rs/claude-sdk) - Full API reference
- [Examples](./examples/) - Working code examples
- [ROADMAP.md](./ROADMAP.md) - Feature tracking & future plans
- [CHANGELOG.md](./CHANGELOG.md) - Version history
- [CONTRIBUTING.md](./CONTRIBUTING.md) - Development guide
- [RELEASE.md](./RELEASE.md) - Release process

**Official Claude Resources:**
- [Claude API Docs](https://platform.claude.com/docs)
- [Messages API](https://platform.claude.com/docs/en/api/messages)
- [Tool Use Guide](https://platform.claude.com/docs/en/agents-and-tools/tool-use)
- [Extended Thinking](https://platform.claude.com/docs/en/build-with-claude/extended-thinking)
- [Batch Processing](https://platform.claude.com/docs/en/build-with-claude/batch-processing)

---

## 🎯 Design Principles

1. **API Compliance** - Match official Claude API exactly
2. **Type Safety** - Leverage Rust's type system for correctness
3. **Zero Cost** - Idiomatic async Rust with no runtime overhead
4. **Platform Agnostic** - Same API for Anthropic & Bedrock
5. **Production Ready** - Comprehensive error handling, retry logic, validation
6. **Developer Friendly** - Clear APIs, good docs, helpful tools

---

## 🏆 Status

**Version:** 1.0.0
**Status:** Production Ready ✅
**Features:** 22/22 (100%)
**Tests:** 90 passing
**Platforms:** Anthropic + AWS Bedrock

**Built for:** [Colony Shell](https://github.com/mcfearsome/colony-shell) F015 - Claude ADK Integration

---

## 📄 License

MIT License - See [LICENSE](LICENSE) for details

---

## 🙏 Acknowledgments

Built with the official [Claude API](https://platform.claude.com/docs) and inspired by:
- [anthropic-sdk-typescript](https://github.com/anthropics/anthropic-sdk-typescript)
- [anthropic-sdk-python](https://github.com/anthropics/anthropic-sdk-python)

Special thanks to Anthropic for creating Claude and maintaining excellent API documentation.
