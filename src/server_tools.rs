//! Server-side tool definitions for Claude API built-in tools.
//!
//! These tools are executed by Claude's infrastructure, not by your code.
//! Add them to `MessagesRequest.tools` to enable server-side capabilities.

use crate::types::CacheControl;
use serde::{Deserialize, Serialize};

/// Web search tool — lets Claude search the internet during a response.
///
/// # Example
///
/// ```rust
/// use claude_sdk::server_tools::WebSearchTool;
///
/// let tool = WebSearchTool::new()
///     .with_allowed_domains(vec!["example.com".into()])
///     .with_max_uses(5);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchTool {
    /// Always `"web_search_20260209"` (or overridden via constructor)
    #[serde(rename = "type")]
    pub tool_type: String,

    /// Tool name (always `"web_search"`)
    pub name: String,

    /// Restrict searches to these domains only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,

    /// Never search these domains
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_domains: Option<Vec<String>>,

    /// Maximum number of searches per request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,

    /// User's location for localizing search results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<UserLocation>,

    /// Cache control for this tool definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl WebSearchTool {
    /// Create a new WebSearchTool with the latest tool version.
    pub fn new() -> Self {
        Self {
            tool_type: "web_search_20260209".into(),
            name: "web_search".into(),
            allowed_domains: None,
            blocked_domains: None,
            max_uses: None,
            user_location: None,
            cache_control: None,
        }
    }

    /// Restrict searches to the specified domains.
    pub fn with_allowed_domains(mut self, domains: Vec<String>) -> Self {
        self.allowed_domains = Some(domains);
        self
    }

    /// Block searches from the specified domains.
    pub fn with_blocked_domains(mut self, domains: Vec<String>) -> Self {
        self.blocked_domains = Some(domains);
        self
    }

    /// Set the maximum number of web searches allowed per request.
    pub fn with_max_uses(mut self, max: u32) -> Self {
        self.max_uses = Some(max);
        self
    }

    /// Set the user's location for localizing search results.
    pub fn with_user_location(mut self, location: UserLocation) -> Self {
        self.user_location = Some(location);
        self
    }

    /// Set cache control for this tool definition.
    pub fn with_cache_control(mut self, cache_control: CacheControl) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

/// User location for localizing web search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLocation {
    /// Always `"approximate"`
    #[serde(rename = "type")]
    pub location_type: String,

    /// City name (e.g., `"San Francisco"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    /// ISO 3166-1 alpha-2 country code (e.g., `"US"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// Region or state (e.g., `"California"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// IANA timezone (e.g., `"America/Los_Angeles"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

impl UserLocation {
    /// Create a new approximate UserLocation.
    pub fn new() -> Self {
        Self {
            location_type: "approximate".into(),
            city: None,
            country: None,
            region: None,
            timezone: None,
        }
    }
}

impl Default for UserLocation {
    fn default() -> Self {
        Self::new()
    }
}

/// Web fetch tool — lets Claude retrieve content from a specific URL.
///
/// # Example
///
/// ```rust
/// use claude_sdk::server_tools::WebFetchTool;
///
/// let tool = WebFetchTool::new();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchTool {
    /// Always `"web_fetch_20260309"`
    #[serde(rename = "type")]
    pub tool_type: String,

    /// Tool name (always `"web_fetch"`)
    pub name: String,

    /// Restrict fetches to these domains only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,

    /// Never fetch from these domains
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_domains: Option<Vec<String>>,

    /// Maximum number of fetch operations per request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,

    /// Maximum number of tokens to return from fetched content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_content_tokens: Option<u32>,

    /// Cache control for this tool definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl WebFetchTool {
    /// Create a new WebFetchTool with the latest tool version.
    pub fn new() -> Self {
        Self {
            tool_type: "web_fetch_20260309".into(),
            name: "web_fetch".into(),
            allowed_domains: None,
            blocked_domains: None,
            max_uses: None,
            max_content_tokens: None,
            cache_control: None,
        }
    }
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Code execution tool — lets Claude execute code in a sandboxed environment.
///
/// # Example
///
/// ```rust
/// use claude_sdk::server_tools::CodeExecutionTool;
///
/// let tool = CodeExecutionTool::new();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionTool {
    /// Always `"code_execution_20260120"`
    #[serde(rename = "type")]
    pub tool_type: String,

    /// Tool name (always `"code_execution"`)
    pub name: String,

    /// Cache control for this tool definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl CodeExecutionTool {
    /// Create a new CodeExecutionTool with the latest tool version.
    pub fn new() -> Self {
        Self {
            tool_type: "code_execution_20260120".into(),
            name: "code_execution".into(),
            cache_control: None,
        }
    }
}

impl Default for CodeExecutionTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Bash tool — lets Claude execute bash commands in a sandboxed environment.
///
/// # Example
///
/// ```rust
/// use claude_sdk::server_tools::BashTool;
///
/// let tool = BashTool::new();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashTool {
    /// Always `"bash_20250124"`
    #[serde(rename = "type")]
    pub tool_type: String,

    /// Tool name (always `"bash"`)
    pub name: String,

    /// Cache control for this tool definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl BashTool {
    /// Create a new BashTool with the latest tool version.
    pub fn new() -> Self {
        Self {
            tool_type: "bash_20250124".into(),
            name: "bash".into(),
            cache_control: None,
        }
    }
}

impl Default for BashTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Text editor tool — lets Claude read and edit files using structured operations.
///
/// # Example
///
/// ```rust
/// use claude_sdk::server_tools::TextEditorTool;
///
/// let tool = TextEditorTool::new();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorTool {
    /// Always `"text_editor_20250728"`
    #[serde(rename = "type")]
    pub tool_type: String,

    /// Tool name (always `"str_replace_editor"`)
    pub name: String,

    /// Maximum number of characters to return when viewing file contents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_characters: Option<u32>,

    /// Cache control for this tool definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl TextEditorTool {
    /// Create a new TextEditorTool with the latest tool version.
    pub fn new() -> Self {
        Self {
            tool_type: "text_editor_20250728".into(),
            name: "str_replace_editor".into(),
            max_characters: None,
            cache_control: None,
        }
    }
}

impl Default for TextEditorTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CacheControl;

    // --- WebSearchTool ---

    #[test]
    fn test_web_search_tool_new_type_and_name() {
        let tool = WebSearchTool::new();
        assert_eq!(tool.tool_type, "web_search_20260209");
        assert_eq!(tool.name, "web_search");
    }

    #[test]
    fn test_web_search_tool_serialization() {
        let tool = WebSearchTool::new();
        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["type"], "web_search_20260209");
        assert_eq!(json["name"], "web_search");
        assert!(json.get("allowed_domains").is_none());
        assert!(json.get("blocked_domains").is_none());
        assert!(json.get("max_uses").is_none());
        assert!(json.get("user_location").is_none());
        assert!(json.get("cache_control").is_none());
    }

    #[test]
    fn test_web_search_tool_builder_allowed_domains() {
        let tool =
            WebSearchTool::new().with_allowed_domains(vec!["example.com".into(), "docs.rs".into()]);
        let json = serde_json::to_value(&tool).unwrap();
        let domains = json["allowed_domains"].as_array().unwrap();
        assert_eq!(domains.len(), 2);
        assert_eq!(domains[0], "example.com");
        assert_eq!(domains[1], "docs.rs");
    }

    #[test]
    fn test_web_search_tool_builder_blocked_domains() {
        let tool = WebSearchTool::new().with_blocked_domains(vec!["spam.example.com".into()]);
        let json = serde_json::to_value(&tool).unwrap();
        let domains = json["blocked_domains"].as_array().unwrap();
        assert_eq!(domains[0], "spam.example.com");
    }

    #[test]
    fn test_web_search_tool_builder_max_uses() {
        let tool = WebSearchTool::new().with_max_uses(10);
        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["max_uses"], 10);
    }

    #[test]
    fn test_web_search_tool_builder_user_location() {
        let location = UserLocation {
            location_type: "approximate".into(),
            city: Some("San Francisco".into()),
            country: Some("US".into()),
            region: Some("California".into()),
            timezone: Some("America/Los_Angeles".into()),
        };
        let tool = WebSearchTool::new().with_user_location(location);
        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["user_location"]["type"], "approximate");
        assert_eq!(json["user_location"]["city"], "San Francisco");
        assert_eq!(json["user_location"]["country"], "US");
        assert_eq!(json["user_location"]["region"], "California");
        assert_eq!(json["user_location"]["timezone"], "America/Los_Angeles");
    }

    #[test]
    fn test_web_search_tool_builder_cache_control() {
        let tool = WebSearchTool::new().with_cache_control(CacheControl::ephemeral());
        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn test_web_search_tool_roundtrip() {
        let tool = WebSearchTool::new()
            .with_allowed_domains(vec!["example.com".into()])
            .with_max_uses(5);
        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: WebSearchTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tool_type, "web_search_20260209");
        assert_eq!(deserialized.name, "web_search");
        assert_eq!(deserialized.max_uses, Some(5));
        let domains = deserialized.allowed_domains.unwrap();
        assert_eq!(domains[0], "example.com");
    }

    // --- UserLocation ---

    #[test]
    fn test_user_location_new() {
        let loc = UserLocation::new();
        assert_eq!(loc.location_type, "approximate");
        assert!(loc.city.is_none());
        assert!(loc.country.is_none());
        assert!(loc.region.is_none());
        assert!(loc.timezone.is_none());
    }

    #[test]
    fn test_user_location_serialization_skips_none() {
        let loc = UserLocation::new();
        let json = serde_json::to_value(&loc).unwrap();
        assert_eq!(json["type"], "approximate");
        assert!(json.get("city").is_none());
    }

    // --- WebFetchTool ---

    #[test]
    fn test_web_fetch_tool_new_type_and_name() {
        let tool = WebFetchTool::new();
        assert_eq!(tool.tool_type, "web_fetch_20260309");
        assert_eq!(tool.name, "web_fetch");
    }

    #[test]
    fn test_web_fetch_tool_serialization() {
        let tool = WebFetchTool::new();
        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["type"], "web_fetch_20260309");
        assert_eq!(json["name"], "web_fetch");
        assert!(json.get("max_uses").is_none());
        assert!(json.get("max_content_tokens").is_none());
    }

    #[test]
    fn test_web_fetch_tool_roundtrip() {
        let tool = WebFetchTool::new();
        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: WebFetchTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tool_type, "web_fetch_20260309");
        assert_eq!(deserialized.name, "web_fetch");
    }

    // --- CodeExecutionTool ---

    #[test]
    fn test_code_execution_tool_new_type_and_name() {
        let tool = CodeExecutionTool::new();
        assert_eq!(tool.tool_type, "code_execution_20260120");
        assert_eq!(tool.name, "code_execution");
    }

    #[test]
    fn test_code_execution_tool_serialization() {
        let tool = CodeExecutionTool::new();
        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["type"], "code_execution_20260120");
        assert_eq!(json["name"], "code_execution");
        assert!(json.get("cache_control").is_none());
    }

    #[test]
    fn test_code_execution_tool_roundtrip() {
        let tool = CodeExecutionTool::new();
        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: CodeExecutionTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tool_type, "code_execution_20260120");
        assert_eq!(deserialized.name, "code_execution");
    }

    // --- BashTool ---

    #[test]
    fn test_bash_tool_new_type_and_name() {
        let tool = BashTool::new();
        assert_eq!(tool.tool_type, "bash_20250124");
        assert_eq!(tool.name, "bash");
    }

    #[test]
    fn test_bash_tool_serialization() {
        let tool = BashTool::new();
        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["type"], "bash_20250124");
        assert_eq!(json["name"], "bash");
        assert!(json.get("cache_control").is_none());
    }

    #[test]
    fn test_bash_tool_roundtrip() {
        let tool = BashTool::new();
        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: BashTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tool_type, "bash_20250124");
        assert_eq!(deserialized.name, "bash");
    }

    // --- TextEditorTool ---

    #[test]
    fn test_text_editor_tool_new_type_and_name() {
        let tool = TextEditorTool::new();
        assert_eq!(tool.tool_type, "text_editor_20250728");
        assert_eq!(tool.name, "str_replace_editor");
    }

    #[test]
    fn test_text_editor_tool_serialization() {
        let tool = TextEditorTool::new();
        let json = serde_json::to_value(&tool).unwrap();
        assert_eq!(json["type"], "text_editor_20250728");
        assert_eq!(json["name"], "str_replace_editor");
        assert!(json.get("max_characters").is_none());
        assert!(json.get("cache_control").is_none());
    }

    #[test]
    fn test_text_editor_tool_roundtrip() {
        let tool = TextEditorTool::new();
        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: TextEditorTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tool_type, "text_editor_20250728");
        assert_eq!(deserialized.name, "str_replace_editor");
    }
}
