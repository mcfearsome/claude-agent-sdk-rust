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