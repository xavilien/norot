use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::config::{ClassifierConfig, ModelType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub category: String,
    pub confidence: f32,
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentData {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub text_content: Option<String>,
    pub metadata: HashMap<String, String>,
}

pub struct ContentClassifier {
    config: ClassifierConfig,
    client: reqwest::Client,
}

impl ContentClassifier {
    pub fn new(config: ClassifierConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
    
    pub async fn classify_content(&self, content: &ContentData) -> Result<ClassificationResult> {
        if !self.config.enabled {
            return Ok(ClassificationResult {
                category: "unclassified".to_string(),
                confidence: 0.0,
                reasoning: Some("Classification disabled".to_string()),
            });
        }
        
        match self.config.model_type {
            ModelType::Mock => self.mock_classify(content).await,
            ModelType::OpenAI => self.openai_classify(content).await,
            ModelType::Local => self.local_classify(content).await,
        }
    }
    
    async fn mock_classify(&self, content: &ContentData) -> Result<ClassificationResult> {
        // Simple mock classification based on URL and title patterns
        let text = format!(
            "{} {} {}",
            content.url,
            content.title.as_deref().unwrap_or(""),
            content.description.as_deref().unwrap_or("")
        ).to_lowercase();
        
        // Educational content patterns
        if text.contains("tutorial") || text.contains("learn") || text.contains("education") 
           || text.contains("guide") || text.contains("how-to") || text.contains("course") {
            return Ok(ClassificationResult {
                category: "educational".to_string(),
                confidence: 0.85,
                reasoning: Some("Contains educational keywords".to_string()),
            });
        }
        
        // News and informative content
        if text.contains("news") || text.contains("analysis") || text.contains("research") 
           || text.contains("science") || text.contains("technology") {
            return Ok(ClassificationResult {
                category: "informative".to_string(),
                confidence: 0.80,
                reasoning: Some("Contains informative keywords".to_string()),
            });
        }
        
        // Clickbait and low-value content patterns
        if text.contains("you won't believe") || text.contains("shocking") || text.contains("viral")
           || text.contains("click here") || text.contains("amazing trick") || text.contains("hate him")
           || text.contains("doctors don't want") || text.contains("this one weird") {
            return Ok(ClassificationResult {
                category: "clickbait".to_string(),
                confidence: 0.90,
                reasoning: Some("Contains clickbait keywords".to_string()),
            });
        }
        
        // Entertainment content
        if text.contains("meme") || text.contains("funny") || text.contains("cute") 
           || text.contains("cat") || text.contains("dog") || text.contains("entertainment") {
            return Ok(ClassificationResult {
                category: "entertainment".to_string(),
                confidence: 0.75,
                reasoning: Some("Contains entertainment keywords".to_string()),
            });
        }
        
        // Social media patterns that often lead to mindless scrolling
        if content.url.contains("instagram.com") || content.url.contains("tiktok.com") 
           || content.url.contains("twitter.com") || content.url.contains("facebook.com") {
            if text.contains("story") || text.contains("reel") || text.contains("post") {
                return Ok(ClassificationResult {
                    category: "mindless_scrolling".to_string(),
                    confidence: 0.70,
                    reasoning: Some("Social media content likely to encourage mindless scrolling".to_string()),
                });
            }
        }
        
        // Default to neutral
        Ok(ClassificationResult {
            category: "neutral".to_string(),
            confidence: 0.50,
            reasoning: Some("No clear classification patterns found".to_string()),
        })
    }
    
    async fn openai_classify(&self, content: &ContentData) -> Result<ClassificationResult> {
        // OpenAI API integration would go here
        // For now, fallback to mock
        self.mock_classify(content).await
    }
    
    async fn local_classify(&self, content: &ContentData) -> Result<ClassificationResult> {
        // Local model integration would go here
        // For now, fallback to mock
        self.mock_classify(content).await
    }
    
    pub fn get_brainrot_score(&self, classification: &ClassificationResult) -> f32 {
        match classification.category.as_str() {
            "clickbait" => 0.9,
            "mindless_scrolling" => 0.8,
            "entertainment" => 0.4,
            "neutral" => 0.5,
            "informative" => 0.2,
            "educational" => 0.1,
            _ => 0.5,
        }
    }
}

// Helper function to extract content from HTML (simplified)
pub fn extract_content_from_html(html: &str, url: &str) -> ContentData {
    // This is a simplified content extraction
    // In a real implementation, you'd use a proper HTML parser
    let title = extract_between(html, "<title>", "</title>");
    let description = extract_meta_content(html, "description")
        .or_else(|| extract_between(html, "<meta name=\"description\" content=\"", "\""));
    
    ContentData {
        url: url.to_string(),
        title,
        description,
        text_content: Some(html.to_string()),
        metadata: HashMap::new(),
    }
}

fn extract_between(text: &str, start: &str, end: &str) -> Option<String> {
    let start_pos = text.find(start)? + start.len();
    let end_pos = text[start_pos..].find(end)? + start_pos;
    Some(text[start_pos..end_pos].trim().to_string())
}

fn extract_meta_content(html: &str, name: &str) -> Option<String> {
    let pattern = format!("name=\"{}\"", name);
    if let Some(start) = html.find(&pattern) {
        if let Some(content_start) = html[start..].find("content=\"") {
            let content_pos = start + content_start + 9; // length of "content=\""
            if let Some(end_pos) = html[content_pos..].find("\"") {
                return Some(html[content_pos..content_pos + end_pos].to_string());
            }
        }
    }
    None
}