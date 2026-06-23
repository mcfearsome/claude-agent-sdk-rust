//! Type definitions for Claude API requests and responses.
//!
//! This module contains all the core types used for communicating with the Claude API:
//!
//! - [`MessagesRequest`] - The main request type for sending messages
//! - [`MessagesResponse`] - Response from the Messages API
//! - [`Message`] - Conversation messages with user/assistant roles
//! - [`ContentBlock`] - Text, images, documents, tool calls, and more
//! - [`CustomTool`] - Custom client-side tool definitions
//! - [`ToolDefinition`] - Enum wrapping custom and server tools
//! - [`ToolChoice`] - Control how Claude uses tools
//!
//! # Example
//!
//! ```rust
//! use claude_sdk::types::{MessagesRequest, Message, CustomTool, ToolChoice};
//! use serde_json::json;
//!
//! // Create a basic request
//! let request = MessagesRequest::new(
//!     "claude-sonnet-4-5-20250929",
//!     1024,
//!     vec![Message::user("Hello!")],
//! )
//! .with_system("You are a helpful assistant.")
//! .with_temperature(0.7);
//! ```

use serde::{Deserialize, Serialize};

/// Role in a conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

/// Content block in a message
///
/// Known content block types are deserialized into their respective variants.
/// Unrecognized types (e.g., new API features) are captured in [`ContentBlock::Unknown`]
/// instead of causing a deserialization error.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text content
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
        /// Citations (appears in responses when using search_result blocks)
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<Vec<Citation>>,
    },
    /// Image content
    Image {
        source: ImageSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// Document content (PDFs, text files)
    ///
    /// Requires beta header: `anthropic-beta: files-api-2025-04-14`
    Document {
        source: DocumentSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<CitationConfig>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// Tool use request from the assistant
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
        /// Which system invoked this tool ("direct", "code_execution_20250825", etc.)
        #[serde(skip_serializing_if = "Option::is_none")]
        caller: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// Tool result from the user
    ToolResult {
        tool_use_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<ToolResultContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
    /// Thinking block from extended thinking
    ///
    /// Contains Claude's step-by-step reasoning process.
    /// Appears when extended thinking is enabled.
    Thinking {
        thinking: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Redacted thinking block
    ///
    /// Contains encrypted thinking that was flagged by safety systems.
    /// Must be passed back unmodified in multi-turn conversations.
    RedactedThinking { data: String },
    /// Search result for RAG with automatic citations
    ///
    /// Supported in: Opus 4.5, Opus 4.1, Opus 4, Sonnet 4.5, Sonnet 4, Haiku 3.5
    SearchResult {
        source: String,
        title: String,
        content: Vec<TextBlock>,
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<CitationConfig>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// Server-side tool invocation
    ServerToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// Web search tool result
    WebSearchToolResult {
        tool_use_id: String,
        content: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// Code execution tool result
    CodeExecutionToolResult {
        tool_use_id: String,
        content: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// Unknown content block type (forward compatibility)
    ///
    /// When the API returns a content block type this SDK doesn't
    /// recognize, it's captured here rather than causing a deserialization error.
    #[serde(untagged)]
    Unknown {
        /// The `type` field value
        block_type: String,
        /// Raw JSON of the unknown block
        data: serde_json::Value,
    },
}

/// Content of a tool result -- either plain text or structured content blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    /// Plain text result
    Text(String),
    /// Array of content blocks
    Blocks(Vec<ContentBlock>),
}

/// Private helper enum for deserialization of known ContentBlock variants.
///
/// Mirrors all known variants of [`ContentBlock`] exactly, with derived
/// `Deserialize`. Used by the custom `Deserialize` impl on `ContentBlock` to
/// avoid infinite recursion (since `ContentBlock` can't derive `Deserialize`
/// due to the `Unknown` catch-all variant needing to capture raw JSON data).
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentBlockHelper {
    Text {
        text: String,
        cache_control: Option<CacheControl>,
        citations: Option<Vec<Citation>>,
    },
    Image {
        source: ImageSource,
        cache_control: Option<CacheControl>,
    },
    Document {
        source: DocumentSource,
        title: Option<String>,
        context: Option<String>,
        citations: Option<CitationConfig>,
        cache_control: Option<CacheControl>,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
        caller: Option<String>,
        cache_control: Option<CacheControl>,
    },
    ToolResult {
        tool_use_id: String,
        content: Option<ToolResultContent>,
        is_error: Option<bool>,
    },
    Thinking {
        thinking: String,
        signature: Option<String>,
    },
    RedactedThinking {
        data: String,
    },
    SearchResult {
        source: String,
        title: String,
        content: Vec<TextBlock>,
        citations: Option<CitationConfig>,
        cache_control: Option<CacheControl>,
    },
    ServerToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
        cache_control: Option<CacheControl>,
    },
    WebSearchToolResult {
        tool_use_id: String,
        content: serde_json::Value,
        cache_control: Option<CacheControl>,
    },
    CodeExecutionToolResult {
        tool_use_id: String,
        content: serde_json::Value,
        cache_control: Option<CacheControl>,
    },
}

impl<'de> serde::Deserialize<'de> for ContentBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        // Try to deserialize as a known variant via the helper enum
        match serde_json::from_value::<ContentBlockHelper>(value.clone()) {
            Ok(helper) => Ok(match helper {
                ContentBlockHelper::Text {
                    text,
                    cache_control,
                    citations,
                } => ContentBlock::Text {
                    text,
                    cache_control,
                    citations,
                },
                ContentBlockHelper::Image {
                    source,
                    cache_control,
                } => ContentBlock::Image {
                    source,
                    cache_control,
                },
                ContentBlockHelper::Document {
                    source,
                    title,
                    context,
                    citations,
                    cache_control,
                } => ContentBlock::Document {
                    source,
                    title,
                    context,
                    citations,
                    cache_control,
                },
                ContentBlockHelper::ToolUse {
                    id,
                    name,
                    input,
                    caller,
                    cache_control,
                } => ContentBlock::ToolUse {
                    id,
                    name,
                    input,
                    caller,
                    cache_control,
                },
                ContentBlockHelper::ToolResult {
                    tool_use_id,
                    content,
                    is_error,
                } => ContentBlock::ToolResult {
                    tool_use_id,
                    content,
                    is_error,
                },
                ContentBlockHelper::Thinking {
                    thinking,
                    signature,
                } => ContentBlock::Thinking {
                    thinking,
                    signature,
                },
                ContentBlockHelper::RedactedThinking { data } => {
                    ContentBlock::RedactedThinking { data }
                }
                ContentBlockHelper::SearchResult {
                    source,
                    title,
                    content,
                    citations,
                    cache_control,
                } => ContentBlock::SearchResult {
                    source,
                    title,
                    content,
                    citations,
                    cache_control,
                },
                ContentBlockHelper::ServerToolUse {
                    id,
                    name,
                    input,
                    cache_control,
                } => ContentBlock::ServerToolUse {
                    id,
                    name,
                    input,
                    cache_control,
                },
                ContentBlockHelper::WebSearchToolResult {
                    tool_use_id,
                    content,
                    cache_control,
                } => ContentBlock::WebSearchToolResult {
                    tool_use_id,
                    content,
                    cache_control,
                },
                ContentBlockHelper::CodeExecutionToolResult {
                    tool_use_id,
                    content,
                    cache_control,
                } => ContentBlock::CodeExecutionToolResult {
                    tool_use_id,
                    content,
                    cache_control,
                },
            }),
            Err(_) => {
                // Unknown type -- extract the type field and capture the raw data
                let block_type = value
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                Ok(ContentBlock::Unknown {
                    block_type,
                    data: value,
                })
            }
        }
    }
}

/// Text block for search result content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    #[serde(rename = "type")]
    pub block_type: String, // Always "text"
    pub text: String,
}

/// Image source for vision
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    /// Base64-encoded image
    Base64 { media_type: String, data: String },
    /// Image URL
    Url { url: String },
    /// File ID from Files API
    ///
    /// Requires beta header: `anthropic-beta: files-api-2025-04-14`
    File { file_id: String },
}

/// Document source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocumentSource {
    /// File ID from Files API
    File { file_id: String },
    /// Inline text document
    Text { media_type: String, data: String },
}

/// Citation configuration for documents and search results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CitationConfig {
    pub enabled: bool,
}

/// Citation location in a response
///
/// Claude automatically includes these when using search_result blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    #[serde(rename = "type")]
    pub citation_type: String, // "search_result_location"

    pub source: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    pub cited_text: String,

    pub search_result_index: usize,

    pub start_block_index: usize,

    pub end_block_index: usize,
}

/// Cache control for prompt caching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub cache_type: CacheType,

    /// TTL for cached content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<CacheTtl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CacheType {
    Ephemeral,
}

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

impl CacheControl {
    /// Create an ephemeral cache control
    pub fn ephemeral() -> Self {
        Self {
            cache_type: CacheType::Ephemeral,
            ttl: None,
        }
    }

    /// Create an ephemeral cache control with a specific TTL
    pub fn ephemeral_with_ttl(ttl: CacheTtl) -> Self {
        Self {
            cache_type: CacheType::Ephemeral,
            ttl: Some(ttl),
        }
    }
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

impl Message {
    /// Create a user message with text content
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentBlock::Text {
                text: text.into(),
                cache_control: None,
                citations: None,
            }],
        }
    }

    /// Create an assistant message with text content
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentBlock::Text {
                text: text.into(),
                cache_control: None,
                citations: None,
            }],
        }
    }

    /// Create a user message with a tool result
    pub fn tool_result(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentBlock::ToolResult {
                tool_use_id: tool_use_id.into(),
                content: Some(ToolResultContent::Text(content.into())),
                is_error: None,
            }],
        }
    }
}

/// System prompt format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    String(String),
    Blocks(Vec<SystemBlock>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Custom client-side tool definition
///
/// Defines a tool with a name, description, and JSON Schema for its inputs.
/// Claude can call this tool and the client handles execution.
///
/// # Example
///
/// ```rust
/// use claude_sdk::CustomTool;
/// use serde_json::json;
///
/// let tool = CustomTool {
///     name: "get_weather".into(),
///     description: "Get weather for a location".into(),
///     input_schema: json!({
///         "type": "object",
///         "properties": {
///             "location": { "type": "string" }
///         },
///         "required": ["location"]
///     }),
///     disable_user_input: Some(true),
///     input_examples: None,
///     cache_control: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,

    /// If true, Claude will use this tool programmatically without asking the user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_user_input: Option<bool>,

    /// Example inputs for the tool (beta feature)
    ///
    /// Requires beta header: `anthropic-beta: advanced-tool-use-2025-11-20`
    /// Each example must be valid according to the input_schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_examples: Option<Vec<serde_json::Value>>,

    /// Cache control for this tool definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

/// Renamed to [`CustomTool`] in v2.0. Use `CustomTool` directly.
#[deprecated(
    since = "2.0.0",
    note = "Renamed to CustomTool. Use CustomTool directly."
)]
pub type Tool = CustomTool;

/// Tool definition -- either a custom client tool or a built-in server tool.
///
/// Use this type in [`MessagesRequest::tools`].
///
/// # Variants
///
/// - [`ToolDefinition::Custom`] - A client-side tool with name, description, and JSON schema
/// - [`ToolDefinition::Server`] - Any server-managed tool (web search, code execution, etc.)
///
/// # Example
///
/// ```rust
/// use claude_sdk::{CustomTool, ToolDefinition};
/// use serde_json::json;
///
/// // Custom tool
/// let custom = ToolDefinition::Custom(CustomTool {
///     name: "my_tool".into(),
///     description: "A custom tool".into(),
///     input_schema: json!({"type": "object"}),
///     disable_user_input: None,
///     input_examples: None,
///     cache_control: None,
/// });
///
/// // Server tool (raw JSON)
/// let server = ToolDefinition::Server(json!({
///     "type": "web_search_20250305",
///     "name": "web_search",
///     "max_uses": 5
/// }));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolDefinition {
    /// Custom client-side tool with name, description, and JSON schema
    Custom(CustomTool),
    /// Any server tool -- uses raw JSON since server tool types vary
    Server(serde_json::Value),
}

impl From<CustomTool> for ToolDefinition {
    fn from(tool: CustomTool) -> Self {
        ToolDefinition::Custom(tool)
    }
}

/// Tool choice configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolChoice {
    /// Let Claude decide whether to use tools (default)
    Auto {
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    /// Claude must use one of the provided tools
    Any {
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    /// Force Claude to use a specific tool
    Tool {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        disable_parallel_tool_use: Option<bool>,
    },
    /// Prevent Claude from using any tools
    None,
}

impl ToolChoice {
    /// Create Auto variant
    pub fn auto() -> Self {
        Self::Auto {
            disable_parallel_tool_use: None,
        }
    }

    /// Create Any variant
    pub fn any() -> Self {
        Self::Any {
            disable_parallel_tool_use: None,
        }
    }

    /// Create Tool variant with specific tool name
    pub fn tool(name: impl Into<String>) -> Self {
        Self::Tool {
            name: name.into(),
            disable_parallel_tool_use: None,
        }
    }

    /// Create None variant
    pub fn none() -> Self {
        Self::None
    }
}

/// Detailed output token breakdown
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OutputTokensDetails {
    /// Tokens used for thinking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_tokens: Option<u32>,
}

/// Server tool usage counts
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ServerToolUsage {
    /// Number of web search requests made
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_requests: Option<u32>,
    /// Number of web fetch requests made
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_fetch_requests: Option<u32>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,

    /// Tokens written to cache (prompt caching)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,

    /// Tokens read from cache (prompt caching)
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

/// Extended usage information for responses with thinking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedUsage {
    #[serde(flatten)]
    pub base: Usage,

    /// Tokens used for thinking (extended thinking)
    ///
    /// Note: With summarized thinking (Claude 4+), you're billed for the full
    /// thinking tokens, not the summarized tokens you see in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_tokens: Option<u32>,
}

/// Request to create a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesRequest {
    /// Model identifier (e.g., "claude-3-5-sonnet-20241022")
    pub model: String,

    /// Maximum tokens to generate
    pub max_tokens: u32,

    /// Conversation messages
    pub messages: Vec<Message>,

    /// System prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,

    /// Available tools (custom client tools and/or server tools)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,

    /// Tool choice configuration
    ///
    /// Controls how Claude uses tools:
    /// - `Auto` (default): Claude decides whether to use tools
    /// - `Any`: Claude must use one of the provided tools
    /// - `Tool { name }`: Force Claude to use a specific tool
    /// - `None`: Prevent Claude from using any tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Sampling temperature (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Top-p sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Top-k sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Output configuration (beta)
    ///
    /// Controls output behavior like effort level.
    /// Requires beta header for effort: `anthropic-beta: effort-2025-11-24`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_config: Option<OutputConfig>,

    /// Extended thinking configuration
    ///
    /// Enables Claude's step-by-step reasoning process.
    /// Supported models: Sonnet 4.5, Haiku 4.5, Opus 4.5, and more.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,

    /// Request metadata for abuse detection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,

    /// Service tier for request routing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,

    /// Geographic inference routing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inference_geo: Option<String>,
}

/// Extended thinking configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ThinkingConfig {
    /// Enable extended thinking with token budget
    Enabled {
        /// Maximum tokens Claude can use for thinking
        ///
        /// Minimum: 1024 tokens
        /// Can exceed max_tokens with interleaved thinking (beta: interleaved-thinking-2025-05-14)
        budget_tokens: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        display: Option<ThinkingDisplay>,
    },
    /// Disable extended thinking
    Disabled,
    /// Adaptive thinking -- let the model decide how much to think
    Adaptive {
        #[serde(skip_serializing_if = "Option::is_none")]
        display: Option<ThinkingDisplay>,
    },
}

/// How to display thinking blocks in responses
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingDisplay {
    /// Show summarized thinking
    Summarized,
    /// Omit thinking from response
    Omitted,
}

/// Output configuration for controlling response behavior
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputConfig {
    /// Effort level: controls token spending vs. response quality
    ///
    /// - `high` (default): Maximum capability, uses as many tokens as needed
    /// - `medium`: Balanced approach with moderate token savings
    /// - `low`: Most efficient, significant token savings
    ///
    /// Requires beta header: `anthropic-beta: effort-2025-11-24`
    /// Only supported by Claude Opus 4.5
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<EffortLevel>,

    /// Output format specification for structured outputs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<OutputFormat>,
}

/// Output format specification for structured outputs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputFormat {
    #[serde(rename = "type")]
    pub format_type: String,
    pub schema: serde_json::Value,
}

/// Effort level for response generation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EffortLevel {
    /// Maximum capability (default if omitted)
    High,
    /// Balanced token savings
    Medium,
    /// Maximum token efficiency
    Low,
    /// Extra-high effort
    #[serde(rename = "xhigh")]
    XHigh,
    /// Maximum effort
    Max,
}

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
    Auto,
    StandardOnly,
}

impl MessagesRequest {
    /// Create a new message request with required fields
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::{MessagesRequest, Message};
    ///
    /// let request = MessagesRequest::new(
    ///     "claude-3-5-sonnet-20241022",
    ///     1024,
    ///     vec![Message::user("Hello!")]
    /// );
    /// ```
    pub fn new(model: impl Into<String>, max_tokens: u32, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            max_tokens,
            messages,
            system: None,
            tools: None,
            tool_choice: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: None,
            output_config: None,
            thinking: None,
            metadata: None,
            service_tier: None,
            inference_geo: None,
        }
    }

    /// Set the system prompt.
    ///
    /// The system prompt provides instructions and context that guide Claude's behavior.
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::{MessagesRequest, Message};
    ///
    /// let request = MessagesRequest::new(
    ///     "claude-sonnet-4-5-20250929",
    ///     1024,
    ///     vec![Message::user("What's 2+2?")],
    /// )
    /// .with_system("You are a math tutor. Always explain your reasoning step by step.");
    /// ```
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(SystemPrompt::String(system.into()));
        self
    }

    /// Set the available tools for this request.
    ///
    /// Accepts any mix of custom and server tools via [`ToolDefinition`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::{MessagesRequest, Message, CustomTool, ToolDefinition};
    /// use serde_json::json;
    ///
    /// let calculator = ToolDefinition::Custom(CustomTool {
    ///     name: "calculator".into(),
    ///     description: "Perform basic arithmetic operations".into(),
    ///     input_schema: json!({
    ///         "type": "object",
    ///         "properties": {
    ///             "operation": { "type": "string", "enum": ["add", "subtract", "multiply", "divide"] },
    ///             "a": { "type": "number" },
    ///             "b": { "type": "number" }
    ///         },
    ///         "required": ["operation", "a", "b"]
    ///     }),
    ///     disable_user_input: Some(true),
    ///     input_examples: None,
    ///     cache_control: None,
    /// });
    ///
    /// let request = MessagesRequest::new(
    ///     "claude-sonnet-4-5-20250929",
    ///     1024,
    ///     vec![Message::user("What's 15 * 7?")],
    /// )
    /// .with_tools(vec![calculator]);
    /// ```
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set tools using only custom (client-side) tools.
    ///
    /// Convenience method that wraps each [`CustomTool`] in [`ToolDefinition::Custom`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::{MessagesRequest, Message, CustomTool};
    /// use serde_json::json;
    ///
    /// let tool = CustomTool {
    ///     name: "my_tool".into(),
    ///     description: "A tool".into(),
    ///     input_schema: json!({"type": "object"}),
    ///     disable_user_input: None,
    ///     input_examples: None,
    ///     cache_control: None,
    /// };
    ///
    /// let request = MessagesRequest::new(
    ///     "claude-sonnet-4-5-20250929",
    ///     1024,
    ///     vec![Message::user("Hello")],
    /// )
    /// .with_custom_tools(vec![tool]);
    /// ```
    pub fn with_custom_tools(mut self, tools: Vec<CustomTool>) -> Self {
        self.tools = Some(tools.into_iter().map(ToolDefinition::Custom).collect());
        self
    }

    /// Set tool choice configuration.
    ///
    /// Controls how Claude decides whether and which tools to use.
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::{MessagesRequest, Message, ToolChoice};
    ///
    /// // Force Claude to use a specific tool
    /// let request = MessagesRequest::new(
    ///     "claude-sonnet-4-5-20250929",
    ///     1024,
    ///     vec![Message::user("Search for weather")],
    /// )
    /// .with_tool_choice(ToolChoice::tool("get_weather"));
    ///
    /// // Or let Claude decide (default)
    /// let request2 = MessagesRequest::new(
    ///     "claude-sonnet-4-5-20250929",
    ///     1024,
    ///     vec![Message::user("Hello")],
    /// )
    /// .with_tool_choice(ToolChoice::auto());
    /// ```
    pub fn with_tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    /// Set the sampling temperature.
    ///
    /// Temperature controls randomness in the output:
    /// - `0.0` - Deterministic, most likely tokens
    /// - `0.5` - Balanced creativity
    /// - `1.0` - Maximum randomness
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::{MessagesRequest, Message};
    ///
    /// // Low temperature for factual responses
    /// let factual = MessagesRequest::new(
    ///     "claude-sonnet-4-5-20250929",
    ///     1024,
    ///     vec![Message::user("What is the capital of France?")],
    /// )
    /// .with_temperature(0.0);
    ///
    /// // Higher temperature for creative writing
    /// let creative = MessagesRequest::new(
    ///     "claude-sonnet-4-5-20250929",
    ///     1024,
    ///     vec![Message::user("Write a short poem about the ocean.")],
    /// )
    /// .with_temperature(0.8);
    /// ```
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set effort level (beta - requires `anthropic-beta: effort-2025-11-24` header).
    ///
    /// Controls the trade-off between response quality and token usage.
    /// Only supported by Claude Opus 4.5.
    ///
    /// # Effort Levels
    ///
    /// - [`EffortLevel::High`] - Maximum capability (default)
    /// - [`EffortLevel::Medium`] - Balanced token savings
    /// - [`EffortLevel::Low`] - Maximum efficiency
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::{MessagesRequest, Message, EffortLevel};
    ///
    /// let request = MessagesRequest::new(
    ///     "claude-opus-4-5-20251101",  // Opus 4.5 only
    ///     1024,
    ///     vec![Message::user("Summarize this document briefly.")],
    /// )
    /// .with_effort(EffortLevel::Low);  // Optimize for efficiency
    /// ```
    pub fn with_effort(mut self, effort: EffortLevel) -> Self {
        let config = self.output_config.get_or_insert(OutputConfig {
            effort: None,
            format: None,
        });
        config.effort = Some(effort);
        self
    }

    /// Set JSON schema for structured output
    pub fn with_json_schema(mut self, schema: serde_json::Value) -> Self {
        let config = self.output_config.get_or_insert(OutputConfig {
            effort: None,
            format: None,
        });
        config.format = Some(OutputFormat {
            format_type: "json_schema".into(),
            schema,
        });
        self
    }

    /// Enable extended thinking with a token budget.
    ///
    /// Extended thinking allows Claude to reason through complex problems
    /// step-by-step before providing a final answer.
    ///
    /// # Requirements
    ///
    /// - Supported by: Claude Sonnet 4.5, Haiku 4.5, Opus 4.5, and other Claude 4+ models
    /// - Minimum budget: 1024 tokens
    /// - The thinking process appears in [`ContentBlock::Thinking`] blocks
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::{MessagesRequest, Message};
    ///
    /// let request = MessagesRequest::new(
    ///     "claude-sonnet-4-5-20250929",
    ///     8192,
    ///     vec![Message::user("Solve this step by step: If a train travels...")],
    /// )
    /// .with_thinking(4096);  // Allow up to 4096 tokens for reasoning
    /// ```
    pub fn with_thinking(mut self, budget_tokens: u32) -> Self {
        self.thinking = Some(ThinkingConfig::Enabled {
            budget_tokens,
            display: None,
        });
        self
    }

    /// Enable adaptive thinking -- let the model decide how much to think
    pub fn with_adaptive_thinking(mut self) -> Self {
        self.thinking = Some(ThinkingConfig::Adaptive { display: None });
        self
    }

    /// Set request metadata for abuse detection.
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set the service tier for request routing.
    pub fn with_service_tier(mut self, tier: ServiceTier) -> Self {
        self.service_tier = Some(tier);
        self
    }

    /// Set the geographic inference routing.
    pub fn with_inference_geo(mut self, geo: impl Into<String>) -> Self {
        self.inference_geo = Some(geo.into());
        self
    }
}

/// Response from the token counting endpoint
///
/// Use `ClaudeClient::count_tokens()` to get server-side token counts
/// before sending a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCount {
    /// Number of input tokens the request would use
    pub input_tokens: u32,
    /// Tokens that would be written to cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    /// Tokens that would be read from cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
}

/// Stop reason for a message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Natural end of message
    EndTurn,
    /// Hit max tokens
    MaxTokens,
    /// Hit a stop sequence
    StopSequence,
    /// Model wants to use a tool
    ToolUse,
    /// Long-running server tool paused the turn
    ///
    /// Continue by sending the response content back in the next request.
    /// Used with server tools like web search.
    PauseTurn,
    /// Model refused the request
    Refusal,
}

/// Details about why the model stopped (currently only for refusals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopDetails {
    #[serde(rename = "type")]
    pub stop_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<RefusalCategory>,

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

/// Response from creating a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesResponse {
    pub id: String,

    #[serde(rename = "type")]
    pub response_type: String,

    pub role: Role,

    pub content: Vec<ContentBlock>,

    pub model: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_details: Option<StopDetails>,

    pub usage: Usage,

    /// Rate limit info from response headers (not part of JSON body)
    #[serde(skip)]
    pub rate_limit_info: Option<RateLimitInfo>,
}

/// Rate limit information from API response headers
///
/// The API returns these headers with every response to help
/// clients manage their request rate.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_control_ephemeral() {
        let cache = CacheControl::ephemeral();
        assert_eq!(cache.cache_type, CacheType::Ephemeral);
    }

    #[test]
    fn test_cache_control_serialization() {
        let cache = CacheControl::ephemeral();
        let json = serde_json::to_string(&cache).unwrap();
        assert_eq!(json, r#"{"type":"ephemeral"}"#);
    }

    #[test]
    fn test_text_content_with_cache() {
        let content = ContentBlock::Text {
            text: "test".into(),
            cache_control: Some(CacheControl::ephemeral()),
            citations: None,
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "test");
        assert_eq!(json["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn test_system_block_with_cache() {
        let block = SystemBlock {
            block_type: "text".into(),
            text: "You are helpful".into(),
            cache_control: Some(CacheControl::ephemeral()),
        };

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn test_tool_with_cache() {
        let tool = CustomTool {
            name: "test".into(),
            description: "test tool".into(),
            input_schema: serde_json::json!({"type": "object"}),
            disable_user_input: Some(true),
            input_examples: None,
            cache_control: Some(CacheControl::ephemeral()),
        };

        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["name"], "test");
        assert_eq!(json["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn test_message_constructors() {
        let user_msg = Message::user("hello");
        assert_eq!(user_msg.role, Role::User);
        assert_eq!(user_msg.content.len(), 1);

        let assistant_msg = Message::assistant("hi");
        assert_eq!(assistant_msg.role, Role::Assistant);

        let tool_result = Message::tool_result("id_123", "result");
        assert_eq!(tool_result.role, Role::User);
        match &tool_result.content[0] {
            ContentBlock::ToolResult { tool_use_id, .. } => {
                assert_eq!(tool_use_id, "id_123");
            }
            _ => panic!("Expected ToolResult"),
        }
    }

    #[test]
    fn test_messages_request_builder() {
        let request = MessagesRequest::new(
            "claude-sonnet-4-5-20250929",
            1024,
            vec![Message::user("test")],
        )
        .with_system("System prompt")
        .with_temperature(0.7);

        assert_eq!(request.model, "claude-sonnet-4-5-20250929");
        assert_eq!(request.max_tokens, 1024);
        assert_eq!(request.messages.len(), 1);
        assert!(request.system.is_some());
        assert_eq!(request.temperature, Some(0.7));
    }

    #[test]
    fn test_metadata_serialization() {
        let request = MessagesRequest::new(
            "claude-sonnet-4-5-20250929",
            1024,
            vec![Message::user("Hello")],
        )
        .with_metadata(Metadata {
            user_id: Some("user-abc-123".into()),
        });

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["metadata"]["user_id"], "user-abc-123");
    }

    #[test]
    fn test_service_tier_serialization() {
        let json = serde_json::to_string(&ServiceTier::StandardOnly).unwrap();
        assert_eq!(json, r#""standard_only""#);

        let json = serde_json::to_string(&ServiceTier::Auto).unwrap();
        assert_eq!(json, r#""auto""#);
    }

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
        let details = response.stop_details.as_ref().unwrap();
        assert_eq!(details.category, Some(RefusalCategory::Cyber));
        assert_eq!(
            details.explanation.as_deref(),
            Some("Request involves prohibited content")
        );
    }

    #[test]
    fn test_effort_xhigh_and_max() {
        let json_xhigh = serde_json::to_string(&EffortLevel::XHigh).unwrap();
        assert_eq!(json_xhigh, r#""xhigh""#);

        let json_max = serde_json::to_string(&EffortLevel::Max).unwrap();
        assert_eq!(json_max, r#""max""#);

        // Round-trip
        let parsed: EffortLevel = serde_json::from_str(r#""xhigh""#).unwrap();
        assert_eq!(parsed, EffortLevel::XHigh);

        let parsed: EffortLevel = serde_json::from_str(r#""max""#).unwrap();
        assert_eq!(parsed, EffortLevel::Max);
    }

    // Task 5: CacheTtl tests

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

    #[test]
    fn test_cache_control_without_ttl_still_works() {
        // Existing behavior: no ttl field
        let cache = CacheControl::ephemeral();
        let json = serde_json::to_string(&cache).unwrap();
        assert_eq!(json, r#"{"type":"ephemeral"}"#);
        assert_eq!(cache.ttl, None);
    }

    // Task 6: Usage expansion tests

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
        assert_eq!(usage.output_tokens, 50);
        let details = usage.output_tokens_details.unwrap();
        assert_eq!(details.thinking_tokens, Some(20));
        let server = usage.server_tool_use.unwrap();
        assert_eq!(server.web_search_requests, Some(3));
        assert_eq!(usage.service_tier.as_deref(), Some("priority"));
    }

    #[test]
    fn test_usage_without_new_fields_still_works() {
        let json = r#"{"input_tokens": 10, "output_tokens": 5}"#;
        let usage: Usage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.input_tokens, 10);
        assert!(usage.output_tokens_details.is_none());
        assert!(usage.server_tool_use.is_none());
    }

    // Task 6: TokenCount tests

    #[test]
    fn test_token_count_deserialization() {
        let json = r#"{
            "input_tokens": 1234,
            "cache_creation_input_tokens": 100,
            "cache_read_input_tokens": 50
        }"#;
        let count: TokenCount = serde_json::from_str(json).unwrap();
        assert_eq!(count.input_tokens, 1234);
        assert_eq!(count.cache_creation_input_tokens, Some(100));
        assert_eq!(count.cache_read_input_tokens, Some(50));
    }

    #[test]
    fn test_token_count_minimal() {
        let json = r#"{"input_tokens": 42}"#;
        let count: TokenCount = serde_json::from_str(json).unwrap();
        assert_eq!(count.input_tokens, 42);
        assert!(count.cache_creation_input_tokens.is_none());
    }

    // Task 8: RateLimitInfo tests

    #[test]
    fn test_rate_limit_info_default() {
        let info = RateLimitInfo::default();
        assert!(info.requests_remaining.is_none());
        assert!(info.tokens_remaining.is_none());
        assert!(info.requests_reset.is_none());
        assert!(info.tokens_reset.is_none());
    }

    #[test]
    fn test_response_deserialization_with_skipped_rate_limit() {
        let json = r#"{
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [{"type": "text", "text": "hello"}],
            "model": "claude-sonnet-4-5-20250929",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;

        let response: MessagesResponse = serde_json::from_str(json).unwrap();
        assert!(response.rate_limit_info.is_none()); // Skipped in JSON
    }

    // Task 7: ContentBlock::Unknown forward-compatible deserialization tests

    #[test]
    fn test_unknown_content_block_deserializes() {
        let json = r#"{"type": "some_future_block", "id": "fb_123", "data": "test"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::Unknown { block_type, data } => {
                assert_eq!(block_type, "some_future_block");
                assert_eq!(data["id"], "fb_123");
            }
            _ => panic!("Expected Unknown variant"),
        }
    }

    #[test]
    fn test_known_content_blocks_still_work() {
        let json = r#"{"type": "text", "text": "hello"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::Text { text, .. } => assert_eq!(text, "hello"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_content_block_roundtrip() {
        let original = ContentBlock::Text {
            text: "test".into(),
            cache_control: None,
            citations: None,
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        match deserialized {
            ContentBlock::Text { text, .. } => assert_eq!(text, "test"),
            _ => panic!("Expected Text variant after roundtrip"),
        }
    }

    #[test]
    fn test_unknown_block_in_response() {
        let json = r#"{
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [
                {"type": "text", "text": "hello"},
                {"type": "some_future_block", "id": "fb_1", "data": "test"}
            ],
            "model": "claude-sonnet-4-5-20250929",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;

        let response: MessagesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.content.len(), 2);
        match &response.content[0] {
            ContentBlock::Text { text, .. } => assert_eq!(text, "hello"),
            _ => panic!("Expected Text variant"),
        }
        match &response.content[1] {
            ContentBlock::Unknown { block_type, .. } => {
                assert_eq!(block_type, "some_future_block");
            }
            _ => panic!("Expected Unknown variant"),
        }
    }

    #[test]
    fn test_unknown_block_without_type_field() {
        // Edge case: a JSON object with no "type" field should fail for the helper
        // and fall through to Unknown with "unknown" as block_type
        let json = r#"{"foo": "bar"}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::Unknown { block_type, data } => {
                assert_eq!(block_type, "unknown");
                assert_eq!(data["foo"], "bar");
            }
            _ => panic!("Expected Unknown variant"),
        }
    }

    // Task 3: Server tool content block tests

    #[test]
    fn test_server_tool_use_deserialization() {
        let json = r#"{"type": "server_tool_use", "id": "stu_123", "name": "web_search", "input": {"query": "rust"}}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::ServerToolUse {
                id, name, input, ..
            } => {
                assert_eq!(id, "stu_123");
                assert_eq!(name, "web_search");
                assert_eq!(input["query"], "rust");
            }
            _ => panic!("Expected ServerToolUse variant"),
        }
    }

    #[test]
    fn test_web_search_tool_result_deserialization() {
        let json = r#"{"type": "web_search_tool_result", "tool_use_id": "stu_123", "content": [{"type": "web_page", "url": "https://example.com"}]}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::WebSearchToolResult {
                tool_use_id,
                content,
                ..
            } => {
                assert_eq!(tool_use_id, "stu_123");
                assert!(content.is_array());
            }
            _ => panic!("Expected WebSearchToolResult variant"),
        }
    }

    #[test]
    fn test_code_execution_tool_result_deserialization() {
        let json = r#"{"type": "code_execution_tool_result", "tool_use_id": "ce_123", "content": {"stdout": "hello"}}"#;
        let block: ContentBlock = serde_json::from_str(json).unwrap();
        match block {
            ContentBlock::CodeExecutionToolResult {
                tool_use_id,
                content,
                ..
            } => {
                assert_eq!(tool_use_id, "ce_123");
                assert_eq!(content["stdout"], "hello");
            }
            _ => panic!("Expected CodeExecutionToolResult variant"),
        }
    }

    // Task 4: Structured output + adaptive thinking tests

    #[test]
    fn test_structured_output_serialization() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            }
        });
        let request = MessagesRequest::new(
            "claude-sonnet-4-5-20250929",
            1024,
            vec![Message::user("test")],
        )
        .with_json_schema(schema.clone());

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["output_config"]["format"]["type"], "json_schema");
        assert_eq!(json["output_config"]["format"]["schema"], schema);
    }

    #[test]
    fn test_effort_and_schema_coexist() {
        let request = MessagesRequest::new(
            "claude-sonnet-4-5-20250929",
            1024,
            vec![Message::user("test")],
        )
        .with_effort(EffortLevel::Low)
        .with_json_schema(serde_json::json!({"type": "object"}));

        let config = request.output_config.as_ref().unwrap();
        assert_eq!(config.effort, Some(EffortLevel::Low));
        assert!(config.format.is_some());
    }

    #[test]
    fn test_adaptive_thinking_serialization() {
        let request = MessagesRequest::new(
            "claude-sonnet-4-5-20250929",
            1024,
            vec![Message::user("test")],
        )
        .with_adaptive_thinking();

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["thinking"]["type"], "adaptive");
    }

    #[test]
    fn test_thinking_config_enabled_with_display() {
        let config = ThinkingConfig::Enabled {
            budget_tokens: 4096,
            display: Some(ThinkingDisplay::Summarized),
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["type"], "enabled");
        assert_eq!(json["budget_tokens"], 4096);
        assert_eq!(json["display"], "summarized");
    }

    // Task 5: ToolChoice, ToolResultContent, ToolUse caller tests

    #[test]
    fn test_tool_choice_auto_serialization() {
        let choice = ToolChoice::auto();
        let json = serde_json::to_value(&choice).unwrap();
        assert_eq!(json["type"], "auto");
        // disable_parallel_tool_use should be omitted when None
        assert!(json.get("disable_parallel_tool_use").is_none());
    }

    #[test]
    fn test_tool_choice_with_disable_parallel() {
        let choice = ToolChoice::Auto {
            disable_parallel_tool_use: Some(true),
        };
        let json = serde_json::to_value(&choice).unwrap();
        assert_eq!(json["type"], "auto");
        assert_eq!(json["disable_parallel_tool_use"], true);
    }

    #[test]
    fn test_tool_choice_tool_serialization() {
        let choice = ToolChoice::tool("my_tool");
        let json = serde_json::to_value(&choice).unwrap();
        assert_eq!(json["type"], "tool");
        assert_eq!(json["name"], "my_tool");
        assert!(json.get("disable_parallel_tool_use").is_none());
    }

    #[test]
    fn test_tool_result_content_text_serialization() {
        let content = ToolResultContent::Text("hello".into());
        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json, serde_json::json!("hello"));
    }

    #[test]
    fn test_tool_result_content_blocks_serialization() {
        let content = ToolResultContent::Blocks(vec![ContentBlock::Text {
            text: "result".into(),
            cache_control: None,
            citations: None,
        }]);
        let json = serde_json::to_value(&content).unwrap();
        assert!(json.is_array());
        assert_eq!(json[0]["type"], "text");
        assert_eq!(json[0]["text"], "result");
    }

    #[test]
    fn test_tool_use_with_caller() {
        let block = ContentBlock::ToolUse {
            id: "tu_123".into(),
            name: "my_tool".into(),
            input: serde_json::json!({}),
            caller: Some("code_execution_20250825".into()),
            cache_control: None,
        };
        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["caller"], "code_execution_20250825");
    }

    #[test]
    fn test_tool_use_without_caller() {
        let block = ContentBlock::ToolUse {
            id: "tu_123".into(),
            name: "my_tool".into(),
            input: serde_json::json!({}),
            caller: None,
            cache_control: None,
        };
        let json = serde_json::to_value(&block).unwrap();
        assert!(json.get("caller").is_none());
    }

    #[test]
    fn test_server_tool_use_in_response() {
        let json = r#"{
            "id": "msg_123",
            "type": "message",
            "role": "assistant",
            "content": [
                {"type": "text", "text": "searching..."},
                {"type": "server_tool_use", "id": "stu_1", "name": "web_search", "input": {"query": "rust"}}
            ],
            "model": "claude-sonnet-4-5-20250929",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;

        let response: MessagesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.content.len(), 2);
        match &response.content[1] {
            ContentBlock::ServerToolUse { id, name, .. } => {
                assert_eq!(id, "stu_1");
                assert_eq!(name, "web_search");
            }
            _ => panic!("Expected ServerToolUse variant"),
        }
    }
}
