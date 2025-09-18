use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_agents: u32,
    pub tasks_processed: u64,
    pub data_points_collected: u64,
    pub threat_detections: u64,
    pub error_count: u64,
    pub response_times: HashMap<String, Duration>,
    pub throughput: HashMap<String, u64>,
}
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operation: String,
    pub duration: Duration,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
#[derive(Debug, Clone)]
pub struct BusinessMetrics {
    pub timestamp: DateTime<Utc>,
    pub active_users: u32,
    pub data_quality_score: f64,
    pub threat_detection_rate: f64,
    pub false_positive_rate: f64,
    pub system_uptime: Duration,
    pub sla_compliance: f64,
}
pub struct MetricsCollector {
    metrics: Arc<RwLock<SystemMetrics>>,
    performance_history: Arc<RwLock<Vec<PerformanceMetrics>>>,
    business_metrics: Arc<RwLock<BusinessMetrics>>,
    counters: Arc<RwLock<HashMap<String, u64>>>,
    gauges: Arc<RwLock<HashMap<String, f64>>>,
    histograms: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
}
impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(SystemMetrics {
                timestamp: Utc::now(),
                cpu_usage: 0.0,
                memory_usage: 0.0,
                active_agents: 0,
                tasks_processed: 0,
                data_points_collected: 0,
                threat_detections: 0,
                error_count: 0,
                response_times: HashMap::new(),
                throughput: HashMap::new(),
            })),
            performance_history: Arc::new(RwLock::new(Vec::new())),
            business_metrics: Arc::new(RwLock::new(BusinessMetrics {
                timestamp: Utc::now(),
                active_users: 0,
                data_quality_score: 0.0,
                threat_detection_rate: 0.0,
                false_positive_rate: 0.0,
                system_uptime: Duration::from_secs(0),
                sla_compliance: 0.0,
            })),
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub async fn increment_counter(&self, name: &str, value: u64) {
        let mut counters = self.counters.write().await;
        *counters.entry(name.to_string()).or_insert(0) += value;
    }
    pub async fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.write().await;
        gauges.insert(name.to_string(), value);
    }
    pub async fn record_duration(&self, name: &str, duration: Duration) {
        let mut histograms = self.histograms.write().await;
        histograms.entry(name.to_string()).or_insert_with(Vec::new).push(duration);
        if let Some(measurements) = histograms.get_mut(name) {
            if measurements.len() > 1000 {
                measurements.drain(0..measurements.len() - 1000);
            }
        }
    }
    pub async fn record_performance(&self, operation: String, duration: Duration, success: bool, metadata: HashMap<String, String>) {
        let performance = PerformanceMetrics {
            operation,
            duration,
            success,
            timestamp: Utc::now(),
            metadata,
        };
        let mut history = self.performance_history.write().await;
        history.push(performance);
        if history.len() > 10000 {
            history.drain(0..history.len() - 10000);
        }
    }
    pub async fn update_system_metrics(&self, metrics: SystemMetrics) {
        let mut current_metrics = self.metrics.write().await;
        *current_metrics = metrics;
    }
    pub async fn update_business_metrics(&self, metrics: BusinessMetrics) {
        let mut current_metrics = self.business_metrics.write().await;
        *current_metrics = metrics;
    }
    pub async fn get_system_metrics(&self) -> SystemMetrics {
        self.metrics.read().await.clone()
    }
    pub async fn get_business_metrics(&self) -> BusinessMetrics {
        self.business_metrics.read().await.clone()
    }
    pub async fn get_counters(&self) -> HashMap<String, u64> {
        self.counters.read().await.clone()
    }
    pub async fn get_gauges(&self) -> HashMap<String, f64> {
        self.gauges.read().await.clone()
    }
    pub async fn get_performance_summary(&self, operation: &str) -> Option<PerformanceSummary> {
        let history = self.performance_history.read().await;
        let operation_metrics: Vec<&PerformanceMetrics> = history
            .iter()
            .filter(|m| m.operation == operation)
            .collect();
        if operation_metrics.is_empty() {
            return None;
        }
        let total_count = operation_metrics.len();
        let success_count = operation_metrics.iter().filter(|m| m.success).count();
        let success_rate = success_count as f64 / total_count as f64;
        let durations: Vec<Duration> = operation_metrics.iter().map(|m| m.duration).collect();
        let avg_duration = durations.iter().sum::<Duration>() / total_count as u32;
        let min_duration = durations.iter().min().copied().unwrap_or_default();
        let max_duration = durations.iter().max().copied().unwrap_or_default();
        Some(PerformanceSummary {
            operation: operation.to_string(),
            total_count,
            success_count,
            success_rate,
            avg_duration,
            min_duration,
            max_duration,
        })
    }
    pub async fn get_histogram_summary(&self, name: &str) -> Option<HistogramSummary> {
        let histograms = self.histograms.read().await;
        let measurements = histograms.get(name)?;
        if measurements.is_empty() {
            return None;
        }
        let mut sorted_measurements = measurements.clone();
        sorted_measurements.sort();
        let count = sorted_measurements.len();
        let sum: Duration = sorted_measurements.iter().sum();
        let avg = sum / count as u32;
        let min = sorted_measurements[0];
        let max = sorted_measurements[count - 1];
        let p50_idx = (count as f64 * 0.5) as usize;
        let p95_idx = (count as f64 * 0.95) as usize;
        let p99_idx = (count as f64 * 0.99) as usize;
        let p50 = sorted_measurements[p50_idx.min(count - 1)];
        let p95 = sorted_measurements[p95_idx.min(count - 1)];
        let p99 = sorted_measurements[p99_idx.min(count - 1)];
        Some(HistogramSummary {
            name: name.to_string(),
            count,
            sum,
            avg,
            min,
            max,
            p50,
            p95,
            p99,
        })
    }
}
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub operation: String,
    pub total_count: usize,
    pub success_count: usize,
    pub success_rate: f64,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
}
#[derive(Debug, Clone)]
pub struct HistogramSummary {
    pub name: String,
    pub count: usize,
    pub sum: Duration,
    pub avg: Duration,
    pub min: Duration,
    pub max: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
}
pub struct PerformanceTimer {
    start: Instant,
    operation: String,
    collector: Arc<MetricsCollector>,
}
impl PerformanceTimer {
    pub fn new(operation: String, collector: Arc<MetricsCollector>) -> Self {
        Self {
            start: Instant::now(),
            operation,
            collector,
        }
    }
    pub fn finish(self, success: bool, metadata: HashMap<String, String>) {
        let duration = self.start.elapsed();
        let collector = self.collector.clone();
        let operation = self.operation.clone();
        tokio::spawn(async move {
            collector.record_performance(operation, duration, success, metadata).await;
        });
    }
}
impl Drop for PerformanceTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let collector = self.collector.clone();
        let operation = self.operation.clone();
        tokio::spawn(async move {
            collector.record_duration(&operation, duration).await;
        });
    }
}
pub struct PrometheusExporter {
    collector: Arc<MetricsCollector>,
}
impl PrometheusExporter {
    pub fn new(collector: Arc<MetricsCollector>) -> Self {
        Self { collector }
    }
    pub async fn export_metrics(&self) -> String {
        let mut output = String::new();
        let counters = self.collector.get_counters().await;
        for (name, value) in counters {
            output.push_str(&format!("intelligence_counter_{} {}\n", name, value));
        }
        let gauges = self.collector.get_gauges().await;
        for (name, value) in gauges {
            output.push_str(&format!("intelligence_gauge_{} {}\n", name, value));
        }
        let system_metrics = self.collector.get_system_metrics().await;
        output.push_str(&format!("intelligence_system_cpu_usage {}\n", system_metrics.cpu_usage));
        output.push_str(&format!("intelligence_system_memory_usage {}\n", system_metrics.memory_usage));
        output.push_str(&format!("intelligence_system_active_agents {}\n", system_metrics.active_agents));
        output.push_str(&format!("intelligence_system_tasks_processed {}\n", system_metrics.tasks_processed));
        output.push_str(&format!("intelligence_system_data_points_collected {}\n", system_metrics.data_points_collected));
        output.push_str(&format!("intelligence_system_threat_detections {}\n", system_metrics.threat_detections));
        output.push_str(&format!("intelligence_system_error_count {}\n", system_metrics.error_count));
        let business_metrics = self.collector.get_business_metrics().await;
        output.push_str(&format!("intelligence_business_active_users {}\n", business_metrics.active_users));
        output.push_str(&format!("intelligence_business_data_quality_score {}\n", business_metrics.data_quality_score));
        output.push_str(&format!("intelligence_business_threat_detection_rate {}\n", business_metrics.threat_detection_rate));
        output.push_str(&format!("intelligence_business_false_positive_rate {}\n", business_metrics.false_positive_rate));
        output.push_str(&format!("intelligence_business_sla_compliance {}\n", business_metrics.sla_compliance));
        output
    }
}
