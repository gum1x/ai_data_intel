use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use config::{Config, ConfigError, Environment, File};
use anyhow::Result;

/// Configuration management for the intelligence system

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub kafka: KafkaConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
    pub ai_providers: HashMap<String, AiProviderConfig>,
    pub agents: AgentsConfig,
    pub analytics: AnalyticsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub enable_cors: bool,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub postgres_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
    pub command_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    pub bootstrap_servers: Vec<String>,
    pub group_id: String,
    pub auto_offset_reset: String,
    pub enable_auto_commit: bool,
    pub session_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub max_poll_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub encryption_key: String,
    pub rate_limit_requests: u32,
    pub rate_limit_window_seconds: u64,
    pub allowed_ips: Vec<String>,
    pub enable_audit_logging: bool,
    pub password_min_length: usize,
    pub password_require_special_chars: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub prometheus_port: u16,
    pub jaeger_endpoint: Option<String>,
    pub log_level: String,
    pub enable_tracing: bool,
    pub metrics_retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    pub api_key: String,
    pub base_url: String,
    pub rate_limit_requests_per_minute: u32,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfig {
    pub max_concurrent_tasks: u32,
    pub task_timeout_seconds: u64,
    pub health_check_interval_seconds: u64,
    pub agent_types: HashMap<String, AgentTypeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTypeConfig {
    pub enabled: bool,
    pub max_instances: u32,
    pub resources: ResourceConfig,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub cpu_limit: String,
    pub memory_limit: String,
    pub disk_limit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub batch_size: usize,
    pub processing_interval_seconds: u64,
    pub retention_days: u32,
    pub enable_real_time: bool,
    pub model_paths: HashMap<String, String>,
}

impl IntelligenceConfig {
    pub fn load() -> Result<Self> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        
        let config = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("INTELLIGENCE").separator("_"))
            .build()?;
        
        let config: IntelligenceConfig = config.try_deserialize()?;
        config.validate()?;
        Ok(config)
    }
    
    fn validate(&self) -> Result<()> {
        if self.server.port == 0 {
            return Err(anyhow::anyhow!("Server port cannot be 0"));
        }
        
        if self.database.postgres_url.is_empty() {
            return Err(anyhow::anyhow!("Database URL cannot be empty"));
        }
        
        if self.security.jwt_secret.len() < 32 {
            return Err(anyhow::anyhow!("JWT secret must be at least 32 characters"));
        }
        
        if self.security.encryption_key.len() < 32 {
            return Err(anyhow::anyhow!("Encryption key must be at least 32 characters"));
        }
        
        Ok(())
    }
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: num_cpus::get(),
                max_connections: 1000,
                timeout_seconds: 30,
                enable_cors: true,
                cors_origins: vec!["*".to_string()],
            },
            database: DatabaseConfig {
                postgres_url: "postgresql://localhost/intelligence".to_string(),
                max_connections: 20,
                min_connections: 5,
                connection_timeout_seconds: 30,
                idle_timeout_seconds: 600,
                max_lifetime_seconds: 1800,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                max_connections: 10,
                connection_timeout_seconds: 5,
                command_timeout_seconds: 3,
            },
            kafka: KafkaConfig {
                bootstrap_servers: vec!["localhost:9092".to_string()],
                group_id: "intelligence-system".to_string(),
                auto_offset_reset: "latest".to_string(),
                enable_auto_commit: true,
                session_timeout_ms: 30000,
                heartbeat_interval_ms: 3000,
                max_poll_interval_ms: 300000,
            },
            security: SecurityConfig {
                jwt_secret: "your-super-secret-jwt-key-here-must-be-32-chars-min".to_string(),
                jwt_expiry_hours: 24,
                encryption_key: "your-super-secret-encryption-key-32-chars".to_string(),
                rate_limit_requests: 1000,
                rate_limit_window_seconds: 3600,
                allowed_ips: vec![],
                enable_audit_logging: true,
                password_min_length: 12,
                password_require_special_chars: true,
            },
            monitoring: MonitoringConfig {
                prometheus_port: 9090,
                jaeger_endpoint: None,
                log_level: "info".to_string(),
                enable_tracing: true,
                metrics_retention_days: 30,
            },
            ai_providers: HashMap::new(),
            agents: AgentsConfig {
                max_concurrent_tasks: 100,
                task_timeout_seconds: 300,
                health_check_interval_seconds: 30,
                agent_types: HashMap::new(),
            },
            analytics: AnalyticsConfig {
                batch_size: 1000,
                processing_interval_seconds: 60,
                retention_days: 90,
                enable_real_time: true,
                model_paths: HashMap::new(),
            },
        }
    }
}
