# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.1.0] - 2026-06-23

### Added
- **Container support**: `container` field on `MessagesRequest` (`with_container()`), `Container` struct on response, `ContainerUpload` content block
- **Mid-conversation system blocks**: `MidConvSystem` content block variant for injecting system instructions mid-conversation
- **Memory tool**: `MemoryTool` in `server_tools` module for persistent memory across conversations
- **Tool search tools**: `ToolSearchBm25` and `ToolSearchRegex` in `server_tools` for searching over large tool sets
- **CustomTool enhancements**: `defer_loading`, `eager_input_streaming`, `strict` optional fields

## [2.0.0] - 2026-06-23

### Added
- **Server tools module** (`server_tools`): `WebSearchTool`, `WebFetchTool`, `CodeExecutionTool`, `BashTool`, `TextEditorTool` with ergonomic constructors and builder methods
- **`ToolDefinition` enum**: Wraps `Custom(CustomTool)` and `Server(Value)` — replaces `Vec<Tool>` in `MessagesRequest.tools`
- **Server tool content blocks**: `ServerToolUse`, `WebSearchToolResult`, `CodeExecutionToolResult` variants on `ContentBlock`
- **Structured output**: `OutputConfig.format` with `OutputFormat` for JSON schema-based output via `with_json_schema()`
- **Adaptive thinking**: `ThinkingConfig::Adaptive` variant + `ThinkingDisplay` enum (`Summarized`, `Omitted`)
- **Token counting endpoint**: `ClaudeClient::count_tokens()` — server-side `POST /v1/messages/count_tokens`
- **`ToolResultContent` union**: Tool results accept either `Text(String)` or `Blocks(Vec<ContentBlock>)`
- **`caller` field** on `ContentBlock::ToolUse`: tracks which system invoked the tool

### Changed
- `Tool` renamed to `CustomTool` (deprecated type alias `Tool` preserved for migration)
- `MessagesRequest.tools` type changed from `Option<Vec<Tool>>` to `Option<Vec<ToolDefinition>>`
- `ToolChoice::Auto`, `Any`, `Tool` are now struct variants containing `disable_parallel_tool_use: Option<bool>`
- `MessagesRequest.disable_parallel_tool_use` removed (moved into `ToolChoice` variants)
- `ThinkingConfig::Enabled` gains `display: Option<ThinkingDisplay>` field
- `ToolResult.content` type changed from `Option<String>` to `Option<ToolResultContent>`
- `OutputConfig` no longer implements `Eq` (contains `serde_json::Value`)

## [1.1.0] - 2026-06-19

### Added
- **New models**: Claude Sonnet 4.6, Opus 4.6, Opus 4.7, Opus 4.8, Fable 5, Mythos 5 model constants with full metadata
- **Request metadata**: `Metadata` struct with `user_id` for abuse detection via `with_metadata()`
- **Service tier**: `ServiceTier` enum (`Auto`, `StandardOnly`) for priority routing via `with_service_tier()`
- **Inference geo**: Geographic region routing via `with_inference_geo()`
- **Refusal handling**: `StopReason::Refusal` with `StopDetails` struct (category: `Cyber`/`Bio`/`ReasoningExtraction`, explanation)
- **Effort levels**: `EffortLevel::XHigh` and `EffortLevel::Max` variants
- **Cache TTL**: `CacheTtl` enum (`FiveMinutes`, `OneHour`) and `CacheControl::ephemeral_with_ttl()` constructor
- **Usage details**: `OutputTokensDetails` (thinking tokens), `ServerToolUsage` (web search/fetch counts), `service_tier` and `inference_geo` on `Usage`
- **Forward compatibility**: `ContentBlock::Unknown` variant catches unrecognized content block types instead of causing deserialization errors
- **Rate limit info**: `RateLimitInfo` struct parsed from API response headers (`requests_remaining`, `tokens_remaining`, reset times)

### Fixed
- Doc test in `error.rs` referencing non-existent `with_max_retries` method (now uses `with_max_attempts`)

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