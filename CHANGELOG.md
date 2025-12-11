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

## [0.1.0] - TBD

Initial release (pending).
