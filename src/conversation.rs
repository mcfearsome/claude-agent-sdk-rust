//! Conversation builder for multi-turn interactions
//!
//! This module provides a builder for managing conversation state,
//! including tool use and multi-turn exchanges.

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
