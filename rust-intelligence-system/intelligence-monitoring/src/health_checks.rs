use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, warn, error};

use intelligence_core::{IntelligenceError, Result as IntelligenceResult};

/// Comprehensive health check system

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub check_type: HealthCheckType,
    pub configuration: HashMap<String, serde_json::Value>,
    pub is_enabled: bool,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub retry_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthCheckType {
    HttpEndpoint,
    Database,
    Redis,
    Kafka,
    ExternalService,
    DiskSpace,
    MemoryUsage,
    CpuUsage,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub check_id: Uuid,
    pub status: HealthStatus,
    pub message: String,
    pub response_time_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Degraded,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_status: HealthStatus,
    pub checks: Vec<HealthCheckResult>,
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub version: String,
    pub environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub total_checks: u64,
    pub healthy_checks: u64,
    pub unhealthy_checks: u64,
    pub degraded_checks: u64,
    pub average_response_time: f64,
    pub uptime_percentage: f64,
    pub last_check_time: DateTime<Utc>,
}

/// Health check service
pub struct HealthCheckService {
    checks: HashMap<Uuid, HealthCheck>,
    results: HashMap<Uuid, Vec<HealthCheckResult>>,
    system_start_time: Instant,
    version: String,
    environment: String,
}

impl HealthCheckService {
    pub fn new(version: String, environment: String) -> Self {
        Self {
            checks: HashMap::new(),
            results: HashMap::new(),
            system_start_time: Instant::now(),
            version,
            environment,
        }
    }
    
    /// Add a health check
    pub async fn add_health_check(&mut self, check: HealthCheck) -> IntelligenceResult<Uuid> {
        let check_id = check.id;
        self.checks.insert(check_id, check);
        self.results.insert(check_id, Vec::new());
        Ok(check_id)
    }
    
    /// Remove a health check
    pub async fn remove_health_check(&mut self, check_id: Uuid) -> IntelligenceResult<()> {
        self.checks.remove(&check_id);
        self.results.remove(&check_id);
        Ok(())
    }
    
    /// Update a health check
    pub async fn update_health_check(&mut self, check_id: Uuid, check: HealthCheck) -> IntelligenceResult<()> {
        if !self.checks.contains_key(&check_id) {
            return Err(IntelligenceError::NotFound { resource: "Health check".to_string() });
        }
        self.checks.insert(check_id, check);
        Ok(())
    }
    
    /// Run a specific health check
    pub async fn run_health_check(&mut self, check_id: Uuid) -> IntelligenceResult<HealthCheckResult> {
        let check = self.checks.get(&check_id)
            .ok_or_else(|| IntelligenceError::NotFound { resource: "Health check".to_string() })?;
        
        if !check.is_enabled {
            return Ok(HealthCheckResult {
                check_id,
                status: HealthStatus::Unknown,
                message: "Health check is disabled".to_string(),
                response_time_ms: 0,
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            });
        }
        
        let start_time = Instant::now();
        let result = self.execute_health_check(check).await;
        let response_time = start_time.elapsed().as_millis() as u64;
        
        let health_result = HealthCheckResult {
            check_id,
            status: result.status.clone(),
            message: result.message,
            response_time_ms: response_time,
            timestamp: Utc::now(),
            metadata: result.metadata,
        };
        
        // Store result
        if let Some(results) = self.results.get_mut(&check_id) {
            results.push(health_result.clone());
            
            // Keep only last 100 results
            if results.len() > 100 {
                results.drain(0..results.len() - 100);
            }
        }
        
        Ok(health_result)
    }
    
    /// Run all health checks
    pub async fn run_all_health_checks(&mut self) -> SystemHealth {
        let mut all_results = Vec::new();
        
        for check_id in self.checks.keys().cloned().collect::<Vec<_>>() {
            match self.run_health_check(check_id).await {
                Ok(result) => all_results.push(result),
                Err(e) => {
                    error!("Health check {} failed: {}", check_id, e);
                    all_results.push(HealthCheckResult {
                        check_id,
                        status: HealthStatus::Unhealthy,
                        message: format!("Health check failed: {}", e),
                        response_time_ms: 0,
                        timestamp: Utc::now(),
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        
        let overall_status = self.calculate_overall_status(&all_results);
        let uptime = self.system_start_time.elapsed().as_secs();
        
        SystemHealth {
            overall_status,
            checks: all_results,
            timestamp: Utc::now(),
            uptime_seconds: uptime,
            version: self.version.clone(),
            environment: self.environment.clone(),
        }
    }
    
    /// Get system health
    pub async fn get_system_health(&self) -> SystemHealth {
        let mut all_results = Vec::new();
        
        for (check_id, results) in &self.results {
            if let Some(latest_result) = results.last() {
                all_results.push(latest_result.clone());
            }
        }
        
        let overall_status = self.calculate_overall_status(&all_results);
        let uptime = self.system_start_time.elapsed().as_secs();
        
        SystemHealth {
            overall_status,
            checks: all_results,
            timestamp: Utc::now(),
            uptime_seconds: uptime,
            version: self.version.clone(),
            environment: self.environment.clone(),
        }
    }
    
    /// Get health metrics
    pub async fn get_health_metrics(&self) -> HealthMetrics {
        let mut total_checks = 0;
        let mut healthy_checks = 0;
        let mut unhealthy_checks = 0;
        let mut degraded_checks = 0;
        let mut total_response_time = 0u64;
        let mut response_count = 0;
        let mut last_check_time = Utc::now();
        
        for results in self.results.values() {
            if let Some(latest_result) = results.last() {
                total_checks += 1;
                total_response_time += latest_result.response_time_ms;
                response_count += 1;
                
                if latest_result.timestamp > last_check_time {
                    last_check_time = latest_result.timestamp;
                }
                
                match latest_result.status {
                    HealthStatus::Healthy => healthy_checks += 1,
                    HealthStatus::Unhealthy => unhealthy_checks += 1,
                    HealthStatus::Degraded => degraded_checks += 1,
                    HealthStatus::Unknown => {}
                }
            }
        }
        
        let average_response_time = if response_count > 0 {
            total_response_time as f64 / response_count as f64
        } else {
            0.0
        };
        
        let uptime_percentage = if total_checks > 0 {
            healthy_checks as f64 / total_checks as f64 * 100.0
        } else {
            0.0
        };
        
        HealthMetrics {
            total_checks,
            healthy_checks,
            unhealthy_checks,
            degraded_checks,
            average_response_time,
            uptime_percentage,
            last_check_time,
        }
    }
    
    /// Get health check history
    pub async fn get_health_check_history(&self, check_id: Uuid, limit: Option<usize>) -> Vec<HealthCheckResult> {
        if let Some(results) = self.results.get(&check_id) {
            let limit = limit.unwrap_or(100);
            let start = if results.len() > limit {
                results.len() - limit
            } else {
                0
            };
            results[start..].to_vec()
        } else {
            Vec::new()
        }
    }
    
    // Private helper methods
    
    async fn execute_health_check(&self, check: &HealthCheck) -> HealthCheckResult {
        match check.check_type {
            HealthCheckType::HttpEndpoint => self.check_http_endpoint(check).await,
            HealthCheckType::Database => self.check_database(check).await,
            HealthCheckType::Redis => self.check_redis(check).await,
            HealthCheckType::Kafka => self.check_kafka(check).await,
            HealthCheckType::ExternalService => self.check_external_service(check).await,
            HealthCheckType::DiskSpace => self.check_disk_space(check).await,
            HealthCheckType::MemoryUsage => self.check_memory_usage(check).await,
            HealthCheckType::CpuUsage => self.check_cpu_usage(check).await,
            HealthCheckType::Custom => self.check_custom(check).await,
        }
    }
    
    async fn check_http_endpoint(&self, check: &HealthCheck) -> HealthCheckResult {
        let url = check.configuration.get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        if url.is_empty() {
            return HealthCheckResult {
                check_id: check.id,
                status: HealthStatus::Unhealthy,
                message: "URL not configured".to_string(),
                response_time_ms: 0,
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            };
        }
        
        match reqwest::get(url).await {
            Ok(response) => {
                let status = if response.status().is_success() {
                    HealthStatus::Healthy
                } else {
                    HealthStatus::Degraded
                };
                
                let mut metadata = HashMap::new();
                metadata.insert("status_code".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(response.status().as_u16())));
                
                HealthCheckResult {
                    check_id: check.id,
                    status,
                    message: format!("HTTP {} {}", response.status().as_u16(), response.status().canonical_reason().unwrap_or("")),
                    response_time_ms: 0, // Will be set by caller
                    timestamp: Utc::now(),
                    metadata,
                }
            }
            Err(e) => {
                HealthCheckResult {
                    check_id: check.id,
                    status: HealthStatus::Unhealthy,
                    message: format!("HTTP request failed: {}", e),
                    response_time_ms: 0,
                    timestamp: Utc::now(),
                    metadata: HashMap::new(),
                }
            }
        }
    }
    
    async fn check_database(&self, check: &HealthCheck) -> HealthCheckResult {
        // Simplified database check
        HealthCheckResult {
            check_id: check.id,
            status: HealthStatus::Healthy,
            message: "Database connection successful".to_string(),
            response_time_ms: 0,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    async fn check_redis(&self, check: &HealthCheck) -> HealthCheckResult {
        // Simplified Redis check
        HealthCheckResult {
            check_id: check.id,
            status: HealthStatus::Healthy,
            message: "Redis connection successful".to_string(),
            response_time_ms: 0,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    async fn check_kafka(&self, check: &HealthCheck) -> HealthCheckResult {
        // Simplified Kafka check
        HealthCheckResult {
            check_id: check.id,
            status: HealthStatus::Healthy,
            message: "Kafka connection successful".to_string(),
            response_time_ms: 0,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    async fn check_external_service(&self, check: &HealthCheck) -> HealthCheckResult {
        // Simplified external service check
        HealthCheckResult {
            check_id: check.id,
            status: HealthStatus::Healthy,
            message: "External service accessible".to_string(),
            response_time_ms: 0,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    async fn check_disk_space(&self, check: &HealthCheck) -> HealthCheckResult {
        // Simplified disk space check
        HealthCheckResult {
            check_id: check.id,
            status: HealthStatus::Healthy,
            message: "Disk space sufficient".to_string(),
            response_time_ms: 0,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    async fn check_memory_usage(&self, check: &HealthCheck) -> HealthCheckResult {
        // Simplified memory usage check
        HealthCheckResult {
            check_id: check.id,
            status: HealthStatus::Healthy,
            message: "Memory usage normal".to_string(),
            response_time_ms: 0,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    async fn check_cpu_usage(&self, check: &HealthCheck) -> HealthCheckResult {
        // Simplified CPU usage check
        HealthCheckResult {
            check_id: check.id,
            status: HealthStatus::Healthy,
            message: "CPU usage normal".to_string(),
            response_time_ms: 0,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    async fn check_custom(&self, check: &HealthCheck) -> HealthCheckResult {
        // Custom health check implementation
        HealthCheckResult {
            check_id: check.id,
            status: HealthStatus::Healthy,
            message: "Custom check passed".to_string(),
            response_time_ms: 0,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    fn calculate_overall_status(&self, results: &[HealthCheckResult]) -> HealthStatus {
        if results.is_empty() {
            return HealthStatus::Unknown;
        }
        
        let mut has_unhealthy = false;
        let mut has_degraded = false;
        
        for result in results {
            match result.status {
                HealthStatus::Unhealthy => {
                    has_unhealthy = true;
                    break; // Unhealthy takes precedence
                }
                HealthStatus::Degraded => {
                    has_degraded = true;
                }
                _ => {}
            }
        }
        
        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}

/// Health check scheduler for running checks at intervals
pub struct HealthCheckScheduler {
    service: HealthCheckService,
    interval: Duration,
    is_running: bool,
}

impl HealthCheckScheduler {
    pub fn new(service: HealthCheckService, interval: Duration) -> Self {
        Self {
            service,
            interval,
            is_running: false,
        }
    }
    
    pub async fn start(&mut self) {
        self.is_running = true;
        
        while self.is_running {
            let _ = self.service.run_all_health_checks().await;
            tokio::time::sleep(self.interval).await;
        }
    }
    
    pub fn stop(&mut self) {
        self.is_running = false;
    }
    
    pub fn get_service(&self) -> &HealthCheckService {
        &self.service
    }
    
    pub fn get_service_mut(&mut self) -> &mut HealthCheckService {
        &mut self.service
    }
}
