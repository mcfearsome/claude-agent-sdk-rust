//! Interactive REPL for Claude SDK
//!
//! A terminal-based REPL for testing and interacting with Claude.
//!
//! Features:
//! - Multi-turn conversations with streaming
//! - Tool execution preview
//! - Token counting display
//! - Conversation save/load
//! - Slash commands for configuration
//! - Both Anthropic and Bedrock backends
//!
//! Run with:
//! ```bash
//! export ANTHROPIC_API_KEY="your-api-key"
//! cargo run --bin claude-repl
//! ```

use claude_sdk::{models, ClaudeClient, ContentBlock, ConversationBuilder, StreamEvent, Tool};
use futures::StreamExt;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};

#[derive(Clone, Serialize, Deserialize)]
struct ReplConfig {
    model_id: String,
    max_tokens: u32,
    backend: String, // "anthropic" or "bedrock"
    region: Option<String>,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            model_id: models::CLAUDE_SONNET_4_5.anthropic_id.to_string(),
            max_tokens: 4096,
            backend: "anthropic".to_string(),
            region: None,
        }
    }
}

struct Repl {
    client: ClaudeClient,
    conversation: ConversationBuilder,
    editor: DefaultEditor,
    config: ReplConfig,
    token_counter: claude_sdk::tokens::TokenCounter,
}

impl Repl {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = ReplConfig::default();

        // Create client based on backend
        let client = Self::create_client(&config).await?;

        let editor = DefaultEditor::new()?;
        let conversation = ConversationBuilder::new()
            .with_system("You are Claude, a helpful AI assistant created by Anthropic.");

        Ok(Self {
            client,
            conversation,
            editor,
            config,
            token_counter: claude_sdk::tokens::TokenCounter::new(),
        })
    }

    async fn create_client(
        config: &ReplConfig,
    ) -> Result<ClaudeClient, Box<dyn std::error::Error>> {
        match config.backend.as_str() {
            "anthropic" => {
                let api_key = std::env::var("ANTHROPIC_API_KEY")?;
                Ok(ClaudeClient::anthropic(api_key))
            }
            #[cfg(feature = "bedrock")]
            "bedrock" => {
                let region = config
                    .region
                    .as_ref()
                    .ok_or("Bedrock backend requires region")?;
                ClaudeClient::bedrock(region).await.map_err(|e| e.into())
            }
            _ => Err("Unknown backend".into()),
        }
    }

    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.print_welcome();

        loop {
            let prompt = format!(
                "You [{}]> ",
                self.token_counter.count_request(
                    &self
                        .conversation
                        .build(&self.config.model_id, self.config.max_tokens)
                )
            );

            match self.editor.readline(&prompt) {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    self.editor.add_history_entry(line)?;

                    if line.starts_with('/') {
                        if let Err(e) = self.handle_command(line).await {
                            eprintln!("Command error: {}", e);
                        }
                    } else if let Err(e) = self.send_message(line).await {
                        eprintln!("Error: {}", e);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        println!("\nGoodbye!");
        Ok(())
    }

    fn print_welcome(&self) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!(
            "║           Claude SDK REPL v{}                         ║",
            env!("CARGO_PKG_VERSION")
        );
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  Model: {}", self.config.model_id);
        println!("  Backend: {}", self.config.backend);
        println!("  Max tokens: {}", self.config.max_tokens);
        println!();
        println!("  Type /help for commands");
        println!("  Press Ctrl+C to interrupt, Ctrl+D to exit");
        println!();
    }

    async fn send_message(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.conversation.add_user_message(content);

        let request = self
            .conversation
            .build(&self.config.model_id, self.config.max_tokens);

        print!("Claude> ");
        io::stdout().flush()?;

        let mut stream = self.client.send_streaming(request).await?;
        let mut response_content = Vec::new();
        let mut response_text = String::new();
        let mut input_tokens = 0;
        let mut output_tokens = 0;

        while let Some(event_result) = stream.next().await {
            match event_result? {
                StreamEvent::MessageStart { message } => {
                    input_tokens = message.usage.input_tokens;
                }

                StreamEvent::ContentBlockStart { content_block, .. } => {
                    response_content.push(content_block.clone());

                    if let ContentBlock::ToolUse { name, input, .. } = content_block {
                        println!("\n🔧 [Tool: {} {}]", name, input);
                        print!("Claude> ");
                        io::stdout().flush()?;
                    }
                }

                StreamEvent::ContentBlockDelta { index, delta } => {
                    if let Some(text) = delta.text() {
                        print!("{}", text);
                        io::stdout().flush()?;
                        response_text.push_str(text);

                        // Update stored text
                        if let Some(ContentBlock::Text { text: stored, .. }) =
                            response_content.get_mut(index)
                        {
                            stored.push_str(text);
                        }
                    }
                }

                StreamEvent::MessageDelta { usage, .. } => {
                    output_tokens = usage.output_tokens;
                }

                StreamEvent::MessageStop => break,

                StreamEvent::Error { error } => {
                    eprintln!("\n❌ Error: {}", error.message);
                    return Err(error.message.into());
                }

                _ => {}
            }
        }

        println!();

        // Add assistant response to conversation
        self.conversation
            .add_assistant_with_blocks(response_content);

        // Show token usage
        println!(
            "📊 [in: {}, out: {}, total: {}]",
            input_tokens,
            output_tokens,
            input_tokens + output_tokens
        );
        println!();

        Ok(())
    }

    async fn handle_command(&mut self, cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let command = parts[0];

        match command {
            "/help" => self.show_help(),
            "/clear" => self.clear_conversation(),
            "/save" => self.save_conversation().await?,
            "/load" => self.load_conversation().await?,
            "/tokens" => self.show_tokens(),
            "/model" => self.change_model(&parts[1..])?,
            "/backend" => self.change_backend(&parts[1..]).await?,
            "/quit" | "/exit" => std::process::exit(0),
            _ => println!(
                "Unknown command: {}. Type /help for available commands.",
                cmd
            ),
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                      REPL Commands                           ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  /help              Show this help message");
        println!("  /clear             Clear conversation history");
        println!("  /save              Save conversation to file");
        println!("  /load              Load conversation from file");
        println!("  /tokens            Show token usage statistics");
        println!("  /model <id>        Change model (e.g., /model claude-haiku-4-5-20251001)");
        println!("  /backend <type>    Switch backend (anthropic or bedrock)");
        println!("  /quit, /exit       Exit the REPL");
        println!();
    }

    fn clear_conversation(&mut self) {
        self.conversation.clear_messages();
        println!("✓ Conversation cleared");
    }

    async fn save_conversation(&self) -> Result<(), Box<dyn std::error::Error>> {
        let filename = format!(
            "conversation-{}.json",
            chrono::Utc::now().format("%Y%m%d-%H%M%S")
        );

        #[derive(Serialize)]
        struct SavedConversation {
            config: ReplConfig,
            messages: Vec<claude_sdk::Message>,
            tools: Vec<Tool>,
            system: Option<claude_sdk::types::SystemPrompt>,
        }

        let saved = SavedConversation {
            config: self.config.clone(),
            messages: self.conversation.messages().to_vec(),
            tools: self.conversation.tools().to_vec(),
            system: self.conversation.system().cloned(),
        };

        let json = serde_json::to_string_pretty(&saved)?;
        fs::write(&filename, json)?;

        println!("✓ Saved conversation to {}", filename);
        Ok(())
    }

    async fn load_conversation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Enter filename to load:");
        let filename = self.editor.readline("Filename> ")?;

        let json = fs::read_to_string(filename.trim())?;

        #[derive(Deserialize)]
        struct SavedConversation {
            config: ReplConfig,
            messages: Vec<claude_sdk::Message>,
            tools: Vec<Tool>,
            system: Option<claude_sdk::types::SystemPrompt>,
        }

        let saved: SavedConversation = serde_json::from_str(&json)?;

        // Rebuild conversation using public API
        let mut new_conversation = ConversationBuilder::new();

        // Re-add system prompt if it exists
        if let Some(claude_sdk::types::SystemPrompt::String(s)) = saved.system {
            new_conversation = new_conversation.with_system(s);
        }

        // Re-add tools
        for tool in saved.tools {
            new_conversation = new_conversation.with_tool(tool);
        }

        // Re-add messages
        for message in saved.messages {
            new_conversation.add_assistant_with_blocks(message.content);
        }

        self.conversation = new_conversation;
        self.config = saved.config;

        // Recreate client if backend changed
        self.client = Self::create_client(&self.config).await?;

        println!("✓ Loaded conversation");
        Ok(())
    }

    fn show_tokens(&self) {
        let total = self.conversation.estimate_tokens();
        let model = models::get_model(&self.config.model_id);

        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                   Token Statistics                           ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  Current conversation: ~{} tokens", total);
        println!("  Messages: {}", self.conversation.messages().len());
        println!("  Tools: {}", self.conversation.tools().len());

        if let Some(model_info) = model {
            println!();
            println!("  Model: {}", model_info.name);
            println!("  Context window: {} tokens", model_info.max_context_tokens);
            if let Some(extended) = model_info.max_context_tokens_extended {
                println!("  Extended context: {} tokens (with beta header)", extended);
            }

            let remaining =
                model_info.max_context_tokens as i64 - total as i64 - self.config.max_tokens as i64;
            println!(
                "  Remaining: {} tokens ({:.1}%)",
                remaining,
                (remaining as f64 / model_info.max_context_tokens as f64) * 100.0
            );

            // Estimate cost
            let estimated_cost = model_info.estimate_cost(total as u32, self.config.max_tokens);
            println!("  Estimated cost: ${:.6}", estimated_cost);
        }

        println!();
    }

    fn change_model(&mut self, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        if args.is_empty() {
            println!("Usage: /model <model-id>");
            println!();
            println!("Available models:");
            for model in models::ALL_MODELS {
                println!("  {} ({})", model.anthropic_id, model.name);
            }
            return Ok(());
        }

        self.config.model_id = args[0].to_string();
        println!("✓ Changed model to {}", self.config.model_id);
        Ok(())
    }

    async fn change_backend(&mut self, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        if args.is_empty() {
            println!("Usage: /backend <anthropic|bedrock> [region]");
            println!();
            println!("Current: {}", self.config.backend);
            return Ok(());
        }

        let backend = args[0];
        match backend {
            "anthropic" => {
                self.config.backend = "anthropic".to_string();
                self.config.region = None;
                self.client = Self::create_client(&self.config).await?;
                println!("✓ Switched to Anthropic API");
            }
            #[cfg(feature = "bedrock")]
            "bedrock" => {
                let region = args
                    .get(1)
                    .ok_or("Bedrock requires region (e.g., us-east-1)")?;
                self.config.backend = "bedrock".to_string();
                self.config.region = Some(region.to_string());
                self.client = Self::create_client(&self.config).await?;
                println!("✓ Switched to AWS Bedrock ({})", region);
            }
            #[cfg(not(feature = "bedrock"))]
            "bedrock" => {
                println!("❌ Bedrock support not compiled. Run with --features bedrock");
            }
            _ => println!("Unknown backend: {}. Use 'anthropic' or 'bedrock'", backend),
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    let mut repl = Repl::new().await?;
    repl.run().await?;

    Ok(())
}
