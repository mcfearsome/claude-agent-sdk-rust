//! Automated changelog generator using Claude
//!
//! This tool analyzes git commits and uses Claude to generate structured
//! changelog entries following the Keep a Changelog format.
//!
//! # Usage
//!
//! ```bash
//! export ANTHROPIC_API_KEY="your-api-key"
//! cargo run --bin update-changelog
//! ```
//!
//! This will:
//! 1. Get all commits since the last release tag
//! 2. Send them to Claude for analysis
//! 3. Generate changelog entries in Keep a Changelog format
//! 4. Display the generated entries (manual copy to CHANGELOG.md)

use claude_sdk::{ClaudeClient, Message, MessagesRequest, StreamEvent};
use futures::StreamExt;
use std::io::{self, Write};
use std::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Claude SDK Changelog Generator");
    println!("==================================\n");

    // Get API key
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    // Get git commits since last tag
    println!("📋 Fetching git commits since last release...");
    let commits = get_commits_since_last_tag()?;

    if commits.is_empty() {
        println!("No commits found since last release.");
        return Ok(());
    }

    println!("Found {} commits:\n", commits.len());
    for commit in commits.iter().take(5) {
        println!("  • {}", commit.split('\n').next().unwrap_or(commit));
    }
    if commits.len() > 5 {
        println!("  ... and {} more", commits.len() - 5);
    }
    println!();

    // Create Claude client
    let client = ClaudeClient::anthropic(api_key);

    // Build prompt for Claude
    let prompt = format!(
        r#"Analyze these git commits and generate changelog entries following the Keep a Changelog format.

Git commits:
{}

Generate changelog entries in this format:

### Added
- Feature descriptions for new functionality

### Changed
- Descriptions of changes to existing functionality

### Fixed
- Bug fix descriptions

### Breaking Changes
- Any breaking API changes

Rules:
1. Use user-facing language (not technical implementation details)
2. Group related commits together
3. Skip commits that are just formatting, typos, or trivial changes
4. Highlight breaking changes prominently
5. Be concise but descriptive
6. Use present tense ("Add X" not "Added X")

Generate only the changelog section, no preamble."#,
        commits.join("\n---\n")
    );

    println!("🤖 Asking Claude to generate changelog entries...\n");

    // Send request to Claude with streaming
    let request = MessagesRequest::new(
        claude_sdk::models::CLAUDE_SONNET_4_5.anthropic_id,
        2048,
        vec![Message::user(prompt)],
    );

    let mut stream = client.send_streaming(request).await?;

    println!("📄 Generated Changelog Entries:");
    println!("─────────────────────────────────────");

    let mut changelog_text = String::new();

    // Stream the response
    while let Some(event_result) = stream.next().await {
        match event_result? {
            StreamEvent::ContentBlockDelta { delta, .. } => {
                if let Some(text) = delta.text() {
                    print!("{}", text);
                    io::stdout().flush()?;
                    changelog_text.push_str(text);
                }
            }
            StreamEvent::MessageStop => break,
            StreamEvent::Error { error } => {
                eprintln!("\n❌ Error: {}", error.message);
                return Err(error.message.into());
            }
            _ => {}
        }
    }

    println!("\n─────────────────────────────────────\n");

    // Save to file for easy copying
    std::fs::write("CHANGELOG_GENERATED.md", &changelog_text)?;
    println!("💾 Saved to CHANGELOG_GENERATED.md");
    println!();
    println!("📋 Next steps:");
    println!("  1. Review the generated entries above");
    println!("  2. Copy relevant sections to CHANGELOG.md under [Unreleased]");
    println!("  3. Edit as needed for clarity and accuracy");
    println!("  4. Delete CHANGELOG_GENERATED.md when done");

    Ok(())
}

/// Get all commits since the last git tag
fn get_commits_since_last_tag() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Get the most recent tag
    let last_tag = Command::new("git")
        .args(["describe", "--tags", "--abbrev=0"])
        .output();

    let range = if let Ok(output) = last_tag {
        if output.status.success() {
            let tag = String::from_utf8(output.stdout)?.trim().to_string();
            println!("Last release: {}", tag);
            format!("{}..HEAD", tag)
        } else {
            // No tags yet, get all commits
            println!("No previous release found, getting all commits");
            "HEAD".to_string()
        }
    } else {
        "HEAD".to_string()
    };

    // Get commit log
    let output = Command::new("git")
        .args(["log", &range, "--pretty=format:%h - %s%n%b", "--no-merges"])
        .output()?;

    if !output.status.success() {
        return Err("Failed to get git log".into());
    }

    let log = String::from_utf8(output.stdout)?;

    // Split into individual commits
    let commits: Vec<String> = log
        .split("\n\n")
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    Ok(commits)
}
