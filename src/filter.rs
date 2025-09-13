use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::{FilterConfig, FilterRule, FilterAction};
use crate::classifier::{ContentClassifier, ClassificationResult, ContentData};
use crate::db::{self, ContentRecord};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterDecision {
    pub action: FilterAction,
    pub reason: String,
    pub classification: ClassificationResult,
    pub should_notify: bool,
}

pub struct ContentFilter {
    config: FilterConfig,
    classifier: ContentClassifier,
}

impl ContentFilter {
    pub fn new(config: FilterConfig, classifier: ContentClassifier) -> Self {
        Self {
            config,
            classifier,
        }
    }
    
    pub async fn evaluate_content(
        &self,
        content: &ContentData,
        pool: &sqlx::SqlitePool,
    ) -> Result<FilterDecision> {
        // If filtering is disabled, allow everything
        if !self.config.enabled {
            return Ok(FilterDecision {
                action: FilterAction::Allow,
                reason: "Filtering disabled".to_string(),
                classification: ClassificationResult {
                    category: "unclassified".to_string(),
                    confidence: 0.0,
                    reasoning: Some("Filtering disabled".to_string()),
                },
                should_notify: false,
            });
        }
        
        // Classify the content
        let classification = self.classifier.classify_content(content).await?;
        
        // Apply filter rules
        let decision = self.apply_filter_rules(&classification, content);
        
        // Log the decision
        self.log_decision(content, &classification, &decision, pool).await?;
        
        Ok(decision)
    }
    
    fn apply_filter_rules(&self, classification: &ClassificationResult, _content: &ContentData) -> FilterDecision {
        let mut best_match: Option<(&String, &FilterRule)> = None;
        let mut best_confidence = 0.0;
        
        // Find the best matching rule
        for (rule_name, rule) in &self.config.rules {
            if !rule.enabled {
                continue;
            }
            
            // Check if classification matches any of the rule's categories
            if rule.categories.contains(&classification.category) {
                if classification.confidence > best_confidence && 
                   classification.confidence >= rule.score_threshold {
                    best_match = Some((rule_name, rule));
                    best_confidence = classification.confidence;
                }
            }
        }
        
        // Apply the best matching rule or default action
        let (action, reason) = if let Some((rule_name, rule)) = best_match {
            (
                rule.action.clone(),
                format!("Matched rule '{}' for category '{}' with confidence {:.2}", 
                        rule_name, classification.category, classification.confidence)
            )
        } else {
            // Default action based on brainrot score
            let brainrot_score = self.classifier.get_brainrot_score(classification);
            if brainrot_score > 0.7 {
                (FilterAction::Block, format!("High brainrot score: {:.2}", brainrot_score))
            } else if brainrot_score > 0.5 {
                (FilterAction::Throttle(3), format!("Medium brainrot score: {:.2}", brainrot_score))
            } else {
                (FilterAction::Allow, format!("Low brainrot score: {:.2}", brainrot_score))
            }
        };
        
        // Determine if we should notify
        let should_notify = match &action {
            FilterAction::Block => classification.confidence >= self.config.notification_threshold,
            FilterAction::Throttle(_) => classification.confidence >= self.config.notification_threshold,
            _ => false,
        };
        
        FilterDecision {
            action,
            reason,
            classification: classification.clone(),
            should_notify,
        }
    }
    
    async fn log_decision(
        &self,
        content: &ContentData,
        classification: &ClassificationResult,
        decision: &FilterDecision,
        pool: &sqlx::SqlitePool,
    ) -> Result<()> {
        let action_str = match &decision.action {
            FilterAction::Block => "blocked".to_string(),
            FilterAction::Throttle(seconds) => format!("throttled_{}", seconds),
            FilterAction::Warning => "warning".to_string(),
            FilterAction::Allow => "allowed".to_string(),
        };
        
        let record = ContentRecord {
            id: Uuid::new_v4().to_string(),
            url: content.url.clone(),
            content_type: classification.category.clone(),
            classification: Some(classification.category.clone()),
            confidence: Some(classification.confidence),
            action_taken: action_str.clone(),
            timestamp: Utc::now(),
        };
        
        // Save to database
        db::save_content_record(pool, &record).await?;
        
        // Update statistics
        let stats_action = match &decision.action {
            FilterAction::Block => "blocked",
            FilterAction::Throttle(_) => "throttled",
            _ => "allowed",
        };
        db::update_filter_stats(pool, stats_action).await?;
        
        Ok(())
    }
    
    pub fn should_block(&self, decision: &FilterDecision) -> bool {
        matches!(decision.action, FilterAction::Block)
    }
    
    pub fn get_throttle_delay(&self, decision: &FilterDecision) -> Option<u64> {
        match decision.action {
            FilterAction::Throttle(seconds) => Some(seconds),
            _ => None,
        }
    }
    
    pub fn generate_blocked_content_html(&self, decision: &FilterDecision, url: &str) -> String {
        format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Content Blocked - NoRot</title>
                <style>
                    body {{
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                        max-width: 600px;
                        margin: 100px auto;
                        padding: 20px;
                        background: #1a1a1a;
                        color: #ffffff;
                        text-align: center;
                    }}
                    .container {{
                        background: #2d2d2d;
                        padding: 40px;
                        border-radius: 12px;
                        box-shadow: 0 8px 32px rgba(0,0,0,0.3);
                    }}
                    .icon {{
                        font-size: 64px;
                        margin-bottom: 20px;
                    }}
                    h1 {{
                        color: #ff6b6b;
                        margin-bottom: 16px;
                    }}
                    .reason {{
                        background: #3a3a3a;
                        padding: 20px;
                        border-radius: 8px;
                        margin: 20px 0;
                        border-left: 4px solid #ff6b6b;
                    }}
                    .url {{
                        color: #888;
                        word-break: break-all;
                        margin: 10px 0;
                    }}
                    .actions {{
                        margin-top: 30px;
                    }}
                    button {{
                        background: #4dabf7;
                        color: white;
                        border: none;
                        padding: 12px 24px;
                        border-radius: 6px;
                        margin: 0 10px;
                        cursor: pointer;
                        font-size: 16px;
                    }}
                    button:hover {{
                        background: #339af0;
                    }}
                    .proceed-btn {{
                        background: #ff8787;
                    }}
                    .proceed-btn:hover {{
                        background: #ff6b6b;
                    }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="icon">🛡️</div>
                    <h1>Content Blocked</h1>
                    <p>This content was identified as potentially low-value and has been blocked to help you avoid doomscrolling.</p>
                    
                    <div class="reason">
                        <strong>Reason:</strong> {}<br>
                        <strong>Category:</strong> {}<br>
                        <strong>Confidence:</strong> {:.1}%
                    </div>
                    
                    <div class="url">
                        <strong>URL:</strong> {}
                    </div>
                    
                    <div class="actions">
                        <button onclick="history.back()">Go Back</button>
                        <button class="proceed-btn" onclick="proceedAnyway()">Proceed Anyway</button>
                    </div>
                </div>
                
                <script>
                    function proceedAnyway() {{
                        if (confirm('Are you sure you want to view this content? It may lead to mindless scrolling.')) {{
                            window.location.href = '{}?norot_bypass=1';
                        }}
                    }}
                </script>
            </body>
            </html>
            "#,
            decision.reason,
            decision.classification.category,
            decision.classification.confidence * 100.0,
            url,
            url
        )
    }
}