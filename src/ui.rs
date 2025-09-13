use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Json},
    http::StatusCode,
};
use serde_json::json;
use tracing::error;

use crate::proxy::AppState;
use crate::db;

pub async fn dashboard(State(_state): State<AppState>) -> impl IntoResponse {
    Html(include_str!("../templates/dashboard.html"))
}

pub async fn api_stats(State(state): State<AppState>) -> impl IntoResponse {
    match db::get_filter_stats(&state.db_pool).await {
        Ok(stats) => Json(json!(stats)).into_response(),
        Err(e) => {
            error!("Failed to get filter stats: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to get stats"}))).into_response()
        }
    }
}

pub async fn api_recent_content(State(state): State<AppState>) -> impl IntoResponse {
    match db::get_recent_records(&state.db_pool, 50).await {
        Ok(records) => Json(json!(records)).into_response(),
        Err(e) => {
            error!("Failed to get recent content: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to get recent content"}))).into_response()
        }
    }
}

pub async fn api_get_config(State(state): State<AppState>) -> impl IntoResponse {
    Json(json!(state.config))
}

pub async fn static_files(Path(file): Path<String>) -> impl IntoResponse {
    match file.as_str() {
        "style.css" => (
            [("content-type", "text/css")],
            include_str!("../static/style.css")
        ).into_response(),
        "script.js" => (
            [("content-type", "application/javascript")],
            include_str!("../static/script.js")
        ).into_response(),
        _ => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}