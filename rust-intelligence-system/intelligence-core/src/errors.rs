use thiserror::Error;
#[derive(Error, Debug)]
pub enum IntelligenceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Kafka error: {0}")]
    Kafka(#[from] kafka::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),
    #[error("Authentication error: {message}")]
    Authentication { message: String },
    #[error("Authorization error: {message}")]
    Authorization { message: String },
    #[error("Data not found: {resource}")]
    NotFound { resource: String },
    #[error("Invalid input: {field} - {message}")]
    InvalidInput { field: String, message: String },
    #[error("Rate limit exceeded: {service}")]
    RateLimitExceeded { service: String },
    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    #[error("Security error: {message}")]
    Security { message: String },
    #[error("Processing error: {message}")]
    Processing { message: String },
    #[error("Timeout error: {operation}")]
    Timeout { operation: String },
    #[error("Internal error: {message}")]
    Internal { message: String },
}
pub type Result<T> = std::result::Result<T, IntelligenceError>;
impl IntelligenceError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            IntelligenceError::Database(_) |
            IntelligenceError::Redis(_) |
            IntelligenceError::Kafka(_) |
            IntelligenceError::ExternalService { .. } |
            IntelligenceError::Timeout { .. }
        )
    }
    pub fn is_security_related(&self) -> bool {
        matches!(
            self,
            IntelligenceError::Authentication { .. } |
            IntelligenceError::Authorization { .. } |
            IntelligenceError::Security { .. }
        )
    }
}
