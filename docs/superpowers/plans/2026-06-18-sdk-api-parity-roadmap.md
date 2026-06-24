# Claude SDK API Parity Roadmap

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring the `claude-sdk` Rust crate to full parity with the current Claude Messages API across three incremental releases.

**Architecture:** v1.1 adds missing fields/variants to existing types (non-breaking, additive). v2.0 restructures `Tool` into a tagged enum for server-side tools and overhauls `ContentBlock` (breaking). v2.1 adds niche features (containers, MCP, memory tool).

**Tech Stack:** Rust 1.70+, serde, reqwest, tokio, thiserror, tiktoken-rs

---

## Release Overview

| Release | Scope | Breaking? | Key Changes |
|---------|-------|-----------|-------------|
| **v1.1** | Forward compat + quick wins | No | New models, missing request params, usage details, effort levels, cache TTL, serde resilience |
| **v2.0** | Server tools + type overhaul | Yes | `Tool` enum, new `ContentBlock` variants, structured output schema, adaptive thinking, token counting endpoint |
| **v2.1** | Niche & experimental | No | Containers, MCP, memory tool, tool search, mid-conversation system blocks |

---

## Chunk 1: v1.1 — Forward Compatibility & Quick Wins

All changes in v1.1 are additive (new fields with `Option`, new enum variants with `#[serde(other)]`). No existing public API signatures change.

### Task 1: Add New Model Constants

**Files:**
- Modify: `src/models.rs`

- [ ] **Step 1: Write failing tests for new models**

```rust
// Add at bottom of mod tests in src/models.rs
#[test]
fn test_new_models_exist() {
    assert_eq!(CLAUDE_SONNET_4_6.family, "sonnet");
    assert_eq!(CLAUDE_OPUS_4_6.family, "opus");
    assert_eq!(CLAUDE_OPUS_4_7.family, "opus");
    assert_eq!(CLAUDE_OPUS_4_8.family, "opus");
    assert_eq!(CLAUDE_FABLE_5.family, "fable");
    assert_eq!(CLAUDE_MYTHOS_5.family, "mythos");
}

#[test]
fn test_new_model_lookup() {
    assert!(get_model("claude-sonnet-4-6-20260401").is_some());
    assert!(get_model("claude-opus-4-6-20260301").is_some());
    assert!(get_model("claude-fable-5-20260501").is_some());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_new_models_exist -- --nocapture`
Expected: FAIL — `CLAUDE_SONNET_4_6` not found

- [ ] **Step 3: Add model constants**

Add new `Model` consts above the `ALL_MODELS` array. Use the model IDs from the current API docs. Note: exact version dates need verification against the live API — use placeholder dates from the docs and mark with a `// TODO: verify date` comment if unsure.

```rust
/// Claude Sonnet 4.6
pub const CLAUDE_SONNET_4_6: Model = Model {
    name: "Claude Sonnet 4.6",
    family: "sonnet",
    version: "2026-04-01", // TODO: verify date from API
    anthropic_id: "claude-sonnet-4-6-20260401",
    bedrock_id: Some("anthropic.claude-sonnet-4-6-20260401-v1:0"),
    bedrock_global_id: Some("global.anthropic.claude-sonnet-4-6-20260401-v1:0"),
    vertex_id: Some("claude-sonnet-4-6@20260401"),
    max_context_tokens: 200_000,
    max_context_tokens_extended: Some(1_000_000),
    max_output_tokens: 64_000,
    supports_vision: true,
    supports_tools: true,
    supports_caching: true,
    supports_extended_thinking: true,
    supports_effort: false,
    cost_per_mtok_input: 3.0,  // TODO: verify pricing
    cost_per_mtok_output: 15.0,
    description: "Best balance of speed and intelligence",
};

// Repeat pattern for: CLAUDE_OPUS_4_6, CLAUDE_OPUS_4_7, CLAUDE_OPUS_4_8,
// CLAUDE_FABLE_5, CLAUDE_MYTHOS_5, CLAUDE_MYTHOS_PREVIEW
// Use API docs for exact IDs, dates, and pricing.
```

Update `ALL_MODELS` to include the new entries at the top (latest first).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test test_new_models -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/models.rs
git commit -m "feat: add Claude 4.6, 4.7, 4.8, Fable 5, Mythos 5 model constants"
```

---

### Task 2: Add Missing Request Parameters (`metadata`, `service_tier`, `inference_geo`)

**Files:**
- Modify: `src/types.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn test_metadata_serialization() {
    let request = MessagesRequest::new(
        "claude-sonnet-4-5-20250929",
        1024,
        vec![Message::user("Hello")],
    )
    .with_metadata(Metadata { user_id: Some("user-abc-123".into()) });

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["metadata"]["user_id"], "user-abc-123");
}

#[test]
fn test_service_tier_serialization() {
    let request = MessagesRequest::new(
        "claude-sonnet-4-5-20250929",
        1024,
        vec![Message::user("Hello")],
    )
    .with_service_tier(ServiceTier::StandardOnly);

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["service_tier"], "standard_only");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_metadata_serialization -- --nocapture`
Expected: FAIL — `Metadata` not found

- [ ] **Step 3: Add types and builder methods**

In `src/types.rs`, add:

```rust
/// Request metadata for abuse detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Opaque user identifier (uuid or hash, no PII)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

/// Service tier for request routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTier {
    /// Use priority capacity if available, fall back to standard
    Auto,
    /// Only use standard capacity
    StandardOnly,
}
```

Add fields to `MessagesRequest`:

```rust
/// Metadata for abuse detection
#[serde(skip_serializing_if = "Option::is_none")]
pub metadata: Option<Metadata>,

/// Service tier: priority vs standard capacity
#[serde(skip_serializing_if = "Option::is_none")]
pub service_tier: Option<ServiceTier>,

/// Geographic region for inference routing
#[serde(skip_serializing_if = "Option::is_none")]
pub inference_geo: Option<String>,
```

Initialize to `None` in `MessagesRequest::new()`.

Add builder methods:

```rust
pub fn with_metadata(mut self, metadata: Metadata) -> Self {
    self.metadata = Some(metadata);
    self
}

pub fn with_service_tier(mut self, tier: ServiceTier) -> Self {
    self.service_tier = Some(tier);
    self
}

pub fn with_inference_geo(mut self, geo: impl Into<String>) -> Self {
    self.inference_geo = Some(geo.into());
    self
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test test_metadata_serialization test_service_tier_serialization -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/types.rs
git commit -m "feat: add metadata, service_tier, inference_geo request params"
```

---

### Task 3: Add `StopReason::Refusal` and `StopDetails`

**Files:**
- Modify: `src/types.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn test_refusal_stop_reason_deserialization() {
    let json = r#"{
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "content": [],
        "model": "claude-sonnet-4-5-20250929",
        "stop_reason": "refusal",
        "stop_details": {
            "type": "refusal",
            "category": "cyber",
            "explanation": "Request involves prohibited content"
        },
        "usage": { "input_tokens": 10, "output_tokens": 0 }
    }"#;

    let response: MessagesResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.stop_reason, Some(StopReason::Refusal));
    assert!(response.stop_details.is_some());
    let details = response.stop_details.unwrap();
    assert_eq!(details.category, Some(RefusalCategory::Cyber));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_refusal_stop_reason -- --nocapture`
Expected: FAIL

- [ ] **Step 3: Add Refusal variant and StopDetails**

In `src/types.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    PauseTurn,
    Refusal,
}

/// Details about why the model stopped (currently only for refusals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopDetails {
    #[serde(rename = "type")]
    pub stop_type: String,

    /// Refusal category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<RefusalCategory>,

    /// Human-readable explanation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

/// Category of content refusal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalCategory {
    Cyber,
    Bio,
    ReasoningExtraction,
}
```

Add `stop_details` to `MessagesResponse`:

```rust
pub struct MessagesResponse {
    // ... existing fields ...

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_details: Option<StopDetails>,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test test_refusal -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/types.rs
git commit -m "feat: add StopReason::Refusal with StopDetails and RefusalCategory"
```

---

### Task 4: Expand `EffortLevel` with `XHigh` and `Max`

**Files:**
- Modify: `src/types.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn test_effort_xhigh_and_max() {
    let json_xhigh = serde_json::to_string(&EffortLevel::XHigh).unwrap();
    assert_eq!(json_xhigh, r#""xhigh""#);

    let json_max = serde_json::to_string(&EffortLevel::Max).unwrap();
    assert_eq!(json_max, r#""max""#);

    // Round-trip
    let parsed: EffortLevel = serde_json::from_str(r#""xhigh""#).unwrap();
    assert_eq!(parsed, EffortLevel::XHigh);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_effort_xhigh -- --nocapture`
Expected: FAIL

- [ ] **Step 3: Add new variants**

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EffortLevel {
    High,
    Medium,
    Low,
    /// Extra-high effort
    #[serde(rename = "xhigh")]
    XHigh,
    /// Maximum effort
    Max,
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test test_effort -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/types.rs
git commit -m "feat: add EffortLevel::XHigh and EffortLevel::Max variants"
```

---

### Task 5: Update `CacheControl` for TTL Support

**Files:**
- Modify: `src/types.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn test_cache_control_with_ttl() {
    let cache = CacheControl::ephemeral_with_ttl(CacheTtl::OneHour);
    let json = serde_json::to_value(&cache).unwrap();
    assert_eq!(json["type"], "ephemeral");
    assert_eq!(json["ttl"], "1h");
}

#[test]
fn test_cache_control_ttl_deserialization() {
    let json = r#"{"type": "ephemeral", "ttl": "5m"}"#;
    let cache: CacheControl = serde_json::from_str(json).unwrap();
    assert_eq!(cache.ttl, Some(CacheTtl::FiveMinutes));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_cache_control_with_ttl -- --nocapture`
Expected: FAIL

- [ ] **Step 3: Add TTL support**

```rust
/// Cache TTL duration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheTtl {
    /// 5-minute TTL (default)
    #[serde(rename = "5m")]
    FiveMinutes,
    /// 1-hour TTL
    #[serde(rename = "1h")]
    OneHour,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub cache_type: CacheType,

    /// TTL for cached content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<CacheTtl>,
}

impl CacheControl {
    /// Create an ephemeral cache control (default 5m TTL)
    pub fn ephemeral() -> Self {
        Self {
            cache_type: CacheType::Ephemeral,
            ttl: None,
        }
    }

    /// Create an ephemeral cache control with explicit TTL
    pub fn ephemeral_with_ttl(ttl: CacheTtl) -> Self {
        Self {
            cache_type: CacheType::Ephemeral,
            ttl: Some(ttl),
        }
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test test_cache_control -- --nocapture`
Expected: PASS (both old and new tests)

- [ ] **Step 5: Commit**

```bash
git add src/types.rs
git commit -m "feat: add CacheTtl (5m, 1h) support to CacheControl"
```

---

### Task 6: Update `Usage` Struct with Detailed Token Accounting

**Files:**
- Modify: `src/types.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn test_usage_with_details_deserialization() {
    let json = r#"{
        "input_tokens": 100,
        "output_tokens": 50,
        "cache_creation_input_tokens": 10,
        "cache_read_input_tokens": 5,
        "output_tokens_details": {
            "thinking_tokens": 20
        },
        "server_tool_use": {
            "web_search_requests": 3
        },
        "service_tier": "priority",
        "inference_geo": "us"
    }"#;

    let usage: Usage = serde_json::from_str(json).unwrap();
    assert_eq!(usage.input_tokens, 100);
    let details = usage.output_tokens_details.unwrap();
    assert_eq!(details.thinking_tokens, Some(20));
    let server = usage.server_tool_use.unwrap();
    assert_eq!(server.web_search_requests, Some(3));
    assert_eq!(usage.service_tier, Some("priority".into()));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_usage_with_details -- --nocapture`
Expected: FAIL

- [ ] **Step 3: Expand Usage struct**

```rust
/// Detailed output token breakdown
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OutputTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_tokens: Option<u32>,
}

/// Server tool usage counts
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ServerToolUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_requests: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_fetch_requests: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,

    /// Detailed breakdown of output tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens_details: Option<OutputTokensDetails>,

    /// Server tool invocation counts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_tool_use: Option<ServerToolUsage>,

    /// Which service tier handled this request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,

    /// Which geographic region processed this request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inference_geo: Option<String>,
}
```

Note: `Usage` changes from `Copy` to `Clone` only (it now contains `Option<String>`). Update any `Copy` bounds. Remove the separate `ExtendedUsage` struct since thinking tokens are now in `Usage.output_tokens_details`.

- [ ] **Step 4: Fix compilation and run tests**

Run: `cargo test -- --nocapture`
Expected: All tests PASS. If `Copy` removal causes issues in streaming.rs, update the `Usage` field in `StreamEvent::MessageDelta` to use `Clone`.

- [ ] **Step 5: Commit**

```bash
git add src/types.rs src/streaming.rs
git commit -m "feat: expand Usage with output_tokens_details, server_tool_use, service_tier"
```

---

### Task 7: Add Forward-Compatible `ContentBlock` Handling

**Files:**
- Modify: `src/types.rs`
- Modify: `src/tokens.rs`

This is critical for preventing deserialization failures when the API returns new content block types that this SDK doesn't know about yet.

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn test_unknown_content_block_deserializes() {
    let json = r#"{"type": "some_future_type", "data": "whatever"}"#;
    let block: ContentBlock = serde_json::from_str(json).unwrap();
    match block {
        ContentBlock::Unknown { block_type, .. } => {
            assert_eq!(block_type, "some_future_type");
        }
        _ => panic!("Expected Unknown variant"),
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_unknown_content_block -- --nocapture`
Expected: FAIL — deserialization error

- [ ] **Step 3: Add Unknown variant with custom deserialization**

Since `ContentBlock` uses `#[serde(tag = "type")]`, we need a catch-all. The cleanest approach is to implement custom deserialization that tries known variants first, then falls back to `Unknown`:

Add to the `ContentBlock` enum:

```rust
/// Unknown content block type (forward compatibility)
///
/// When the API returns a content block type this SDK doesn't
/// recognize, it's captured here rather than causing a deserialization error.
Unknown {
    /// The `type` field value
    #[serde(rename = "type")]
    block_type: String,
    /// Raw JSON of the unknown block
    #[serde(flatten)]
    data: serde_json::Value,
},
```

Note: `#[serde(tag = "type")]` with an unknown variant requires using `#[serde(other)]` on a unit variant OR implementing custom `Deserialize`. Since `Unknown` has data fields, implement custom `Deserialize`:

Replace `#[derive(Deserialize)]` on `ContentBlock` with a manual `impl<'de> Deserialize<'de> for ContentBlock` that:
1. Deserializes as `serde_json::Value`
2. Reads the `"type"` field
3. Matches known types and delegates to the typed variant
4. Falls through to `Unknown` for unrecognized types

Keep `#[derive(Serialize)]` — only deserialization needs the custom impl.

Update `src/tokens.rs` `count_content_block` to handle:

```rust
ContentBlock::Unknown { .. } => 0, // Can't estimate unknown blocks
```

- [ ] **Step 4: Run all tests**

Run: `cargo test --all-features`
Expected: ALL PASS

- [ ] **Step 5: Commit**

```bash
git add src/types.rs src/tokens.rs
git commit -m "feat: add ContentBlock::Unknown for forward-compatible deserialization"
```

---

### Task 8: Add Rate Limit Header Parsing

**Files:**
- Modify: `src/client.rs`
- Modify: `src/types.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn test_rate_limit_info_default() {
    let info = RateLimitInfo::default();
    assert!(info.requests_remaining.is_none());
    assert!(info.tokens_remaining.is_none());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_rate_limit_info -- --nocapture`
Expected: FAIL

- [ ] **Step 3: Add RateLimitInfo struct and parse from response headers**

In `src/types.rs`:

```rust
/// Rate limit information from API response headers
#[derive(Debug, Clone, Default)]
pub struct RateLimitInfo {
    /// Remaining requests in current window
    pub requests_remaining: Option<u32>,
    /// Remaining tokens in current window
    pub tokens_remaining: Option<u32>,
    /// Time until request limit resets (ISO 8601)
    pub requests_reset: Option<String>,
    /// Time until token limit resets (ISO 8601)
    pub tokens_reset: Option<String>,
}
```

In `src/client.rs`, add a helper to extract rate limit info from response headers:

```rust
fn parse_rate_limit_headers(headers: &reqwest::header::HeaderMap) -> RateLimitInfo {
    RateLimitInfo {
        requests_remaining: headers
            .get("anthropic-ratelimit-requests-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok()),
        tokens_remaining: headers
            .get("anthropic-ratelimit-tokens-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok()),
        requests_reset: headers
            .get("anthropic-ratelimit-requests-reset")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        tokens_reset: headers
            .get("anthropic-ratelimit-tokens-reset")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
    }
}
```

Add `RateLimitInfo` to `MessagesResponse` as an optional field (skip serialization, populated only from HTTP responses):

```rust
/// Rate limit info from response headers (not serialized)
#[serde(skip)]
pub rate_limit_info: Option<RateLimitInfo>,
```

Populate it in `send_anthropic()` after successful deserialization.

- [ ] **Step 4: Run tests**

Run: `cargo test --all-features`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/types.rs src/client.rs
git commit -m "feat: parse rate limit headers into RateLimitInfo on responses"
```

---

### Task 9: Update Re-exports and Bump Version

**Files:**
- Modify: `src/lib.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Add new re-exports to lib.rs**

```rust
pub use types::{
    // existing exports...
    CacheTtl, Metadata, OutputTokensDetails, RateLimitInfo,
    RefusalCategory, ServerToolUsage, ServiceTier, StopDetails,
};
```

- [ ] **Step 2: Bump version in Cargo.toml**

Change `version = "1.0.1"` to `version = "1.1.0"`.

- [ ] **Step 3: Run full test suite and clippy**

Run: `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --all -- --check`
Expected: ALL PASS, zero warnings, formatted

- [ ] **Step 4: Commit**

```bash
git add src/lib.rs Cargo.toml
git commit -m "chore: bump version to 1.1.0, update re-exports"
```

---

## Chunk 2: v2.0 — Server Tools & Type Overhaul (Outline)

> Detailed step-by-step plan to be written when v1.1 is complete.

### Task 10: Restructure `Tool` as Tagged Enum

The current `Tool` struct becomes `ToolDefinition::Custom`. New variants:

```rust
pub enum ToolDefinition {
    Custom(CustomTool),                   // Current Tool struct
    WebSearch(WebSearchTool),             // web_search_20250305 / 20260209
    WebFetch(WebFetchTool),               // web_fetch_20250910+
    CodeExecution(CodeExecutionTool),     // code_execution_20250522+
    Bash(BashTool),                       // bash_20250124
    TextEditor(TextEditorTool),           // text_editor_20250124+
}
```

Each variant has its own config fields (allowed_domains, max_uses, etc.).

### Task 11: Add Server Tool Content Blocks

New `ContentBlock` variants:
- `ServerToolUse` — server tool invocation
- `WebSearchToolResult` — search results with URLs
- `WebFetchToolResult` — fetched page content
- `CodeExecutionToolResult` — stdout/stderr/return_code
- `BashCodeExecutionToolResult`
- `TextEditorCodeExecutionToolResult`

### Task 12: Structured Output via JSON Schema

Expand `OutputConfig` to include `format`:

```rust
pub struct OutputConfig {
    pub effort: Option<EffortLevel>,
    pub format: Option<OutputFormat>,
}

pub struct OutputFormat {
    pub format_type: String, // "json_schema"
    pub schema: serde_json::Value,
}
```

### Task 13: Adaptive Thinking + Display Parameter

Expand `ThinkingConfig`:

```rust
pub enum ThinkingConfig {
    Enabled { budget_tokens: u32, display: Option<ThinkingDisplay> },
    Disabled,
    Adaptive { display: Option<ThinkingDisplay> },
}

pub enum ThinkingDisplay {
    Summarized,
    Omitted,
}
```

### Task 14: Move `disable_parallel_tool_use` into `ToolChoice`

Restructure `ToolChoice` to absorb the field:

```rust
pub enum ToolChoice {
    Auto { disable_parallel_tool_use: Option<bool> },
    Any { disable_parallel_tool_use: Option<bool> },
    Tool { name: String, disable_parallel_tool_use: Option<bool> },
    None,
}
```

Remove `disable_parallel_tool_use` from `MessagesRequest`.

### Task 15: Token Counting API Endpoint

Add `count_tokens()` method to `ClaudeClient`:

```rust
pub async fn count_tokens(&self, request: MessagesRequest) -> Result<TokenCount> {
    // POST /v1/messages/count_tokens
}

pub struct TokenCount {
    pub input_tokens: u32,
    pub cache_creation_input_tokens: Option<u32>,
    pub cache_read_input_tokens: Option<u32>,
}
```

### Task 16: `ToolResult.content` Accepts String or Content Blocks

Change `ToolResult.content` from `Option<String>` to accept either a string or an array of content blocks (matching the API's union type).

### Task 17: Add `caller` Field to `ToolUse`

```rust
ToolUse {
    id: String,
    name: String,
    input: serde_json::Value,
    caller: Option<String>, // "direct", "code_execution_20250825", etc.
    cache_control: Option<CacheControl>,
},
```

---

## Chunk 3: v2.1 — Niche & Experimental (Outline)

> Detailed step-by-step plan to be written when v2.0 is complete.

### Task 18: Container Support

- Add `container` field to `MessagesRequest` (container ID string)
- Add `Container` response struct (`id`, `expires_at`)
- Add `ContainerUpload` content block

### Task 19: Mid-Conversation System Blocks

- Add `MidConversationSystem` content block type

### Task 20: Memory Tool

- Add `MemoryTool` variant to `ToolDefinition`

### Task 21: Tool Search Tools

- Add `ToolSearchBm25` and `ToolSearchRegex` variants to `ToolDefinition`
- Add `ToolSearchToolResult` content block

### Task 22: Tool Definition Enhancements

- Add `defer_loading: Option<bool>` to `CustomTool`
- Add `eager_input_streaming: Option<bool>` to `CustomTool`
- Add `strict: Option<bool>` to `CustomTool`
