# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial implementation of Claude SDK for Rust
- Non-streaming API (`send_message()`)
- Streaming API with SSE support (`send_streaming()`)
- Comprehensive Model registry with all Claude 4.5, 4.x, and 3.x models
- Model metadata: context windows, output limits, pricing, capabilities
- AWS Bedrock endpoint support (regional, global, us, eu, ap prefixes)
- Extended context window tracking (1M tokens for Sonnet 4.5/4.0)
- Prompt caching types (CacheControl)
- Tool use types (Tool, ToolUse, ToolResult)
- Comprehensive error handling with retry-after support
- Examples: `simple_chat`, `streaming_chat`
- Full documentation and API docs
- GitHub Actions CI/CD pipeline
- Upstream SDK monitoring workflow

### Changed
- N/A (initial release)

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A

## [0.1.0]

### Added
- Core Claude API client with support for all Claude models (4.5, 4.x, and 3.x families)
- Streaming support via Server-Sent Events (SSE) for real-time message responses
- Non-streaming message API for synchronous interactions
- Comprehensive model registry with metadata including context windows, pricing, and capabilities
- Support for extended context up to 1M tokens for Sonnet models
- AWS Bedrock regional endpoints support (standard, global, US, EU, and AP regions)
- Robust error handling with automatic retry-after parsing for rate limits
- Complete authentication system for API requests
- Two example implementations:
  - Simple chat for basic non-streaming usage
  - Streaming chat with real-time token statistics
- Developer tooling:
  - Pre-commit git hooks for testing, linting, formatting, and documentation checks
  - Automated changelog generator
  - GitHub Actions CI/CD pipeline with testing, releases, and monitoring
- Comprehensive documentation including README, contributing guidelines, and release processes
- 18 unit tests and 8 documentation tests with full coverage

## [0.1.1]

### Fixed
- Fix release automation workflow that was failing due to deprecated GitHub action, ensuring releases are published successfully with improved formatting and automatic release notes generation

## [1.0.0]

### Added

- **Batch Processing API**: Submit up to 100,000 requests asynchronously with 50% cost discount on all tokens. Includes batch creation, status tracking, cancellation, automatic polling, and efficient JSONL result streaming. Results available for 29 days.

- **Extended Thinking API**: Enable Claude's extended reasoning capabilities for complex problem-solving tasks.

- **Files API and Vision Support**: Upload files up to 500 MB (images, PDFs, text files, datasets) and reference them by ID in messages. Upload and storage operations are free; files are charged as input tokens when used in conversations.

- **Search Results and Citations**: Add search results as content blocks with automatic source citations. Enables RAG applications with built-in citation tracking. Claude automatically attributes responses to sources using `search_result_location`.

- **Document Blocks**: Attach PDFs and text documents to messages using file IDs or base64 encoding, with optional titles, context, and citation support.

- **Structured Outputs**: Control tool execution with `tool_choice` parameter (auto/any/specific tool/none), input examples for tools (beta), and ability to disable parallel tool execution. Includes `pause_turn` stop reason for long-running server operations.

- **Effort Parameter**: Control response quality vs. token usage trade-offs with high/medium/low effort levels (Claude Opus 4.5 only).

- **Interactive REPL**: Command-line interface for interactive conversations with Claude.

- **Token Counting**: Estimate token usage for messages, conversations, and requests using the cl100k_base tokenizer. Validate requests fit within context windows (up to 1M tokens with extended context) before sending to avoid errors.

- **AWS Bedrock Support**: Full streaming and non-streaming API support for Claude models on AWS Bedrock with automatic SigV4 signing and credential management.

- **System Prompts Library**: Pre-built prompts including Claude Code system prompt for software engineering agents, coding assistant, RAG assistant with citations, JSON extraction, and parallel tool use guidance.

- **Prompt Caching**: Cache system prompts, tool definitions, and large context blocks for 90% cost reduction on repeated content. Cache writes cost +25% once; cache reads save -90% (5-minute cache duration).

- **Retry Logic with Exponential Backoff**: Automatic retry handling for rate limits (429) and server errors (5xx) with configurable backoff parameters and respect for API retry-after headers.

- **Tool Use and Multi-Turn Conversations**: Define tools with JSON schemas, handle tool execution requests from Claude, provide results, and manage complete multi-turn agentic workflows with `ConversationBuilder`.

- **Conversation Management**: Track message history, manage system prompts (with caching), add/remove tools, and clear messages while preserving context.

### Changed

- Token counting now integrates with `ConversationBuilder` via `estimate_tokens()` and `fits_in_context()` methods for proactive context management

- Tool definitions now support input examples, parallel execution control, and forced tool selection modes

- Context window validation now supports extended context limits up to 1M tokens

### Breaking Changes

None - all additions are backwards compatible with existing APIs.