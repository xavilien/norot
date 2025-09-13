use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, Method, StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::{any, get},
    Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::{sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, error};

use crate::config::{Config, FilterAction};
use crate::classifier::{ContentClassifier, extract_content_from_html};
use crate::filter::ContentFilter;
use crate::ui;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: SqlitePool,
    pub filter: Arc<ContentFilter>,
    pub client: reqwest::Client,
}

impl AppState {
    pub fn new(config: Config, db_pool: SqlitePool) -> Self {
        let classifier = ContentClassifier::new(config.classifier.clone());
        let filter = Arc::new(ContentFilter::new(config.filters.clone(), classifier));
        
        Self {
            config,
            db_pool,
            filter,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

#[derive(Deserialize)]
struct ProxyQuery {
    url: Option<String>,
    norot_bypass: Option<String>,
}

pub async fn create_app(state: AppState) -> Result<Router> {
    let app = Router::new()
        // UI routes
        .route("/", get(ui::dashboard))
        .route("/api/stats", get(ui::api_stats))
        .route("/api/recent", get(ui::api_recent_content))
        .route("/api/config", get(ui::api_get_config))
        .route("/static/*file", get(ui::static_files))
        // Proxy route - handles all other requests
        .fallback(any(proxy_handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state);
    
    Ok(app)
}

async fn proxy_handler(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    Query(query): Query<ProxyQuery>,
    body: axum::body::Body,
) -> impl IntoResponse {
    // If this is a request to our UI, let it pass through
    if uri.path().starts_with("/api/") || uri.path() == "/" || uri.path().starts_with("/static/") {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }
    
    // Extract target URL from various sources
    let target_url = if let Some(url) = query.url {
        url
    } else {
        // Try to construct URL from Host header and path
        if let Some(host) = headers.get("host") {
            format!("https://{}{}", host.to_str().unwrap_or(""), uri.path_and_query().map(|pq| pq.as_str()).unwrap_or(""))
        } else {
            return (StatusCode::BAD_REQUEST, "No target URL provided").into_response();
        }
    };
    
    info!("Proxying request to: {}", target_url);
    
    // Check for bypass parameter
    let bypass = query.norot_bypass.is_some();
    
    // For non-bypass requests, check if we should filter
    if !bypass && should_filter_url(&target_url) {
        match filter_request(&state, &target_url).await {
            Ok(response) => return response,
            Err(e) => {
                error!("Error filtering request: {}", e);
                // Continue with proxy on error
            }
        }
    }
    
    // Proxy the request
    match proxy_request(&state, &method, &target_url, &headers, body).await {
        Ok(response) => response,
        Err(e) => {
            error!("Error proxying request: {}", e);
            (StatusCode::BAD_GATEWAY, format!("Proxy error: {}", e)).into_response()
        }
    }
}

fn should_filter_url(url: &str) -> bool {
    // Only filter social media and content sites
    let social_media_domains = [
        "instagram.com",
        "facebook.com", 
        "twitter.com",
        "x.com",
        "tiktok.com",
        "youtube.com",
        "reddit.com",
        "snapchat.com",
        "linkedin.com",
    ];
    
    social_media_domains.iter().any(|domain| url.contains(domain))
}

async fn filter_request(state: &AppState, url: &str) -> Result<Response> {
    // Fetch the content first (for GET requests)
    let response = state.client.get(url).send().await?;
    
    if !response.status().is_success() {
        return Ok((StatusCode::BAD_GATEWAY, "Failed to fetch content").into_response());
    }
    
    let content_type = response.headers()
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("");
    
    // Only filter HTML content
    if !content_type.contains("text/html") {
        // For non-HTML content, just proxy it through
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.bytes().await?;
        
        let mut response_builder = axum::response::Response::builder()
            .status(axum::http::StatusCode::from_u16(status.as_u16())?);
            
        for (key, value) in headers.iter() {
            if let (Ok(axum_key), Ok(axum_value)) = (
                axum::http::HeaderName::from_bytes(key.as_str().as_bytes()),
                axum::http::HeaderValue::from_bytes(value.as_bytes())
            ) {
                response_builder = response_builder.header(axum_key, axum_value);
            }
        }
        
        return Ok(response_builder.body(axum::body::Body::from(body))?);
    }
    
    let html = response.text().await?;
    let content_data = extract_content_from_html(&html, url);
    
    // Evaluate content through filter
    let decision = state.filter.evaluate_content(&content_data, &state.db_pool).await?;
    
    match decision.action {
        FilterAction::Block => {
            let blocked_html = state.filter.generate_blocked_content_html(&decision, url);
            Ok(Html(blocked_html).into_response())
        },
        FilterAction::Throttle(seconds) => {
            info!("Throttling request to {} for {} seconds", url, seconds);
            tokio::time::sleep(Duration::from_secs(seconds)).await;
            
            // Return the original content after delay
            Ok(Html(html).into_response())
        },
        FilterAction::Warning => {
            // Inject warning into the content
            let warning_html = inject_warning(&html, &decision);
            Ok(Html(warning_html).into_response())
        },
        FilterAction::Allow => {
            Ok(Html(html).into_response())
        }
    }
}

async fn proxy_request(
    state: &AppState,
    method: &Method,
    url: &str,
    headers: &HeaderMap,
    body: axum::body::Body,
) -> Result<Response> {
    let mut request_builder = state.client.request(
        reqwest::Method::from_bytes(method.as_str().as_bytes())?,
        url
    );
    
    // Copy relevant headers
    for (key, value) in headers.iter() {
        let key_str = key.as_str();
        // Skip headers that shouldn't be forwarded
        if !["host", "connection", "transfer-encoding", "content-length"].contains(&key_str) {
            if let (Ok(req_key), Ok(req_value)) = (
                reqwest::header::HeaderName::from_bytes(key_str.as_bytes()),
                reqwest::header::HeaderValue::from_bytes(value.as_bytes())
            ) {
                request_builder = request_builder.header(req_key, req_value);
            }
        }
    }
    
    // Add body for POST/PUT requests
    if method == Method::POST || method == Method::PUT || method == Method::PATCH {
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await?;
        request_builder = request_builder.body(body_bytes);
    }
    
    let response = request_builder.send().await?;
    
    let status = response.status();
    let headers = response.headers().clone();
    let body_bytes = response.bytes().await?;
    
    let mut response_builder = axum::response::Response::builder()
        .status(axum::http::StatusCode::from_u16(status.as_u16())?);
        
    for (key, value) in headers.iter() {
        if let (Ok(axum_key), Ok(axum_value)) = (
            axum::http::HeaderName::from_bytes(key.as_str().as_bytes()),
            axum::http::HeaderValue::from_bytes(value.as_bytes())
        ) {
            response_builder = response_builder.header(axum_key, axum_value);
        }
    }
    
    Ok(response_builder.body(axum::body::Body::from(body_bytes))?)
}

fn inject_warning(html: &str, decision: &crate::filter::FilterDecision) -> String {
    let warning_banner = format!(
        r#"
        <div id="norot-warning" style="
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            background: linear-gradient(135deg, #ff9500, #ff6b35);
            color: white;
            padding: 12px 20px;
            text-align: center;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            font-size: 14px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.3);
            z-index: 9999;
            cursor: pointer;
        " onclick="this.remove()">
            ⚠️ <strong>NoRot Warning:</strong> This content was classified as "{}" ({:.0}% confidence). {}
            <span style="float: right; font-size: 18px;">&times;</span>
        </div>
        <script>
            document.body.style.marginTop = '50px';
            setTimeout(() => {{
                const warning = document.getElementById('norot-warning');
                if (warning) warning.remove();
                document.body.style.marginTop = '0';
            }}, 5000);
        </script>
        "#,
        decision.classification.category,
        decision.classification.confidence * 100.0,
        decision.reason
    );
    
    // Try to inject after <body> tag, or at the beginning if not found
    if let Some(body_pos) = html.find("<body") {
        if let Some(body_end) = html[body_pos..].find(">") {
            let insert_pos = body_pos + body_end + 1;
            format!("{}{}{}", &html[..insert_pos], warning_banner, &html[insert_pos..])
        } else {
            format!("{}{}", warning_banner, html)
        }
    } else {
        format!("{}{}", warning_banner, html)
    }
}