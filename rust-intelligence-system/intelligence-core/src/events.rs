use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::types::*;

/// Event system for the intelligence platform

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceEvent {
    pub id: Uuid,
    pub event_type: EventType,
    pub source: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    // Data events
    DataCollected,
    DataProcessed,
    DataValidated,
    DataStored,
    DataRetrieved,
    DataDeleted,
    
    // Analysis events
    AnalysisStarted,
    AnalysisCompleted,
    AnalysisFailed,
    AnalysisResultGenerated,
    
    // Agent events
    AgentStarted,
    AgentStopped,
    AgentTaskAssigned,
    AgentTaskCompleted,
    AgentTaskFailed,
    AgentHealthCheck,
    
    // Threat events
    ThreatDetected,
    ThreatAssessed,
    ThreatEscalated,
    ThreatResolved,
    
    // User events
    UserLogin,
    UserLogout,
    UserAction,
    UserPermissionChanged,
    
    // System events
    SystemStartup,
    SystemShutdown,
    SystemError,
    SystemWarning,
    SystemMaintenance,
    
    // Security events
    SecurityAlert,
    AuthenticationFailed,
    AuthorizationDenied,
    DataAccessAttempt,
    SuspiciousActivity,
}

impl IntelligenceEvent {
    pub fn new(event_type: EventType, source: String, data: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            source,
            data,
            timestamp: Utc::now(),
            correlation_id: None,
            user_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn with_metadata(mut self, metadata: std::collections::HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Event handler trait for processing events
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle_event(&self, event: &IntelligenceEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    fn event_types(&self) -> Vec<EventType>;
    
    fn priority(&self) -> EventPriority {
        EventPriority::Normal
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Event bus for managing event distribution
pub struct EventBus {
    handlers: std::collections::HashMap<EventType, Vec<Box<dyn EventHandler>>>,
    event_queue: tokio::sync::mpsc::UnboundedSender<IntelligenceEvent>,
}

impl EventBus {
    pub fn new() -> (Self, EventProcessor) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        let bus = Self {
            handlers: std::collections::HashMap::new(),
            event_queue: tx,
        };
        
        let processor = EventProcessor::new(rx);
        
        (bus, processor)
    }
    
    pub fn register_handler(&mut self, handler: Box<dyn EventHandler>) {
        for event_type in handler.event_types() {
            self.handlers.entry(event_type).or_insert_with(Vec::new).push(handler);
        }
    }
    
    pub fn publish_event(&self, event: IntelligenceEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.event_queue.send(event)?;
        Ok(())
    }
    
    pub fn get_handlers(&self, event_type: &EventType) -> Vec<&Box<dyn EventHandler>> {
        self.handlers.get(event_type).map(|v| v.iter().collect()).unwrap_or_default()
    }
}

/// Event processor for handling events asynchronously
pub struct EventProcessor {
    receiver: tokio::sync::mpsc::UnboundedReceiver<IntelligenceEvent>,
    handlers: std::collections::HashMap<EventType, Vec<Box<dyn EventHandler>>>,
}

impl EventProcessor {
    fn new(receiver: tokio::sync::mpsc::UnboundedReceiver<IntelligenceEvent>) -> Self {
        Self {
            receiver,
            handlers: std::collections::HashMap::new(),
        }
    }
    
    pub fn register_handler(&mut self, handler: Box<dyn EventHandler>) {
        for event_type in handler.event_types() {
            self.handlers.entry(event_type).or_insert_with(Vec::new).push(handler);
        }
    }
    
    pub async fn start_processing(mut self) {
        while let Some(event) = self.receiver.recv().await {
            if let Some(handlers) = self.handlers.get(&event.event_type) {
                for handler in handlers {
                    if let Err(e) = handler.handle_event(&event).await {
                        tracing::error!("Error handling event {:?}: {}", event.event_type, e);
                    }
                }
            }
        }
    }
}

/// Predefined event creators for common scenarios
pub struct EventFactory;

impl EventFactory {
    pub fn data_collected(source: String, data_id: IntelligenceId) -> IntelligenceEvent {
        IntelligenceEvent::new(
            EventType::DataCollected,
            source,
            serde_json::json!({
                "data_id": data_id.0,
                "timestamp": Utc::now()
            })
        )
    }
    
    pub fn analysis_completed(analysis_id: IntelligenceId, result_count: usize) -> IntelligenceEvent {
        IntelligenceEvent::new(
            EventType::AnalysisCompleted,
            "analytics-engine".to_string(),
            serde_json::json!({
                "analysis_id": analysis_id.0,
                "result_count": result_count,
                "timestamp": Utc::now()
            })
        )
    }
    
    pub fn threat_detected(threat_id: IntelligenceId, threat_level: ThreatLevel) -> IntelligenceEvent {
        IntelligenceEvent::new(
            EventType::ThreatDetected,
            "threat-hunter".to_string(),
            serde_json::json!({
                "threat_id": threat_id.0,
                "threat_level": threat_level,
                "timestamp": Utc::now()
            })
        )
    }
    
    pub fn agent_task_completed(agent_id: String, task_id: IntelligenceId) -> IntelligenceEvent {
        IntelligenceEvent::new(
            EventType::AgentTaskCompleted,
            agent_id,
            serde_json::json!({
                "task_id": task_id.0,
                "timestamp": Utc::now()
            })
        )
    }
    
    pub fn system_error(component: String, error_message: String) -> IntelligenceEvent {
        IntelligenceEvent::new(
            EventType::SystemError,
            component,
            serde_json::json!({
                "error_message": error_message,
                "timestamp": Utc::now()
            })
        )
    }
}
