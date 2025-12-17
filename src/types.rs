//! Type definitions for Claude API requests and responses.
//!
//! This module contains all the core types used for communicating with the Claude API:
//!
//! - [`MessagesRequest`] - The main request type for sending messages
//! - [`MessagesResponse`] - Response from the Messages API
//! - [`Message`] - Conversation messages with user/assistant roles
//! - [`ContentBlock`] - Text, images, documents, tool calls, and more
//! - [`Tool`] - Tool definitions for function calling
//! - [`ToolChoice`] - Control how Claude uses tools
//!
//! # Example
//!
//! ```rust
//! use claude_sdk::types::{MessagesRequest, Message, Tool, ToolChoice};
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// Tool result from the user
    ToolResult {
        tool_use_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CacheType {
    Ephemeral,
}

impl CacheControl {
    /// Create an ephemeral cache control
    pub fn ephemeral() -> Self {
        Self {
            cache_type: CacheType::Ephemeral,
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
                content: Some(content.into()),
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

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
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

/// Tool choice configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolChoice {
    /// Let Claude decide whether to use tools (default)
    Auto,
    /// Claude must use one of the provided tools
    Any,
    /// Force Claude to use a specific tool
    Tool { name: String },
    /// Prevent Claude from using any tools
    None,
}

impl ToolChoice {
    /// Create Auto variant
    pub fn auto() -> Self {
        Self::Auto
    }

    /// Create Any variant
    pub fn any() -> Self {
        Self::Any
    }

    /// Create Tool variant with specific tool name
    pub fn tool(name: impl Into<String>) -> Self {
        Self::Tool { name: name.into() }
    }

    /// Create None variant
    pub fn none() -> Self {
        Self::None
    }
}

/// Token usage information
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,

    /// Tokens written to cache (prompt caching)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,

    /// Tokens read from cache (prompt caching)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
}

/// Extended usage information for responses with thinking
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

    /// Available tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Tool choice configuration
    ///
    /// Controls how Claude uses tools:
    /// - `Auto` (default): Claude decides whether to use tools
    /// - `Any`: Claude must use one of the provided tools
    /// - `Tool { name }`: Force Claude to use a specific tool
    /// - `None`: Prevent Claude from using any tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Disable parallel tool use
    ///
    /// When true with `tool_choice: auto`, Claude will use at most one tool.
    /// When true with `tool_choice: any/tool`, Claude will use exactly one tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_parallel_tool_use: Option<bool>,

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
    },
    /// Disable extended thinking
    Disabled,
}

/// Output configuration for controlling response behavior
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
            disable_parallel_tool_use: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            stream: None,
            output_config: None,
            thinking: None,
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
    /// Tools allow Claude to call functions and return structured data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::{MessagesRequest, Message, Tool};
    /// use serde_json::json;
    ///
    /// let calculator = Tool {
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
    /// };
    ///
    /// let request = MessagesRequest::new(
    ///     "claude-sonnet-4-5-20250929",
    ///     1024,
    ///     vec![Message::user("What's 15 * 7?")],
    /// )
    /// .with_tools(vec![calculator]);
    /// ```
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
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

    /// Disable parallel tool use.
    ///
    /// When enabled, Claude will only use one tool at a time instead of
    /// potentially calling multiple tools in parallel.
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::{MessagesRequest, Message};
    ///
    /// let request = MessagesRequest::new(
    ///     "claude-sonnet-4-5-20250929",
    ///     1024,
    ///     vec![Message::user("Get weather and stock price")],
    /// )
    /// .with_disable_parallel_tool_use(true);  // Forces sequential tool calls
    /// ```
    pub fn with_disable_parallel_tool_use(mut self, disable: bool) -> Self {
        self.disable_parallel_tool_use = Some(disable);
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
        self.output_config = Some(OutputConfig {
            effort: Some(effort),
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
        self.thinking = Some(ThinkingConfig::Enabled { budget_tokens });
        self
    }
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

    pub usage: Usage,
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
        let tool = Tool {
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
}
