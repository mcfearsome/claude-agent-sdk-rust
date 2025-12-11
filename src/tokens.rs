//! Token counting utilities for context window management
//!
//! Claude models use the cl100k_base tokenizer (same as GPT-4).
//! This module provides accurate token counting for messages, tools, and system prompts.

use crate::types::{ContentBlock, Message, MessagesRequest, SystemPrompt, Tool};
use tiktoken_rs::cl100k_base;

/// Token counter for Claude API requests
///
/// Uses the cl100k_base tokenizer for accurate token estimation.
///
/// # Example
///
/// ```rust
/// use claude_sdk::tokens::TokenCounter;
/// use claude_sdk::Message;
///
/// let counter = TokenCounter::new();
/// let tokens = counter.count_message(&Message::user("Hello, Claude!"));
/// println!("Message uses {} tokens", tokens);
/// ```
pub struct TokenCounter {
    bpe: tiktoken_rs::CoreBPE,
}

impl TokenCounter {
    /// Create a new token counter
    ///
    /// Uses the cl100k_base tokenizer (same as Claude models).
    pub fn new() -> Self {
        Self {
            bpe: cl100k_base().expect("Failed to load cl100k_base tokenizer"),
        }
    }

    /// Count tokens in a text string
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::tokens::TokenCounter;
    ///
    /// let counter = TokenCounter::new();
    /// let tokens = counter.count_text("Hello, world!");
    /// assert!(tokens > 0);
    /// ```
    pub fn count_text(&self, text: &str) -> usize {
        self.bpe.encode_with_special_tokens(text).len()
    }

    /// Count tokens in a message
    ///
    /// Includes tokens for role and content formatting.
    pub fn count_message(&self, message: &Message) -> usize {
        let mut total = 3; // Base overhead for message structure

        // Role token
        total += 1;

        // Content blocks
        for content in &message.content {
            total += self.count_content_block(content);
        }

        total
    }

    /// Count tokens in a content block
    pub fn count_content_block(&self, content: &ContentBlock) -> usize {
        match content {
            ContentBlock::Text { text, .. } => {
                let mut total = 2; // Type overhead
                total += self.count_text(text);
                total
            }
            ContentBlock::ToolUse { name, input, .. } => {
                let mut total = 4; // Type and structure overhead
                total += self.count_text(name);
                total += self.count_text(&input.to_string());
                total
            }
            ContentBlock::ToolResult {
                tool_use_id,
                content,
                ..
            } => {
                let mut total = 4; // Type and structure overhead
                total += self.count_text(tool_use_id);
                if let Some(text) = content {
                    total += self.count_text(text);
                }
                total
            }
        }
    }

    /// Count tokens in a system prompt
    pub fn count_system_prompt(&self, system: &SystemPrompt) -> usize {
        match system {
            SystemPrompt::String(s) => self.count_text(s),
            SystemPrompt::Blocks(blocks) => blocks
                .iter()
                .map(|block| self.count_text(&block.text))
                .sum(),
        }
    }

    /// Count tokens in a tool definition
    ///
    /// Tools add overhead to the system prompt.
    pub fn count_tool(&self, tool: &Tool) -> usize {
        let mut total = 10; // Base overhead for tool structure

        total += self.count_text(&tool.name);
        total += self.count_text(&tool.description);
        total += self.count_text(&tool.input_schema.to_string());

        total
    }

    /// Count total tokens in a request
    ///
    /// This estimates the input tokens that will be charged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::{MessagesRequest, Message, tokens::TokenCounter};
    ///
    /// let request = MessagesRequest::new(
    ///     "claude-sonnet-4-5-20250929",
    ///     1024,
    ///     vec![Message::user("Hello!")],
    /// ).with_system("You are helpful");
    ///
    /// let counter = TokenCounter::new();
    /// let tokens = counter.count_request(&request);
    /// println!("Request will use ~{} input tokens", tokens);
    /// ```
    pub fn count_request(&self, request: &MessagesRequest) -> usize {
        let mut total = 0;

        // System prompt
        if let Some(system) = &request.system {
            total += self.count_system_prompt(system);
        }

        // Messages
        for message in &request.messages {
            total += self.count_message(message);
        }

        // Tools
        if let Some(tools) = &request.tools {
            for tool in tools {
                total += self.count_tool(tool);
            }
        }

        // Small overhead for request structure
        total += 10;

        total
    }

    /// Validate that a request fits within a model's context window
    ///
    /// Returns Ok(()) if the request fits, or Err with details if it exceeds limits.
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::{MessagesRequest, Message, tokens::TokenCounter, models};
    ///
    /// let request = MessagesRequest::new(
    ///     models::CLAUDE_SONNET_4_5.anthropic_id,
    ///     1024,
    ///     vec![Message::user("Hello!")],
    /// );
    ///
    /// let counter = TokenCounter::new();
    /// let model = &models::CLAUDE_SONNET_4_5;
    ///
    /// match counter.validate_context_window(&request, model, false) {
    ///     Ok(()) => println!("Request fits in context window"),
    ///     Err(e) => println!("Request too large: {}", e),
    /// }
    /// ```
    pub fn validate_context_window(
        &self,
        request: &MessagesRequest,
        model: &crate::models::Model,
        use_extended_context: bool,
    ) -> Result<(), String> {
        let input_tokens = self.count_request(request);
        let max_output = request.max_tokens as usize;
        let total_tokens = input_tokens + max_output;

        let context_limit = if use_extended_context {
            model
                .max_extended_context()
                .unwrap_or(model.max_context_tokens) as usize
        } else {
            model.max_context_tokens as usize
        };

        if total_tokens > context_limit {
            return Err(format!(
                "Request would use ~{} tokens (input: {}, output: {}) but model {} has {} token limit{}",
                total_tokens,
                input_tokens,
                max_output,
                model.name,
                context_limit,
                if use_extended_context { " (extended)" } else { "" }
            ));
        }

        Ok(())
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Message, MessagesRequest};
    use serde_json::json;

    #[test]
    fn test_count_text() {
        let counter = TokenCounter::new();

        // Simple text
        let tokens = counter.count_text("Hello, world!");
        assert!(tokens > 0);
        assert!(tokens < 10); // Should be around 4 tokens

        // Empty string
        assert_eq!(counter.count_text(""), 0);

        // Long text should have more tokens
        let short = counter.count_text("Hi");
        let long = counter.count_text("This is a much longer sentence with more words");
        assert!(long > short);
    }

    #[test]
    fn test_count_message() {
        let counter = TokenCounter::new();

        let msg = Message::user("Hello, Claude!");
        let tokens = counter.count_message(&msg);

        // Should include message overhead + text tokens
        assert!(tokens > 5); // At least overhead + some content
    }

    #[test]
    fn test_count_system_prompt() {
        let counter = TokenCounter::new();

        // String variant
        let system = SystemPrompt::String("You are a helpful assistant".into());
        let tokens = counter.count_system_prompt(&system);
        assert!(tokens > 0);
    }

    #[test]
    fn test_count_tool() {
        let counter = TokenCounter::new();

        let tool = Tool {
            name: "get_weather".into(),
            description: "Get the current weather".into(),
            input_schema: json!({"type": "object"}),
            disable_user_input: None,
            input_examples: None,
            cache_control: None,
        };

        let tokens = counter.count_tool(&tool);
        assert!(tokens > 10); // Should include overhead + content
    }

    #[test]
    fn test_count_request() {
        let counter = TokenCounter::new();

        let request = MessagesRequest::new(
            "claude-sonnet-4-5-20250929",
            1024,
            vec![Message::user("Hello!")],
        )
        .with_system("You are helpful");

        let tokens = counter.count_request(&request);
        assert!(tokens > 0);
        assert!(tokens < 100); // Small request should be under 100 tokens
    }

    #[test]
    fn test_validate_context_window() {
        let counter = TokenCounter::new();
        let model = &crate::models::CLAUDE_SONNET_4_5;

        // Small request should pass
        let small_request =
            MessagesRequest::new(model.anthropic_id, 1024, vec![Message::user("Hello!")]);

        assert!(counter
            .validate_context_window(&small_request, model, false)
            .is_ok());

        // Huge max_tokens should fail
        let huge_request = MessagesRequest::new(
            model.anthropic_id,
            200_000, // Way too large
            vec![Message::user("Hello!")],
        );

        assert!(counter
            .validate_context_window(&huge_request, model, false)
            .is_err());
    }

    #[test]
    fn test_validate_extended_context() {
        let counter = TokenCounter::new();
        let model = &crate::models::CLAUDE_SONNET_4_5;

        // Request that exceeds standard context (200K) but fits in extended (1M)
        // Use 201K max_tokens to guarantee we exceed the 200K limit
        let request =
            MessagesRequest::new(model.anthropic_id, 201_000, vec![Message::user("Hello!")]);

        // Should fail with standard context (200K limit)
        assert!(
            counter
                .validate_context_window(&request, model, false)
                .is_err(),
            "Should fail with standard 200K context"
        );

        // Should pass with extended context (1M tokens)
        assert!(
            counter
                .validate_context_window(&request, model, true)
                .is_ok(),
            "Should pass with extended 1M context"
        );
    }
}
