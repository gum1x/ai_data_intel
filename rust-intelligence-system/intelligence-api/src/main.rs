use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
    compression::CompressionLayer,
};
use intelligence_core::{IntelligenceConfig, SystemHealth};
use intelligence_security::{AuthenticationService, AuthorizationService};
use intelligence_analytics::{DataProcessor, MLModelManager, MassProcessor, UsernameAnalyzer, NFTTracker, RelationshipMapper};
use intelligence_monitoring::{HealthCheckService, MetricsCollector};
use intelligence_collectors::{TelegramCollector, WebScraper, StreamProcessor};
#[derive(Clone)]
pub struct AppState {
    pub config: IntelligenceConfig,
    pub auth_service: Arc<AuthenticationService>,
    pub authz_service: Arc<AuthorizationService>,
    pub data_processor: Arc<DataProcessor>,
    pub ml_models: Arc<MLModelManager>,
    pub mass_processor: Arc<MassProcessor>,
    pub username_analyzer: Arc<UsernameAnalyzer>,
    pub nft_tracker: Arc<NFTTracker>,
    pub relationship_mapper: Arc<RelationshipMapper>,
    pub telegram_collector: Arc<TelegramCollector>,
    pub web_scraper: Arc<WebScraper>,
    pub stream_processor: Arc<StreamProcessor>,
    pub health_service: Arc<HealthCheckService>,
    pub metrics: Arc<MetricsCollector>,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    info!("Starting Intelligence API Server");
    let config = IntelligenceConfig::load()
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;
    let auth_service = Arc::new(AuthenticationService::new(
        config.security.jwt_secret.clone(),
        std::time::Duration::from_secs(config.security.jwt_expiry_hours * 3600),
    ));
    let authz_service = Arc::new(AuthorizationService::new());
    let data_processor = Arc::new(DataProcessor::new(0.8, 300));
    let ml_models = Arc::new(MLModelManager::new());
    let mass_processor = Arc::new(MassProcessor::new(intelligence_analytics::MassProcessingConfig {
        max_concurrent_processors: 8,
        batch_size: 10000,
        processing_timeout_seconds: 300,
        memory_limit_mb: 8192,
        cpu_limit_percent: 80.0,
        enable_parallel_processing: true,
        enable_distributed_processing: true,
        chunk_size: 1000,
        max_retries: 3,
        retry_delay_seconds: 5,
    }));
    let username_analyzer = Arc::new(UsernameAnalyzer::new());
    let nft_tracker = Arc::new(NFTTracker::new());
    let relationship_mapper = Arc::new(RelationshipMapper::new());
    let telegram_collector = Arc::new(TelegramCollector::new(intelligence_collectors::TelegramConfig {
        api_id: 12345,
        api_hash: "your_api_hash".to_string(),
        phone_number: "+1234567890".to_string(),
        session_string: None,
        max_concurrent_sessions: 5,
        rate_limit_per_second: 30,
        batch_size: 1000,
        collection_timeout_seconds: 300,
    }));
    let web_scraper = Arc::new(WebScraper::new(intelligence_collectors::ScrapingConfig {
        max_concurrent_requests: 10,
        request_timeout_seconds: 30,
        rate_limit_per_second: 10,
        user_agents: vec!["Mozilla/5.0 (compatible; IntelligenceBot/1.0)".to_string()],
        proxy_list: vec![],
        respect_robots_txt: true,
        follow_redirects: true,
        max_redirects: 5,
        batch_size: 1000,
        retry_attempts: 3,
        retry_delay_seconds: 2,
    }));
    let stream_processor = Arc::new(StreamProcessor::new(intelligence_collectors::StreamConfig {
        kafka_bootstrap_servers: vec!["localhost:9092".to_string()],
        consumer_group_id: "intelligence-system".to_string(),
        topics: vec![
            "telegram_messages".to_string(),
            "web_scraping".to_string(),
            "api_data".to_string(),
            "blockchain_data".to_string(),
        ],
        batch_size: 1000,
        processing_timeout_ms: 30000,
        max_concurrent_processors: 8,
        buffer_size: 10000,
        commit_interval_ms: 5000,
        auto_offset_reset: "latest".to_string(),
    }));
    let health_service = Arc::new(HealthCheckService::new(
        config.system.version.clone(),
        config.system.environment.clone(),
    ));
    let metrics = Arc::new(MetricsCollector::new());
    let state = AppState {
        config: config.clone(),
        auth_service,
        authz_service,
        data_processor,
        ml_models,
        mass_processor,
        username_analyzer,
        nft_tracker,
        relationship_mapper,
        telegram_collector,
        web_scraper,
        stream_processor,
        health_service,
        metrics,
    };
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/metrics", get(metrics_handler))
        .route("/api/v1/auth/login", post(login_handler))
        .route("/api/v1/auth/logout", post(logout_handler))
        .route("/api/v1/auth/refresh", post(refresh_handler))
        .route("/api/v1/data", get(get_data_handler))
        .route("/api/v1/data", post(create_data_handler))
        .route("/api/v1/analysis", post(analyze_data_handler))
        .route("/api/v1/agents", get(get_agents_handler))
        .route("/api/v1/agents/:id/tasks", post(assign_task_handler))
        .route("/api/v1/system/status", get(system_status_handler))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(std::time::Duration::from_secs(30)))
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive())
        );
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    info!("Server shutdown complete");
    Ok(())
}
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("Shutdown signal received");
}
async fn health_handler(State(state): State<AppState>) -> Result<Json<SystemHealth>, StatusCode> {
    match state.health_service.get_system_health().await {
        health => Ok(Json(health)),
    }
}
async fn ready_handler(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let health = state.health_service.get_system_health().await;
    let ready = match health.overall_status {
        intelligence_core::HealthStatus::Healthy => true,
        intelligence_core::HealthStatus::Degraded => true,
        _ => false,
    };
    Ok(Json(serde_json::json!({
        "ready": ready,
        "status": health.overall_status,
        "timestamp": health.timestamp
    })))
}
async fn metrics_handler(State(state): State<AppState>) -> Result<String, StatusCode> {
    Ok("# HELP intelligence_system_info System information\n# TYPE intelligence_system_info gauge\nintelligence_system_info{version=\"1.0.0\"} 1\n".to_string())
}
async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let username = payload.get("username")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let password = payload.get("password")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if username.is_empty() || password.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let login_request = intelligence_security::LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
        remember_me: payload.get("remember_me")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        ip_address: None,
        user_agent: None,
    };
    match state.auth_service.login(login_request).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).unwrap())),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
async fn logout_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let session_id = payload.get("session_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok());
    if let Some(session_id) = session_id {
        match state.auth_service.logout(session_id).await {
            Ok(_) => Ok(Json(serde_json::json!({"success": true}))),
            Err(_) => Err(StatusCode::BAD_REQUEST),
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}
async fn refresh_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let refresh_token = payload.get("refresh_token")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if refresh_token.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    match state.auth_service.refresh_token(refresh_token).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).unwrap())),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
async fn get_data_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "data": [],
        "total": 0,
        "page": 1,
        "per_page": 100
    })))
}
async fn create_data_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "id": uuid::Uuid::new_v4(),
        "status": "created",
        "timestamp": chrono::Utc::now()
    })))
}
async fn analyze_data_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "analysis_id": uuid::Uuid::new_v4(),
        "status": "completed",
        "results": {},
        "timestamp": chrono::Utc::now()
    })))
}
async fn get_agents_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "agents": [],
        "total": 0
    })))
}
async fn assign_task_handler(
    State(state): State<AppState>,
    axum::extract::Path(agent_id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "task_id": uuid::Uuid::new_v4(),
        "agent_id": agent_id,
        "status": "assigned",
        "timestamp": chrono::Utc::now()
    })))
}
async fn system_status_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let health = state.health_service.get_system_health().await;
    let metrics = state.metrics.get_system_metrics().await;
    Ok(Json(serde_json::json!({
        "health": health,
        "metrics": metrics,
        "version": state.config.system.version,
        "environment": state.config.system.environment
    })))
}
