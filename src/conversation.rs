//! Conversation builder for multi-turn interactions.
//!
//! This module provides [`ConversationBuilder`], a fluent API for managing
//! multi-turn conversations with Claude, including tool use and state management.
//!
//! # When to Use ConversationBuilder
//!
//! Use `ConversationBuilder` when you need to:
//! - Maintain conversation history across multiple turns
//! - Handle tool use/result cycles automatically
//! - Cache system prompts or tools for cost savings
//! - Estimate token usage before sending requests
//!
//! For single-turn requests, use [`MessagesRequest`] directly.
//!
//! # Basic Multi-Turn Conversation
//!
//! ```rust,no_run
//! use claude_sdk::{ClaudeClient, ConversationBuilder, ContentBlock};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ClaudeClient::anthropic(std::env::var("ANTHROPIC_API_KEY")?);
//!
//! let mut conversation = ConversationBuilder::new()
//!     .with_system("You are a helpful assistant.");
//!
//! // First turn
//! conversation.add_user_message("What's the capital of France?");
//! let response = client.send_message(
//!     conversation.build("claude-sonnet-4-5-20250929", 256)
//! ).await?;
//!
//! // Extract and add assistant response
//! let assistant_text = response.content.iter()
//!     .filter_map(|b| match b {
//!         ContentBlock::Text { text, .. } => Some(text.as_str()),
//!         _ => None,
//!     })
//!     .collect::<Vec<_>>()
//!     .join("");
//! conversation.add_assistant_message(&assistant_text);
//!
//! // Second turn (conversation history is maintained)
//! conversation.add_user_message("What's its population?");
//! let response = client.send_message(
//!     conversation.build("claude-sonnet-4-5-20250929", 256)
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Tool Use Workflow
//!
//! Handle tool calls in a multi-turn conversation:
//!
//! ```rust,no_run
//! use claude_sdk::{ClaudeClient, ConversationBuilder, Tool, ContentBlock, StopReason};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ClaudeClient::anthropic(std::env::var("ANTHROPIC_API_KEY")?);
//!
//! let weather_tool = Tool {
//!     name: "get_weather".into(),
//!     description: "Get current weather for a city".into(),
//!     input_schema: json!({
//!         "type": "object",
//!         "properties": { "city": { "type": "string" } },
//!         "required": ["city"]
//!     }),
//!     disable_user_input: Some(true),
//!     input_examples: None,
//!     cache_control: None,
//! };
//!
//! let mut conversation = ConversationBuilder::new()
//!     .with_tool(weather_tool);
//!
//! conversation.add_user_message("What's the weather in Tokyo?");
//!
//! // Send request
//! let response = client.send_message(
//!     conversation.build("claude-sonnet-4-5-20250929", 1024)
//! ).await?;
//!
//! // Check if Claude wants to use a tool
//! if response.stop_reason == Some(StopReason::ToolUse) {
//!     // Add Claude's response with tool use blocks
//!     conversation.add_assistant_with_blocks(response.content.clone());
//!
//!     // Process each tool call
//!     for block in &response.content {
//!         if let ContentBlock::ToolUse { id, name, input, .. } = block {
//!             // Execute the tool (your implementation)
//!             let result = json!({"temperature": 22, "condition": "sunny"});
//!
//!             // Add the result
//!             conversation.add_tool_result(id, result.to_string());
//!         }
//!     }
//!
//!     // Send again for final response
//!     let final_response = client.send_message(
//!         conversation.build("claude-sonnet-4-5-20250929", 1024)
//!     ).await?;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Prompt Caching
//!
//! Cache system prompts and tools to reduce costs on repeated requests:
//!
//! ```rust
//! use claude_sdk::{ConversationBuilder, Tool};
//! use serde_json::json;
//!
//! // Cache a long system prompt (~5 min TTL, 90% cost reduction on cache hits)
//! let conversation = ConversationBuilder::new()
//!     .with_cached_system("You are an expert assistant with extensive knowledge...");
//!
//! // Cache tool definitions
//! let tool = Tool {
//!     name: "analyze".into(),
//!     description: "Analyze data".into(),
//!     input_schema: json!({"type": "object"}),
//!     disable_user_input: Some(true),
//!     input_examples: None,
//!     cache_control: None,
//! };
//!
//! let conversation = ConversationBuilder::new()
//!     .with_cached_tool(tool);
//! ```
//!
//! # Token Management
//!
//! Check if your conversation fits in the context window:
//!
//! ```rust
//! use claude_sdk::{ConversationBuilder, models};
//!
//! let mut conversation = ConversationBuilder::new()
//!     .with_system("You are helpful.");
//! conversation.add_user_message("Tell me about Rust programming.");
//!
//! // Estimate tokens
//! let tokens = conversation.estimate_tokens();
//! println!("Estimated tokens: {}", tokens);
//!
//! // Check against model limits
//! let model = &models::CLAUDE_SONNET_4_5;
//! if conversation.fits_in_context(model, 1024, false) {
//!     println!("Conversation fits!");
//! }
//! ```

use crate::types::{
    CacheControl, ContentBlock, Message, MessagesRequest, Role, SystemBlock, SystemPrompt, Tool,
};

/// Builder for managing multi-turn conversations with Claude
///
/// This builder helps manage conversation state, including:
/// - Message history with proper role alternation
/// - Tool definitions
/// - System prompts
/// - Automatic handling of tool use/result cycles
///
/// # Example
///
/// ```rust
/// use claude_sdk::ConversationBuilder;
///
/// let mut conversation = ConversationBuilder::new()
///     .with_system("You are a helpful assistant");
///
/// conversation.add_user_message("Hello!");
///
/// let request = conversation.build("claude-sonnet-4-5-20250929", 1024);
/// ```
#[derive(Debug, Clone)]
pub struct ConversationBuilder {
    messages: Vec<Message>,
    tools: Vec<Tool>,
    system: Option<SystemPrompt>,
}

impl ConversationBuilder {
    /// Create a new conversation builder
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            tools: Vec::new(),
            system: None,
        }
    }

    /// Set the system prompt
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::ConversationBuilder;
    ///
    /// let conversation = ConversationBuilder::new()
    ///     .with_system("You are a helpful coding assistant");
    /// ```
    pub fn with_system(mut self, prompt: impl Into<String>) -> Self {
        self.system = Some(SystemPrompt::String(prompt.into()));
        self
    }

    /// Set the system prompt with caching enabled
    ///
    /// This caches the system prompt for ~5 minutes, reducing costs on repeated requests.
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::ConversationBuilder;
    ///
    /// let conversation = ConversationBuilder::new()
    ///     .with_cached_system("You are a helpful assistant with access to tools");
    /// ```
    pub fn with_cached_system(mut self, prompt: impl Into<String>) -> Self {
        self.system = Some(SystemPrompt::Blocks(vec![SystemBlock {
            block_type: "text".into(),
            text: prompt.into(),
            cache_control: Some(CacheControl::ephemeral()),
        }]));
        self
    }

    /// Add a tool definition
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::{ConversationBuilder, Tool};
    /// use serde_json::json;
    ///
    /// let tool = Tool {
    ///     name: "get_weather".into(),
    ///     description: "Get weather for a location".into(),
    ///     input_schema: json!({
    ///         "type": "object",
    ///         "properties": {
    ///             "location": {"type": "string"}
    ///         },
    ///         "required": ["location"]
    ///     }),
    ///     disable_user_input: Some(true),
    ///     input_examples: None,
    ///     cache_control: None,
    /// };
    ///
    /// let conversation = ConversationBuilder::new()
    ///     .with_tool(tool);
    /// ```
    pub fn with_tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    /// Add multiple tool definitions
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Add a tool with caching enabled
    ///
    /// This caches the tool definition, reducing costs when using the same tools repeatedly.
    pub fn with_cached_tool(mut self, mut tool: Tool) -> Self {
        tool.cache_control = Some(CacheControl::ephemeral());
        self.tools.push(tool);
        self
    }

    /// Add a user message
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::ConversationBuilder;
    ///
    /// let mut conversation = ConversationBuilder::new();
    /// conversation.add_user_message("What's the weather in NYC?");
    /// ```
    pub fn add_user_message(&mut self, content: impl Into<String>) -> &mut Self {
        self.messages.push(Message::user(content));
        self
    }

    /// Add an assistant message
    ///
    /// Typically used when reconstructing a conversation from history.
    pub fn add_assistant_message(&mut self, content: impl Into<String>) -> &mut Self {
        self.messages.push(Message::assistant(content));
        self
    }

    /// Add an assistant message with content blocks
    ///
    /// Used when the assistant response includes tool use.
    pub fn add_assistant_with_blocks(&mut self, content: Vec<ContentBlock>) -> &mut Self {
        self.messages.push(Message {
            role: Role::Assistant,
            content,
        });
        self
    }

    /// Add a tool result
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::ConversationBuilder;
    ///
    /// let mut conversation = ConversationBuilder::new();
    /// conversation.add_tool_result("toolu_123", r#"{"temp": 72, "condition": "sunny"}"#);
    /// ```
    pub fn add_tool_result(
        &mut self,
        tool_use_id: impl Into<String>,
        result: impl Into<String>,
    ) -> &mut Self {
        self.messages
            .push(Message::tool_result(tool_use_id, result));
        self
    }

    /// Add a tool result with error flag
    pub fn add_tool_error(
        &mut self,
        tool_use_id: impl Into<String>,
        error_message: impl Into<String>,
    ) -> &mut Self {
        self.messages.push(Message {
            role: Role::User,
            content: vec![ContentBlock::ToolResult {
                tool_use_id: tool_use_id.into(),
                content: Some(error_message.into()),
                is_error: Some(true),
            }],
        });
        self
    }

    /// Get the current message history
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Get the tool definitions
    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    /// Get the system prompt
    pub fn system(&self) -> Option<&SystemPrompt> {
        self.system.as_ref()
    }

    /// Clear all messages but keep tools and system prompt
    pub fn clear_messages(&mut self) -> &mut Self {
        self.messages.clear();
        self
    }

    /// Build a MessagesRequest from the current conversation state
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::ConversationBuilder;
    ///
    /// let mut conversation = ConversationBuilder::new()
    ///     .with_system("You are helpful");
    ///
    /// conversation.add_user_message("Hello!");
    ///
    /// let request = conversation.build("claude-sonnet-4-5-20250929", 1024);
    /// ```
    pub fn build(&self, model: impl Into<String>, max_tokens: u32) -> MessagesRequest {
        let mut request = MessagesRequest::new(model, max_tokens, self.messages.clone());

        if let Some(system) = &self.system {
            request.system = Some(system.clone());
        }

        if !self.tools.is_empty() {
            request.tools = Some(self.tools.clone());
        }

        request
    }

    /// Build a request and consume the builder
    pub fn into_request(self, model: impl Into<String>, max_tokens: u32) -> MessagesRequest {
        self.build(model, max_tokens)
    }

    /// Estimate the number of tokens in the current conversation
    ///
    /// This includes system prompt, tools, and all messages.
    /// Useful for managing context window limits.
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::ConversationBuilder;
    ///
    /// let mut conversation = ConversationBuilder::new()
    ///     .with_system("You are helpful");
    ///
    /// conversation.add_user_message("Hello!");
    ///
    /// let tokens = conversation.estimate_tokens();
    /// println!("Conversation uses ~{} tokens", tokens);
    /// ```
    pub fn estimate_tokens(&self) -> usize {
        let counter = crate::tokens::TokenCounter::new();
        let mut total = 0;

        // System prompt
        if let Some(system) = &self.system {
            total += counter.count_system_prompt(system);
        }

        // Messages
        for message in &self.messages {
            total += counter.count_message(message);
        }

        // Tools
        for tool in &self.tools {
            total += counter.count_tool(tool);
        }

        total
    }

    /// Check if the conversation would fit in a model's context window
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::{ConversationBuilder, models};
    ///
    /// let mut conversation = ConversationBuilder::new();
    /// conversation.add_user_message("Hello!");
    ///
    /// let model = &models::CLAUDE_SONNET_4_5;
    /// let fits = conversation.fits_in_context(model, 1024, false);
    /// println!("Fits: {}", fits);
    /// ```
    pub fn fits_in_context(
        &self,
        model: &crate::models::Model,
        max_tokens: u32,
        use_extended_context: bool,
    ) -> bool {
        let counter = crate::tokens::TokenCounter::new();
        let request = self.build(model.anthropic_id, max_tokens);
        counter
            .validate_context_window(&request, model, use_extended_context)
            .is_ok()
    }
}

impl Default for ConversationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_conversation() {
        let mut conv = ConversationBuilder::new();
        conv.add_user_message("Hello");
        conv.add_assistant_message("Hi there!");
        conv.add_user_message("How are you?");

        assert_eq!(conv.messages().len(), 3);
        assert_eq!(conv.messages()[0].role, Role::User);
        assert_eq!(conv.messages()[1].role, Role::Assistant);
    }

    #[test]
    fn test_with_system() {
        let conv = ConversationBuilder::new().with_system("You are helpful");

        assert!(conv.system().is_some());
        match conv.system().unwrap() {
            SystemPrompt::String(s) => assert_eq!(s, "You are helpful"),
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_with_tools() {
        let tool = Tool {
            name: "test_tool".into(),
            description: "A test tool".into(),
            input_schema: json!({"type": "object"}),
            disable_user_input: None,
            input_examples: None,
            cache_control: None,
        };

        let conv = ConversationBuilder::new().with_tool(tool);

        assert_eq!(conv.tools().len(), 1);
        assert_eq!(conv.tools()[0].name, "test_tool");
    }

    #[test]
    fn test_tool_result() {
        let mut conv = ConversationBuilder::new();
        conv.add_tool_result("toolu_123", "success");

        assert_eq!(conv.messages().len(), 1);
        assert_eq!(conv.messages()[0].role, Role::User);

        match &conv.messages()[0].content[0] {
            ContentBlock::ToolResult { tool_use_id, .. } => {
                assert_eq!(tool_use_id, "toolu_123");
            }
            _ => panic!("Expected ToolResult"),
        }
    }

    #[test]
    fn test_build_request() {
        let mut conv = ConversationBuilder::new().with_system("Test system");
        conv.add_user_message("Test message");

        let request = conv.build("claude-sonnet-4-5-20250929", 1024);

        assert_eq!(request.model, "claude-sonnet-4-5-20250929");
        assert_eq!(request.max_tokens, 1024);
        assert_eq!(request.messages.len(), 1);
        assert!(request.system.is_some());
    }

    #[test]
    fn test_clear_messages() {
        let mut conv = ConversationBuilder::new().with_system("System");
        conv.add_user_message("Message 1");
        conv.add_user_message("Message 2");

        assert_eq!(conv.messages().len(), 2);

        conv.clear_messages();
        assert_eq!(conv.messages().len(), 0);
        assert!(conv.system().is_some()); // System preserved
    }

    #[test]
    fn test_cached_system() {
        let conv = ConversationBuilder::new().with_cached_system("Cached prompt");

        match conv.system().unwrap() {
            SystemPrompt::Blocks(blocks) => {
                assert_eq!(blocks.len(), 1);
                assert!(blocks[0].cache_control.is_some());
            }
            _ => panic!("Expected Blocks variant"),
        }
    }
}
