use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub filters: FilterConfig,
    pub classifier: ClassifierConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub proxy_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub enabled: bool,
    pub rules: HashMap<String, FilterRule>,
    pub notification_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    pub enabled: bool,
    pub score_threshold: f32,
    pub action: FilterAction,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterAction {
    Block,
    Throttle(u64), // seconds to delay
    Warning,
    Allow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifierConfig {
    pub enabled: bool,
    pub model_type: ModelType,
    pub api_key: Option<String>,
    pub api_endpoint: Option<String>,
    pub confidence_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    OpenAI,
    Local,
    Mock, // For testing
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub enabled: bool,
    pub theme: String,
    pub show_blocked_count: bool,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        if std::path::Path::new(path).exists() {
            let content = std::fs::read_to_string(path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config
            let default_config = Self::default();
            let content = toml::to_string_pretty(&default_config)?;
            std::fs::write(path, content)?;
            Ok(default_config)
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut default_rules = HashMap::new();
        
        // Default rule for low-value content
        default_rules.insert("brainrot".to_string(), FilterRule {
            enabled: true,
            score_threshold: 0.7,
            action: FilterAction::Block,
            categories: vec!["clickbait".to_string(), "mindless_scrolling".to_string()],
        });
        
        // Default rule for educational content
        default_rules.insert("educational".to_string(), FilterRule {
            enabled: true,
            score_threshold: 0.6,
            action: FilterAction::Allow,
            categories: vec!["learning".to_string(), "tutorial".to_string(), "informative".to_string()],
        });
        
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                proxy_timeout: 30,
            },
            filters: FilterConfig {
                enabled: true,
                rules: default_rules,
                notification_threshold: 0.8,
            },
            classifier: ClassifierConfig {
                enabled: true,
                model_type: ModelType::Mock, // Default to mock for testing
                api_key: None,
                api_endpoint: None,
                confidence_threshold: 0.6,
            },
            ui: UiConfig {
                enabled: true,
                theme: "dark".to_string(),
                show_blocked_count: true,
            },
        }
    }
}