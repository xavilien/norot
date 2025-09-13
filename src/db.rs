use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRecord {
    pub id: String,
    pub url: String,
    pub content_type: String,
    pub classification: Option<String>,
    pub confidence: Option<f32>,
    pub action_taken: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterStats {
    pub total_requests: i64,
    pub blocked_requests: i64,
    pub allowed_requests: i64,
    pub throttled_requests: i64,
}

pub async fn init_database() -> Result<SqlitePool> {
    // Create database directory if it doesn't exist
    std::fs::create_dir_all("data")?;
    
    let database_url = "sqlite:data/norot.db";
    let pool = SqlitePool::connect(database_url).await?;
    
    // Run migrations
    create_tables(&pool).await?;
    
    Ok(pool)
}

async fn create_tables(pool: &SqlitePool) -> Result<()> {
    // Create content_records table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS content_records (
            id TEXT PRIMARY KEY,
            url TEXT NOT NULL,
            content_type TEXT NOT NULL,
            classification TEXT,
            confidence REAL,
            action_taken TEXT NOT NULL,
            timestamp TEXT NOT NULL
        )
        "#
    )
    .execute(pool)
    .await?;
    
    // Create filter_stats table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS filter_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            total_requests INTEGER DEFAULT 0,
            blocked_requests INTEGER DEFAULT 0,
            allowed_requests INTEGER DEFAULT 0,
            throttled_requests INTEGER DEFAULT 0,
            UNIQUE(date)
        )
        "#
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn save_content_record(pool: &SqlitePool, record: &ContentRecord) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO content_records (id, url, content_type, classification, confidence, action_taken, timestamp)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&record.id)
    .bind(&record.url)
    .bind(&record.content_type)
    .bind(&record.classification)
    .bind(record.confidence)
    .bind(&record.action_taken)
    .bind(record.timestamp.to_rfc3339())
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn get_recent_records(pool: &SqlitePool, limit: i64) -> Result<Vec<ContentRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT id, url, content_type, classification, confidence, action_taken, timestamp
        FROM content_records
        ORDER BY timestamp DESC
        LIMIT ?
        "#
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    
    let mut records = Vec::new();
    for row in rows {
        let timestamp_str: String = row.get("timestamp");
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)?
            .with_timezone(&Utc);
        
        records.push(ContentRecord {
            id: row.get("id"),
            url: row.get("url"),
            content_type: row.get("content_type"),
            classification: row.get("classification"),
            confidence: row.get("confidence"),
            action_taken: row.get("action_taken"),
            timestamp,
        });
    }
    
    Ok(records)
}

pub async fn update_filter_stats(
    pool: &SqlitePool,
    action: &str,
) -> Result<()> {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    
    // First, try to update existing record
    let result = match action {
        "blocked" => {
            sqlx::query(
                r#"
                UPDATE filter_stats 
                SET blocked_requests = blocked_requests + 1,
                    total_requests = total_requests + 1
                WHERE date = ?
                "#
            )
            .bind(&today)
            .execute(pool)
            .await?
        },
        "allowed" => {
            sqlx::query(
                r#"
                UPDATE filter_stats 
                SET allowed_requests = allowed_requests + 1,
                    total_requests = total_requests + 1
                WHERE date = ?
                "#
            )
            .bind(&today)
            .execute(pool)
            .await?
        },
        "throttled" => {
            sqlx::query(
                r#"
                UPDATE filter_stats 
                SET throttled_requests = throttled_requests + 1,
                    total_requests = total_requests + 1
                WHERE date = ?
                "#
            )
            .bind(&today)
            .execute(pool)
            .await?
        },
        _ => return Ok(()),
    };
    
    // If no rows were affected, insert a new record
    if result.rows_affected() == 0 {
        let (blocked, allowed, throttled) = match action {
            "blocked" => (1, 0, 0),
            "allowed" => (0, 1, 0),
            "throttled" => (0, 0, 1),
            _ => (0, 0, 0),
        };
        
        sqlx::query(
            r#"
            INSERT INTO filter_stats (date, total_requests, blocked_requests, allowed_requests, throttled_requests)
            VALUES (?, 1, ?, ?, ?)
            "#
        )
        .bind(&today)
        .bind(blocked)
        .bind(allowed)
        .bind(throttled)
        .execute(pool)
        .await?;
    }
    
    Ok(())
}

pub async fn get_filter_stats(pool: &SqlitePool) -> Result<FilterStats> {
    let row = sqlx::query(
        r#"
        SELECT 
            COALESCE(SUM(total_requests), 0) as total_requests,
            COALESCE(SUM(blocked_requests), 0) as blocked_requests,
            COALESCE(SUM(allowed_requests), 0) as allowed_requests,
            COALESCE(SUM(throttled_requests), 0) as throttled_requests
        FROM filter_stats
        WHERE date >= date('now', '-30 days')
        "#
    )
    .fetch_one(pool)
    .await?;
    
    Ok(FilterStats {
        total_requests: row.get("total_requests"),
        blocked_requests: row.get("blocked_requests"),
        allowed_requests: row.get("allowed_requests"),
        throttled_requests: row.get("throttled_requests"),
    })
}