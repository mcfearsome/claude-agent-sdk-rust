//! Model identifiers and metadata for Claude models
//!
//! This module provides constants and metadata for all available Claude models,
//! including constraints like context windows, output limits, and capabilities.
//!
//! # Example
//!
//! ```rust
//! use claude_sdk::models;
//!
//! let model = models::CLAUDE_SONNET_4_5;
//! println!("Using {} ({})", model.name, model.anthropic_id);
//! println!("Max output: {} tokens", model.max_output_tokens);
//! ```

/// Model capabilities and constraints
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    /// Human-readable model name
    pub name: &'static str,

    /// Model family (e.g., "sonnet", "opus", "haiku")
    pub family: &'static str,

    /// Release date/version identifier
    pub version: &'static str,

    /// Anthropic API model identifier
    pub anthropic_id: &'static str,

    /// AWS Bedrock regional endpoint model identifier (if available)
    pub bedrock_id: Option<&'static str>,

    /// AWS Bedrock global endpoint model identifier (if available)
    ///
    /// Claude 4.5+ models support global endpoints for dynamic routing.
    /// Use this for maximum availability across regions.
    pub bedrock_global_id: Option<&'static str>,

    /// Google Vertex AI model identifier (if available)
    pub vertex_id: Option<&'static str>,

    /// Maximum context window in tokens (standard)
    pub max_context_tokens: u32,

    /// Extended context window in tokens (if available)
    ///
    /// Some models support extended context with beta headers.
    /// For example, Claude Sonnet 4.5 and Sonnet 4 support 1M tokens
    /// with the `context-1m-2025-08-07` beta header.
    pub max_context_tokens_extended: Option<u32>,

    /// Maximum output tokens per request
    pub max_output_tokens: u32,

    /// Supports vision (image inputs)
    pub supports_vision: bool,

    /// Supports tool use
    pub supports_tools: bool,

    /// Supports prompt caching
    pub supports_caching: bool,

    /// Supports extended thinking
    pub supports_extended_thinking: bool,

    /// Supports effort parameter (beta)
    ///
    /// Requires beta header: `anthropic-beta: effort-2025-11-24`
    /// Currently only Claude Opus 4.5
    pub supports_effort: bool,

    /// Cost per million input tokens (USD)
    pub cost_per_mtok_input: f64,

    /// Cost per million output tokens (USD)
    pub cost_per_mtok_output: f64,

    /// Brief description of best use cases
    pub description: &'static str,
}

/// AWS Bedrock endpoint region
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BedrockRegion {
    /// Standard regional endpoint (tied to specific AWS region)
    Standard,
    /// Global endpoint (dynamic routing for maximum availability)
    Global,
    /// US-specific regional endpoint
    US,
    /// EU-specific regional endpoint
    EU,
    /// Asia-Pacific-specific regional endpoint
    AsiaPacific,
}

impl BedrockRegion {
    /// Get the prefix for this region
    pub fn prefix(&self) -> &'static str {
        match self {
            BedrockRegion::Standard => "",
            BedrockRegion::Global => "global.",
            BedrockRegion::US => "us.",
            BedrockRegion::EU => "eu.",
            BedrockRegion::AsiaPacific => "ap.",
        }
    }
}

impl Model {
    /// Get the model ID for the Anthropic API
    pub fn anthropic_id(&self) -> &'static str {
        self.anthropic_id
    }

    /// Get the model ID for AWS Bedrock regional endpoint (if available)
    pub fn bedrock_id(&self) -> Option<&'static str> {
        self.bedrock_id
    }

    /// Get the model ID for AWS Bedrock with a specific region prefix
    ///
    /// # Example
    ///
    /// ```rust
    /// use claude_sdk::models::{CLAUDE_SONNET_4_5, BedrockRegion};
    ///
    /// // Standard regional endpoint
    /// let regional = CLAUDE_SONNET_4_5.bedrock_id_for_region(BedrockRegion::Standard);
    /// // → Some("anthropic.claude-sonnet-4-5-20250929-v1:0")
    ///
    /// // Global endpoint
    /// let global = CLAUDE_SONNET_4_5.bedrock_id_for_region(BedrockRegion::Global);
    /// // → Some("global.anthropic.claude-sonnet-4-5-20250929-v1:0")
    ///
    /// // US regional endpoint
    /// let us = CLAUDE_SONNET_4_5.bedrock_id_for_region(BedrockRegion::US);
    /// // → Some("us.anthropic.claude-sonnet-4-5-20250929-v1:0")
    /// ```
    pub fn bedrock_id_for_region(&self, region: BedrockRegion) -> Option<String> {
        self.bedrock_id.map(|id| {
            let prefix = region.prefix();
            if prefix.is_empty() {
                id.to_string()
            } else {
                format!("{}{}", prefix, id)
            }
        })
    }

    /// Get the model ID for AWS Bedrock global endpoint (if available)
    ///
    /// Global endpoints provide dynamic routing for maximum availability.
    /// Available for Claude 4.5+ models.
    ///
    /// This is a convenience method equivalent to `bedrock_id_for_region(BedrockRegion::Global)`.
    pub fn bedrock_global_id(&self) -> Option<&'static str> {
        self.bedrock_global_id
    }

    /// Get the model ID for Google Vertex AI (if available)
    pub fn vertex_id(&self) -> Option<&'static str> {
        self.vertex_id
    }

    /// Check if this model supports extended context (e.g., 1M tokens)
    pub fn supports_extended_context(&self) -> bool {
        self.max_context_tokens_extended.is_some()
    }

    /// Get the extended context window size (if supported)
    ///
    /// Returns `Some(tokens)` if the model supports extended context with beta headers.
    /// For example, Claude Sonnet 4.5 returns `Some(1_000_000)`.
    ///
    /// # Beta Header Required
    ///
    /// To use extended context, include the beta header in your API request:
    /// - Header: `anthropic-beta: context-1m-2025-08-07`
    ///
    /// Note: Extended context may incur additional costs beyond 200K tokens.
    pub fn max_extended_context(&self) -> Option<u32> {
        self.max_context_tokens_extended
    }

    /// Validate that a request is compatible with this model's constraints
    ///
    /// # Parameters
    /// - `max_tokens`: The requested maximum output tokens
    /// - `use_extended_context`: Whether extended context will be used
    pub fn validate_request(
        &self,
        max_tokens: u32,
        use_extended_context: bool,
    ) -> Result<(), String> {
        if max_tokens > self.max_output_tokens {
            return Err(format!(
                "Requested max_tokens ({}) exceeds model limit ({})",
                max_tokens, self.max_output_tokens
            ));
        }

        if use_extended_context && !self.supports_extended_context() {
            return Err(format!(
                "Model {} does not support extended context",
                self.name
            ));
        }

        Ok(())
    }

    /// Estimate cost for a request
    pub fn estimate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        (input_tokens as f64 / 1_000_000.0) * self.cost_per_mtok_input
            + (output_tokens as f64 / 1_000_000.0) * self.cost_per_mtok_output
    }
}

//
// Latest Models (Claude 4.5)
//

/// Claude Sonnet 4.5 (2025-09-29)
///
/// Our smart model for complex agents and coding. Best balance of intelligence,
/// speed, and cost for most use cases.
///
/// Supports 1M context window with beta header `context-1m-2025-08-07`.
pub const CLAUDE_SONNET_4_5: Model = Model {
    name: "Claude Sonnet 4.5",
    family: "sonnet",
    version: "2025-09-29",
    anthropic_id: "claude-sonnet-4-5-20250929",
    bedrock_id: Some("anthropic.claude-sonnet-4-5-20250929-v1:0"),
    bedrock_global_id: Some("global.anthropic.claude-sonnet-4-5-20250929-v1:0"),
    vertex_id: Some("claude-sonnet-4-5@20250929"),
    max_context_tokens: 200_000,
    max_context_tokens_extended: Some(1_000_000),
    max_output_tokens: 64_000,
    supports_vision: true,
    supports_tools: true,
    supports_caching: true,
    supports_extended_thinking: true,
    supports_effort: false,
    cost_per_mtok_input: 3.0,
    cost_per_mtok_output: 15.0,
    description: "Smart model for complex agents and coding",
};

/// Claude Haiku 4.5 (2025-10-01)
///
/// Our fastest model with near-frontier intelligence.
pub const CLAUDE_HAIKU_4_5: Model = Model {
    name: "Claude Haiku 4.5",
    family: "haiku",
    version: "2025-10-01",
    anthropic_id: "claude-haiku-4-5-20251001",
    bedrock_id: Some("anthropic.claude-haiku-4-5-20251001-v1:0"),
    bedrock_global_id: Some("global.anthropic.claude-haiku-4-5-20251001-v1:0"),
    vertex_id: Some("claude-haiku-4-5@20251001"),
    max_context_tokens: 200_000,
    max_context_tokens_extended: None,
    max_output_tokens: 64_000,
    supports_vision: true,
    supports_tools: true,
    supports_caching: true,
    supports_extended_thinking: true,
    supports_effort: false,
    cost_per_mtok_input: 1.0,
    cost_per_mtok_output: 5.0,
    description: "Fastest model with near-frontier intelligence",
};

/// Claude Opus 4.5 (2025-11-01)
///
/// Premium model combining maximum intelligence with practical performance.
/// Supports effort parameter (beta: effort-2025-11-24).
pub const CLAUDE_OPUS_4_5: Model = Model {
    name: "Claude Opus 4.5",
    family: "opus",
    version: "2025-11-01",
    anthropic_id: "claude-opus-4-5-20251101",
    bedrock_id: Some("anthropic.claude-opus-4-5-20251101-v1:0"),
    bedrock_global_id: Some("global.anthropic.claude-opus-4-5-20251101-v1:0"),
    vertex_id: Some("claude-opus-4-5@20251101"),
    max_context_tokens: 200_000,
    max_context_tokens_extended: None,
    max_output_tokens: 64_000,
    supports_vision: true,
    supports_tools: true,
    supports_caching: true,
    supports_extended_thinking: true,
    supports_effort: true, // Only Opus 4.5 supports effort
    cost_per_mtok_input: 5.0,
    cost_per_mtok_output: 25.0,
    description: "Maximum intelligence with practical performance",
};

//
// Legacy Models (Claude 4.x and 3.x)
//

/// Claude Opus 4.1 (2025-08-05)
pub const CLAUDE_OPUS_4_1: Model = Model {
    name: "Claude Opus 4.1",
    family: "opus",
    version: "2025-08-05",
    anthropic_id: "claude-opus-4-1-20250805",
    bedrock_id: Some("anthropic.claude-opus-4-1-20250805-v1:0"),
    bedrock_global_id: None,
    vertex_id: Some("claude-opus-4-1@20250805"),
    max_context_tokens: 200_000,
    max_context_tokens_extended: None,
    max_output_tokens: 32_000,
    supports_vision: true,
    supports_tools: true,
    supports_caching: true,
    supports_extended_thinking: true,
    supports_effort: false,
    cost_per_mtok_input: 15.0,
    cost_per_mtok_output: 75.0,
    description: "Previous generation powerful model",
};

/// Claude Sonnet 4 (2025-05-14)
///
/// Supports 1M context window with beta header `context-1m-2025-08-07`.
pub const CLAUDE_SONNET_4: Model = Model {
    name: "Claude Sonnet 4",
    family: "sonnet",
    version: "2025-05-14",
    anthropic_id: "claude-sonnet-4-20250514",
    bedrock_id: Some("anthropic.claude-sonnet-4-20250514-v1:0"),
    bedrock_global_id: None,
    vertex_id: Some("claude-sonnet-4@20250514"),
    max_context_tokens: 200_000,
    max_context_tokens_extended: Some(1_000_000),
    max_output_tokens: 64_000,
    supports_vision: true,
    supports_tools: true,
    supports_caching: true,
    supports_extended_thinking: true,
    supports_effort: false,
    cost_per_mtok_input: 3.0,
    cost_per_mtok_output: 15.0,
    description: "Previous generation balanced model",
};

/// Claude Sonnet 3.7 (2025-02-19)
///
/// Supports 128K output with beta header `output-128k-2025-02-19`.
pub const CLAUDE_SONNET_3_7: Model = Model {
    name: "Claude Sonnet 3.7",
    family: "sonnet",
    version: "2025-02-19",
    anthropic_id: "claude-3-7-sonnet-20250219",
    bedrock_id: Some("anthropic.claude-3-7-sonnet-20250219-v1:0"),
    bedrock_global_id: None,
    vertex_id: Some("claude-3-7-sonnet@20250219"),
    max_context_tokens: 200_000,
    max_context_tokens_extended: None,
    max_output_tokens: 64_000, // 128K with beta header
    supports_vision: true,
    supports_tools: true,
    supports_caching: true,
    supports_extended_thinking: true,
    supports_effort: false,
    cost_per_mtok_input: 3.0,
    cost_per_mtok_output: 15.0,
    description: "Claude 3.7 balanced model",
};

/// Claude Opus 4 (2025-05-14)
pub const CLAUDE_OPUS_4: Model = Model {
    name: "Claude Opus 4",
    family: "opus",
    version: "2025-05-14",
    anthropic_id: "claude-opus-4-20250514",
    bedrock_id: Some("anthropic.claude-opus-4-20250514-v1:0"),
    bedrock_global_id: None,
    vertex_id: Some("claude-opus-4@20250514"),
    max_context_tokens: 200_000,
    max_context_tokens_extended: None,
    max_output_tokens: 32_000,
    supports_vision: true,
    supports_tools: true,
    supports_caching: true,
    supports_extended_thinking: true,
    supports_effort: false,
    cost_per_mtok_input: 15.0,
    cost_per_mtok_output: 75.0,
    description: "Claude 4 powerful model",
};

/// Claude Haiku 3.5 (2024-10-22)
pub const CLAUDE_HAIKU_3_5: Model = Model {
    name: "Claude Haiku 3.5",
    family: "haiku",
    version: "2024-10-22",
    anthropic_id: "claude-3-5-haiku-20241022",
    bedrock_id: Some("anthropic.claude-3-5-haiku-20241022-v1:0"),
    bedrock_global_id: None,
    vertex_id: Some("claude-3-5-haiku@20241022"),
    max_context_tokens: 200_000,
    max_context_tokens_extended: None,
    max_output_tokens: 8_192,
    supports_vision: true,
    supports_tools: true,
    supports_caching: true,
    supports_extended_thinking: false,
    supports_effort: false,
    cost_per_mtok_input: 0.80,
    cost_per_mtok_output: 4.0,
    description: "Fast and efficient model",
};

/// Claude Haiku 3 (2024-03-07)
pub const CLAUDE_HAIKU_3: Model = Model {
    name: "Claude Haiku 3",
    family: "haiku",
    version: "2024-03-07",
    anthropic_id: "claude-3-haiku-20240307",
    bedrock_id: Some("anthropic.claude-3-haiku-20240307-v1:0"),
    bedrock_global_id: None,
    vertex_id: Some("claude-3-haiku@20240307"),
    max_context_tokens: 200_000,
    max_context_tokens_extended: None,
    max_output_tokens: 4_096,
    supports_vision: true,
    supports_tools: true,
    supports_caching: true,
    supports_extended_thinking: false,
    supports_effort: false,
    cost_per_mtok_input: 0.25,
    cost_per_mtok_output: 1.25,
    description: "Original fast model",
};

/// List of all available models (latest first)
pub const ALL_MODELS: &[&Model] = &[
    // Latest (Claude 4.5)
    &CLAUDE_SONNET_4_5,
    &CLAUDE_HAIKU_4_5,
    &CLAUDE_OPUS_4_5,
    // Legacy (Claude 4.x and 3.x)
    &CLAUDE_OPUS_4_1,
    &CLAUDE_SONNET_4,
    &CLAUDE_SONNET_3_7,
    &CLAUDE_OPUS_4,
    &CLAUDE_HAIKU_3_5,
    &CLAUDE_HAIKU_3,
];

/// Lookup a model by its Anthropic API ID
pub fn get_model_by_anthropic_id(id: &str) -> Option<&'static Model> {
    ALL_MODELS.iter().find(|m| m.anthropic_id == id).copied()
}

/// Lookup a model by its Bedrock ID (any region prefix)
///
/// Supports all Bedrock endpoint types:
/// - Standard regional: `anthropic.claude-sonnet-4-5-20250929-v1:0`
/// - Global: `global.anthropic.claude-sonnet-4-5-20250929-v1:0`
/// - US regional: `us.anthropic.claude-sonnet-4-5-20250929-v1:0`
/// - EU regional: `eu.anthropic.claude-sonnet-4-5-20250929-v1:0`
/// - AP regional: `ap.anthropic.claude-sonnet-4-5-20250929-v1:0`
pub fn get_model_by_bedrock_id(id: &str) -> Option<&'static Model> {
    // Try exact match first
    if let Some(model) = ALL_MODELS
        .iter()
        .find(|m| m.bedrock_id == Some(id) || m.bedrock_global_id == Some(id))
    {
        return Some(*model);
    }

    // Try stripping regional prefixes and matching base ID
    let base_id = id
        .strip_prefix("global.")
        .or_else(|| id.strip_prefix("us."))
        .or_else(|| id.strip_prefix("eu."))
        .or_else(|| id.strip_prefix("ap."))
        .unwrap_or(id);

    ALL_MODELS
        .iter()
        .find(|m| m.bedrock_id == Some(base_id))
        .copied()
}

/// Lookup a model by its Vertex AI ID
pub fn get_model_by_vertex_id(id: &str) -> Option<&'static Model> {
    ALL_MODELS.iter().find(|m| m.vertex_id == Some(id)).copied()
}

/// Lookup a model by any ID (tries Anthropic, Bedrock, and Vertex)
pub fn get_model(id: &str) -> Option<&'static Model> {
    get_model_by_anthropic_id(id)
        .or_else(|| get_model_by_bedrock_id(id))
        .or_else(|| get_model_by_vertex_id(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_constants() {
        assert_eq!(CLAUDE_SONNET_4_5.anthropic_id, "claude-sonnet-4-5-20250929");
        assert_eq!(
            CLAUDE_SONNET_4_5.bedrock_id,
            Some("anthropic.claude-sonnet-4-5-20250929-v1:0")
        );
        assert_eq!(
            CLAUDE_SONNET_4_5.bedrock_global_id,
            Some("global.anthropic.claude-sonnet-4-5-20250929-v1:0")
        );
        assert_eq!(
            CLAUDE_SONNET_4_5.max_context_tokens_extended,
            Some(1_000_000)
        );
    }

    #[test]
    fn test_model_lookup_anthropic() {
        let model = get_model_by_anthropic_id("claude-sonnet-4-5-20250929");
        assert!(model.is_some());
        assert_eq!(model.unwrap().name, "Claude Sonnet 4.5");
    }

    #[test]
    fn test_model_lookup_bedrock_regional() {
        let model = get_model_by_bedrock_id("anthropic.claude-sonnet-4-5-20250929-v1:0");
        assert!(model.is_some());
        assert_eq!(model.unwrap().name, "Claude Sonnet 4.5");
    }

    #[test]
    fn test_model_lookup_bedrock_global() {
        let model = get_model_by_bedrock_id("global.anthropic.claude-sonnet-4-5-20250929-v1:0");
        assert!(model.is_some());
        assert_eq!(model.unwrap().name, "Claude Sonnet 4.5");
    }

    #[test]
    fn test_model_lookup_bedrock_us() {
        let model = get_model_by_bedrock_id("us.anthropic.claude-sonnet-4-5-20250929-v1:0");
        assert!(model.is_some());
        assert_eq!(model.unwrap().name, "Claude Sonnet 4.5");
    }

    #[test]
    fn test_model_lookup_bedrock_eu() {
        let model = get_model_by_bedrock_id("eu.anthropic.claude-sonnet-4-5-20250929-v1:0");
        assert!(model.is_some());
        assert_eq!(model.unwrap().name, "Claude Sonnet 4.5");
    }

    #[test]
    fn test_model_lookup_bedrock_ap() {
        let model = get_model_by_bedrock_id("ap.anthropic.claude-sonnet-4-5-20250929-v1:0");
        assert!(model.is_some());
        assert_eq!(model.unwrap().name, "Claude Sonnet 4.5");
    }

    #[test]
    fn test_bedrock_id_for_region() {
        // Standard regional endpoint
        let regional = CLAUDE_SONNET_4_5.bedrock_id_for_region(BedrockRegion::Standard);
        assert_eq!(
            regional.as_deref(),
            Some("anthropic.claude-sonnet-4-5-20250929-v1:0")
        );

        // Global endpoint
        let global = CLAUDE_SONNET_4_5.bedrock_id_for_region(BedrockRegion::Global);
        assert_eq!(
            global.as_deref(),
            Some("global.anthropic.claude-sonnet-4-5-20250929-v1:0")
        );

        // US regional endpoint
        let us = CLAUDE_SONNET_4_5.bedrock_id_for_region(BedrockRegion::US);
        assert_eq!(
            us.as_deref(),
            Some("us.anthropic.claude-sonnet-4-5-20250929-v1:0")
        );

        // EU regional endpoint
        let eu = CLAUDE_SONNET_4_5.bedrock_id_for_region(BedrockRegion::EU);
        assert_eq!(
            eu.as_deref(),
            Some("eu.anthropic.claude-sonnet-4-5-20250929-v1:0")
        );

        // AP regional endpoint
        let ap = CLAUDE_SONNET_4_5.bedrock_id_for_region(BedrockRegion::AsiaPacific);
        assert_eq!(
            ap.as_deref(),
            Some("ap.anthropic.claude-sonnet-4-5-20250929-v1:0")
        );
    }

    #[test]
    fn test_model_lookup_any() {
        // Should work with Anthropic, Bedrock regional, and all Bedrock prefixes
        assert!(get_model("claude-sonnet-4-5-20250929").is_some());
        assert!(get_model("anthropic.claude-sonnet-4-5-20250929-v1:0").is_some());
        assert!(get_model("global.anthropic.claude-sonnet-4-5-20250929-v1:0").is_some());
        assert!(get_model("us.anthropic.claude-sonnet-4-5-20250929-v1:0").is_some());
        assert!(get_model("eu.anthropic.claude-sonnet-4-5-20250929-v1:0").is_some());
        assert!(get_model("ap.anthropic.claude-sonnet-4-5-20250929-v1:0").is_some());
    }

    #[test]
    fn test_validate_request() {
        assert!(CLAUDE_SONNET_4_5.validate_request(1024, false).is_ok());
        assert!(CLAUDE_SONNET_4_5.validate_request(64_000, false).is_ok());
        assert!(CLAUDE_SONNET_4_5.validate_request(100_000, false).is_err());

        // Test extended context validation
        assert!(CLAUDE_SONNET_4_5.validate_request(1024, true).is_ok());
        assert!(CLAUDE_HAIKU_4_5.validate_request(1024, true).is_err()); // Doesn't support 1M
    }

    #[test]
    fn test_extended_context_support() {
        // Models that support 1M context
        assert!(CLAUDE_SONNET_4_5.supports_extended_context());
        assert_eq!(CLAUDE_SONNET_4_5.max_extended_context(), Some(1_000_000));
        assert!(CLAUDE_SONNET_4.supports_extended_context());

        // Models that don't support 1M context
        assert!(!CLAUDE_HAIKU_4_5.supports_extended_context());
        assert_eq!(CLAUDE_HAIKU_4_5.max_extended_context(), None);
        assert!(!CLAUDE_OPUS_4_5.supports_extended_context());
    }

    #[test]
    fn test_bedrock_global_endpoints() {
        // Claude 4.5 models support global endpoints
        assert!(CLAUDE_SONNET_4_5.bedrock_global_id().is_some());
        assert!(CLAUDE_HAIKU_4_5.bedrock_global_id().is_some());
        assert!(CLAUDE_OPUS_4_5.bedrock_global_id().is_some());

        // Legacy models don't support global endpoints
        assert!(CLAUDE_SONNET_4.bedrock_global_id().is_none());
        assert!(CLAUDE_HAIKU_3_5.bedrock_global_id().is_none());
    }

    #[test]
    fn test_estimate_cost() {
        let cost = CLAUDE_SONNET_4_5.estimate_cost(1000, 500);
        // $3/MTok input + $15/MTok output
        // = (1000/1M * 3) + (500/1M * 15)
        // = 0.003 + 0.0075 = 0.0105
        assert!((cost - 0.0105).abs() < 0.0001);
    }

    #[test]
    fn test_all_models_have_unique_ids() {
        let mut ids = std::collections::HashSet::new();
        for model in ALL_MODELS {
            assert!(ids.insert(model.anthropic_id));
        }
    }
}
