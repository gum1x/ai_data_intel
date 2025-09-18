use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use validator::Validate;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntelligenceId(pub Uuid);
impl IntelligenceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
impl Default for IntelligenceId {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct IntelligenceData {
    pub id: IntelligenceId,
    #[validate(length(min = 1, max = 1000))]
    pub content: String,
    pub source: DataSource,
    pub classification: DataClassification,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub confidence: f64,
    #[validate(range(min = 0.0, max = 1.0))]
    pub quality_score: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataSource {
    Telegram,
    Blockchain,
    SocialMedia,
    WebScraping,
    Api,
    File,
    Database,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Secret,
    TopSecret,
}
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserProfile {
    pub id: IntelligenceId,
    #[validate(length(min = 1, max = 100))]
    pub username: String,
    pub platform: Platform,
    pub attributes: HashMap<String, serde_json::Value>,
    pub risk_score: f64,
    pub last_seen: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Platform {
    Telegram,
    Twitter,
    Discord,
    Blockchain,
    Other(String),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAssessment {
    pub id: IntelligenceId,
    pub target_id: IntelligenceId,
    pub threat_level: ThreatLevel,
    pub indicators: Vec<ThreatIndicator>,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
    pub analyst_id: Option<IntelligenceId>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicator {
    pub indicator_type: IndicatorType,
    pub value: String,
    pub confidence: f64,
    pub source: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IndicatorType {
    IpAddress,
    Domain,
    Email,
    PhoneNumber,
    CryptocurrencyAddress,
    SocialMediaHandle,
    BehavioralPattern,
    NetworkConnection,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub id: IntelligenceId,
    pub analysis_type: AnalysisType,
    pub input_data: Vec<IntelligenceId>,
    pub results: HashMap<String, serde_json::Value>,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
    pub processing_time_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisType {
    BehaviorAnalysis,
    NetworkAnalysis,
    ThreatDetection,
    PatternRecognition,
    RiskAssessment,
    SentimentAnalysis,
    EntityExtraction,
    LinkAnalysis,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: IntelligenceId,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub input_data: Vec<IntelligenceId>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub assigned_agent: Option<String>,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    DataCollection,
    Analysis,
    ThreatHunting,
    NetworkMapping,
    ProfileBuilding,
    ReportGeneration,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_agents: u32,
    pub tasks_processed: u64,
    pub data_points_collected: u64,
    pub threat_detections: u64,
    pub error_count: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: IntelligenceId,
    pub user_id: Option<IntelligenceId>,
    pub action: String,
    pub resource: String,
    pub details: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
