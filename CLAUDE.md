# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`claude-sdk` is a production-ready Rust SDK for the Claude API (crate name: `claude-sdk`, lib name: `claude_sdk`). It supports both the Anthropic API and AWS Bedrock with a unified interface. Published on crates.io. MSRV: Rust 1.70.

## Build & Development Commands

```bash
# Build
cargo build                          # Default (anthropic feature only)
cargo build --all-features           # All features (anthropic + bedrock + repl)

# Test
cargo test --all-features            # Run all tests
cargo test test_name                 # Run a specific test
cargo test -- --nocapture            # Run tests with stdout
ANTHROPIC_API_KEY=... cargo test -- --ignored  # Integration tests (require real API key)

# Lint & Format
cargo clippy --all-targets --all-features -- -D warnings   # Zero warnings policy
cargo fmt --all -- --check           # Check formatting
cargo fmt --all                      # Auto-fix formatting

# Documentation
cargo doc --no-deps --all-features   # Build docs
cargo doc --no-deps --all-features --open  # Build and open

# Run examples (require ANTHROPIC_API_KEY env var)
cargo run --example simple_chat
cargo run --example streaming_chat
cargo run --example tool_use
cargo run --example prompt_caching

# Binaries
cargo run --features repl --bin claude-repl         # Interactive REPL
cargo run --bin update-changelog                     # Auto-generate changelog
```

## Feature Flags

- `anthropic` (default) — Anthropic API support
- `bedrock` — AWS Bedrock support (adds `aws-config`, `aws-sdk-bedrockruntime`)
- `repl` — Interactive REPL binary (adds `rustyline`, `chrono`)
- `full` — All features

## Architecture

### Core Flow

`ClaudeClient` is the central type. It wraps a `reqwest::Client` and a `ClaudeBackend` enum (Anthropic or Bedrock). All requests go through `MessagesRequest` → `ClaudeClient::send_message()` → `MessagesResponse`.

### Module Map

- **`client.rs`** — `ClaudeClient` with `anthropic()` and `bedrock()` constructors. Handles HTTP, auth headers, Bedrock model ID conversion. Entry points: `send_message()`, `send_streaming()`, `send_message_with_retry()`
- **`types.rs`** — All request/response types. `MessagesRequest`, `MessagesResponse`, `Message`, `ContentBlock` (tagged enum: Text/Image/Document/ToolUse/ToolResult/Thinking), `Tool`, `ToolChoice`
- **`streaming.rs`** — `StreamEvent` enum for SSE parsing. Returns `Pin<Box<dyn Stream<Item = Result<StreamEvent>>>>`
- **`conversation.rs`** — `ConversationBuilder` for multi-turn state management with cached system prompts and tools
- **`models.rs`** — Static `Model` structs (9 models) with metadata: context windows, output limits, pricing, Bedrock IDs, capability flags
- **`error.rs`** — `Error` enum with `is_retryable()` and `retry_after()`. Uses `thiserror`
- **`retry.rs`** — `RetryConfig` with exponential backoff
- **`batch.rs`** — `BatchClient` for bulk processing (up to 100K requests)
- **`files.rs`** — Files API client for uploads
- **`tokens.rs`** — Token counting via `tiktoken-rs`
- **`prompts.rs`** — Pre-built system prompts
- **`structured.rs`** — Structured output / forced JSON helpers

### Key Design Decisions

- **Dual model IDs**: Models store both `anthropic_id` and `bedrock_id` because Bedrock uses different ID formats (e.g., `anthropic.claude-sonnet-4-5-20250929-v1:0`)
- **`Pin<Box<dyn Stream>>` for streaming**: Allows internal refactoring without breaking the public API
- **No `Default` for `MessagesRequest`**: Forces explicit model selection — `MessagesRequest::new(model, max_tokens, messages)` is always required
- **`ContentBlock` is a tagged enum**: Uses `#[serde(tag = "type", rename_all = "snake_case")]` to match the Claude API JSON format exactly

## Code Conventions

- **No `unwrap()` in library code** — propagate errors with `?`
- **`thiserror` for library errors, `anyhow` for examples/binaries**
- **All public items need doc comments** with `# Example` sections
- **Match the Claude API exactly** — field names, types, serialization must mirror the API spec
- **Conventional Commits**: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `chore:`
- Pre-commit hooks run tests, clippy, fmt, and doc checks automatically
