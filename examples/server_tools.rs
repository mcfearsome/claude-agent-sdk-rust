//! Server tools example demonstrating built-in Claude tools
//!
//! This example shows how to use server-side tools that Claude
//! executes directly (web search, code execution) alongside
//! custom client-side tools.
//!
//! Run with:
//! ```bash
//! cargo run --example server_tools
//! ```

use claude_sdk::server_tools::{CodeExecutionTool, WebSearchTool};
use claude_sdk::{CustomTool, Message, MessagesRequest, ToolDefinition};
use serde_json::json;

fn main() {
    println!("Claude SDK - Server Tools Example");
    println!("=====================================\n");

    // 1. Web search with domain filtering
    let web_search = WebSearchTool::new()
        .with_allowed_domains(vec!["docs.rs".into(), "crates.io".into()])
        .with_max_uses(5);

    println!("Web Search Tool:");
    println!("  Type: {}", web_search.tool_type);
    println!("  Allowed domains: {:?}", web_search.allowed_domains);
    println!("  Max uses: {:?}\n", web_search.max_uses);

    // 2. Code execution (sandboxed Python)
    let code_exec = CodeExecutionTool::new();

    println!("Code Execution Tool:");
    println!("  Type: {}\n", code_exec.tool_type);

    // 3. Custom tool alongside server tools
    let custom_tool = CustomTool::new(
        "format_output",
        "Format data into a specific output structure",
        json!({
            "type": "object",
            "properties": {
                "title": {"type": "string"},
                "data": {"type": "array", "items": {"type": "string"}}
            },
            "required": ["title", "data"]
        }),
    )
    .programmatic()
    .with_strict();

    println!("Custom Tool:");
    println!("  Name: {}", custom_tool.name);
    println!("  Strict: {:?}\n", custom_tool.strict);

    // 4. Build request with all three tool types
    let request = MessagesRequest::new(
        "claude-sonnet-4-5-20250929",
        4096,
        vec![Message::user(
            "Search for the latest version of the tokio crate, \
             then write Python code to calculate how many days \
             since its release date, and format the results.",
        )],
    )
    .with_tools(vec![
        ToolDefinition::from(web_search),
        ToolDefinition::from(code_exec),
        ToolDefinition::Custom(custom_tool),
    ]);

    // Print the request JSON to show the structure
    println!("Request JSON (tools section):");
    let json = serde_json::to_string_pretty(&request.tools.as_ref().unwrap()).unwrap();
    println!("{}\n", json);

    println!("To send this request, set ANTHROPIC_API_KEY and use:");
    println!("  let response = client.send_message(request).await?;");
}
