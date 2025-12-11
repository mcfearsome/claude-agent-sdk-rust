# Claude SDK Roadmap

This document outlines the current status and future plans for the Claude SDK for Rust.

## Current Status (v0.2.x)

### ✅ Completed Features (11/17 core features)

**Phase 1: Foundation**
- [x] SDK001: HTTP client with authentication
- [x] SDK002: Request/response types
- [x] SDK003: Non-streaming API
- [x] SDK004: SSE streaming parser
- [x] SDK005: Streaming API
- [x] SDK017: CI/CD pipeline

**Phase 2: Tools & Conversations**
- [x] SDK006: Tool use primitives
- [x] SDK007: Multi-turn conversation builder
- [x] SDK009: Error handling taxonomy
- [x] SDK010: Retry logic with exponential backoff
- [x] SDK013: Prompt caching support

### 🚧 Core Features In Progress (6 remaining)

**High Priority:**
- [ ] SDK008: Token counting (tiktoken-rs) - Needed for context management
- [ ] SDK014: AWS Bedrock client - SigV4 signing, Bedrock endpoints
- [ ] SDK011: More examples and integration tests

**Medium Priority:**
- [ ] SDK012: Final documentation polish
- [ ] SDK015: Interactive REPL for testing
- [ ] SDK016: Publish to crates.io

## Advanced Features (Future)

See `.claude/system/future-features.json` for complete list.

### API Features (Supported by Claude)

**Immediate Candidates:**
- **Vision (SDK019)** - Image inputs, already supported by models
- **Extended Thinking (SDK018)** - Show reasoning process, beta header available
- **Structured Outputs (SDK020)** - Force JSON schema with tool_choice
- **Batch API (SDK021)** - 50% discount, async processing

**Nice to Have:**
- **PDF Support (SDK022)** - Build on vision API
- **Context Editing (SDK024)** - Smart truncation and summarization

### Integration Features

**MCP Integration (SDK029)** - High Priority for Colony Shell:
```
MCP Server → Tool Definitions → Claude SDK → Responses
```

Architecture:
1. MCP Client connects to servers (stdio/SSE/HTTP)
2. Adapter converts MCP tools → Claude Tool format
3. Execution bridge runs MCP tool calls
4. SDK sends to Claude with tool results

Design:
- Optional feature flag: `mcp`
- Keep SDK independent of MCP
- Colony Shell will use both together

**Application-Level Features:**
- **Citations (SDK025)** - Parse source references in responses
- **Multilingual (SDK027)** - Conveniences for non-English conversations

### Currently Blocked

These depend on Anthropic releasing new API features:
- **Embeddings (SDK023)** - Anthropic doesn't have embeddings API yet
- **Files API (SDK026)** - File uploads not available via API
- **Effort Parameter (SDK028)** - Not in current API

## Current Capabilities

The SDK can already:
✅ Send messages (streaming and non-streaming)
✅ Use tools with programmatic calling
✅ Manage multi-turn conversations
✅ Cache prompts for 90% cost reduction
✅ Retry on rate limits and errors
✅ Handle all Claude 4.5/4.x/3.x models
✅ Support Bedrock model IDs (client pending)

**Missing for production:**
- Token counting (SDK008) - For context management
- Integration tests (SDK011) - Against real API
- Bedrock client (SDK014) - For AWS users

## Priority for Next Release (v0.3.0)

**Must Have:**
1. SDK008: Token counting
2. SDK011: Integration tests
3. SDK014: Bedrock client (at least basic support)

**Should Have:**
4. SDK019: Vision support (images)
5. SDK020: Structured outputs (tool_choice)

**Nice to Have:**
6. SDK018: Extended thinking
7. SDK015: Interactive REPL

## Long-term Vision (v1.0.0)

**Goals for 1.0.0 release:**
- All core SDK features (SDK001-017) complete
- Vision support (SDK019)
- Structured outputs (SDK020)
- Bedrock fully supported (SDK014)
- Token counting (SDK008)
- Comprehensive examples
- Integration tests
- Published to crates.io
- Stable API (no more breaking changes)

**Timeline:**
- v0.3.0: Add vision, structured outputs, token counting
- v0.4.0: Bedrock client, integration tests
- v0.5.0: Extended thinking, REPL
- v1.0.0: Stable release with full API coverage

## Contributing

Want to help implement these features? See:
- `CONTRIBUTING.md` - Development guidelines
- `.claude/system/features.json` - Core features (SDK001-017)
- `.claude/system/future-features.json` - Advanced features (SDK018+)

Highest impact contributions:
1. Token counting implementation
2. Vision support (image inputs)
3. Integration tests
4. AWS Bedrock client

## Tracking Updates

**Official SDKs to monitor:**
- TypeScript: https://github.com/anthropics/anthropic-sdk-typescript
- Python: https://github.com/anthropics/anthropic-sdk-python
- API Changelog: https://docs.anthropic.com/en/api/changelog

**Automated monitoring:**
- GitHub Action runs weekly (`.github/workflows/upstream-check.yml`)
- Creates issues when new models detected
- Tracks TypeScript/Python SDK versions
