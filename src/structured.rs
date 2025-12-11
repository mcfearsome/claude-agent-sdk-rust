//! Structured outputs using forced tool use
//!
//! This module provides helpers for getting structured JSON outputs from Claude
//! by using tool_choice to force a specific tool that returns the desired schema.

use crate::types::{Tool, ToolChoice};
use serde_json::Value;

/// Create a tool for structured JSON extraction
///
/// This is a convenience function for creating a tool that forces Claude
/// to return structured JSON matching your schema.
///
/// # Example
///
/// ```rust
/// use claude_sdk::structured::json_schema_tool;
/// use serde_json::json;
///
/// let schema = json!({
///     "type": "object",
///     "properties": {
///         "name": {"type": "string"},
///         "age": {"type": "number"},
///         "email": {"type": "string"}
///     },
///     "required": ["name", "age"]
/// });
///
/// let tool = json_schema_tool(
///     "extract_person",
///     "Extract person information from text",
///     schema
/// );
/// ```
pub fn json_schema_tool(
    name: impl Into<String>,
    description: impl Into<String>,
    schema: Value,
) -> Tool {
    Tool {
        name: name.into(),
        description: description.into(),
        input_schema: schema,
        disable_user_input: Some(true),
        input_examples: None,
        cache_control: None,
    }
}

/// Create a forced tool choice for structured outputs
///
/// Use this with a JSON schema tool to guarantee structured output.
///
/// # Example
///
/// ```rust
/// use claude_sdk::structured::{json_schema_tool, force_tool};
/// use claude_sdk::MessagesRequest;
/// use serde_json::json;
///
/// let tool = json_schema_tool(
///     "extract_data",
///     "Extract structured data",
///     json!({"type": "object", "properties": {}})
/// );
///
/// let request = MessagesRequest::new(
///     "claude-sonnet-4-5-20250929",
///     1024,
///     vec![/*messages*/]
/// )
/// .with_tools(vec![tool.clone()])
/// .with_tool_choice(force_tool("extract_data"));
/// ```
pub fn force_tool(tool_name: impl Into<String>) -> ToolChoice {
    ToolChoice::Tool {
        name: tool_name.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_schema_tool() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            }
        });

        let tool = json_schema_tool("test_tool", "Test description", schema.clone());

        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "Test description");
        assert_eq!(tool.input_schema, schema);
        assert_eq!(tool.disable_user_input, Some(true));
    }

    #[test]
    fn test_force_tool() {
        let choice = force_tool("my_tool");

        match choice {
            ToolChoice::Tool { name } => assert_eq!(name, "my_tool"),
            _ => panic!("Expected Tool variant"),
        }
    }
}
