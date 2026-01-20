//! Model definitions and presets
//!
//! Contains model configurations and recommended settings.

use serde::{Deserialize, Serialize};

/// Model preset with recommended settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreset {
    /// Model identifier
    pub name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Description of the model
    pub description: String,
    /// Recommended use case
    pub use_case: ModelUseCase,
    /// Parameter count (for display)
    pub parameters: String,
    /// Recommended temperature
    pub default_temperature: f32,
    /// Whether this model supports function calling
    pub supports_tools: bool,
}

/// Intended use case for a model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelUseCase {
    /// Orchestration and function calling
    Orchestrator,
    /// Code generation and explanation
    Coding,
    /// General conversation
    General,
    /// Both orchestration and coding
    Hybrid,
}

/// Get predefined model presets
pub fn get_model_presets() -> Vec<ModelPreset> {
    vec![
        // Orchestrator models
        ModelPreset {
            name: "functiongemma".to_string(),
            display_name: "FunctionGemma".to_string(),
            description: "Specialized for function calling and tool routing".to_string(),
            use_case: ModelUseCase::Orchestrator,
            parameters: "2B".to_string(),
            default_temperature: 0.1,
            supports_tools: true,
        },
        ModelPreset {
            name: "qwen2.5-coder:7b".to_string(),
            display_name: "Qwen 2.5 Coder 7B".to_string(),
            description: "Excellent code generation with good function calling".to_string(),
            use_case: ModelUseCase::Hybrid,
            parameters: "7B".to_string(),
            default_temperature: 0.3,
            supports_tools: true,
        },
        // Executor models
        ModelPreset {
            name: "gemma3:4b".to_string(),
            display_name: "Gemma 3 4B".to_string(),
            description: "Fast, efficient code generation".to_string(),
            use_case: ModelUseCase::Coding,
            parameters: "4B".to_string(),
            default_temperature: 0.7,
            supports_tools: false,
        },
        ModelPreset {
            name: "gemma3:12b".to_string(),
            display_name: "Gemma 3 12B".to_string(),
            description: "Higher quality code generation".to_string(),
            use_case: ModelUseCase::Coding,
            parameters: "12B".to_string(),
            default_temperature: 0.7,
            supports_tools: false,
        },
        ModelPreset {
            name: "codellama:7b".to_string(),
            display_name: "Code Llama 7B".to_string(),
            description: "Meta's code-specialized model".to_string(),
            use_case: ModelUseCase::Coding,
            parameters: "7B".to_string(),
            default_temperature: 0.7,
            supports_tools: false,
        },
        ModelPreset {
            name: "deepseek-coder:6.7b".to_string(),
            display_name: "DeepSeek Coder 6.7B".to_string(),
            description: "Strong code completion and generation".to_string(),
            use_case: ModelUseCase::Coding,
            parameters: "6.7B".to_string(),
            default_temperature: 0.5,
            supports_tools: false,
        },
        ModelPreset {
            name: "mistral:7b".to_string(),
            display_name: "Mistral 7B".to_string(),
            description: "General purpose with decent function calling".to_string(),
            use_case: ModelUseCase::General,
            parameters: "7B".to_string(),
            default_temperature: 0.7,
            supports_tools: true,
        },
    ]
}

/// Find a model preset by name
pub fn find_preset(name: &str) -> Option<ModelPreset> {
    get_model_presets().into_iter().find(|p| p.name == name)
}

/// Get recommended orchestrator models
pub fn recommended_orchestrators() -> Vec<ModelPreset> {
    get_model_presets()
        .into_iter()
        .filter(|p| {
            p.supports_tools
                && (p.use_case == ModelUseCase::Orchestrator || p.use_case == ModelUseCase::Hybrid)
        })
        .collect()
}

/// Get recommended executor models
pub fn recommended_executors() -> Vec<ModelPreset> {
    get_model_presets()
        .into_iter()
        .filter(|p| p.use_case == ModelUseCase::Coding || p.use_case == ModelUseCase::Hybrid)
        .collect()
}
