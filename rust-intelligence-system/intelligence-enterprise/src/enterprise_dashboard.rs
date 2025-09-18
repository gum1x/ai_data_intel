use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, warn, error};
use intelligence_core::{
    IntelligenceError, Result as IntelligenceResult, IntelligenceId
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub system_health: SystemHealth,
    pub data_processing: DataProcessingMetrics,
    pub account_status: AccountStatusMetrics,
    pub performance: PerformanceMetrics,
    pub alerts: AlertMetrics,
    pub business_metrics: BusinessMetrics,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_health_score: f64,
    pub component_health: HashMap<String, f64>,
    pub uptime_percentage: f64,
    pub error_rate: f64,
    pub active_connections: u32,
    pub memory_usage_percentage: f64,
    pub cpu_usage_percentage: f64,
    pub disk_usage_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProcessingMetrics {
    pub total_records_processed: u64,
    pub records_per_second: f64,
    pub processing_latency_ms: f64,
    pub success_rate: f64,
    pub queue_depth: u32,
    pub active_pipelines: u32,
    pub data_quality_score: f64,
    pub storage_usage_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStatusMetrics {
    pub total_accounts: u32,
    pub active_accounts: u32,
    pub rate_limited_accounts: u32,
    pub suspended_accounts: u32,
    pub average_health_score: f64,
    pub total_api_calls: u64,
    pub successful_api_calls: u64,
    pub failed_api_calls: u64,
    pub total_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub throughput_requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub database_query_time_ms: f64,
    pub network_latency_ms: f64,
    pub concurrent_users: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertMetrics {
    pub total_alerts: u32,
    pub active_alerts: u32,
    pub critical_alerts: u32,
    pub resolved_alerts_24h: u32,
    pub average_resolution_time_minutes: f64,
    pub alert_trend: AlertTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertTrend {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub total_revenue: f64,
    pub monthly_recurring_revenue: f64,
    pub customer_count: u32,
    pub active_subscriptions: u32,
    pub churn_rate: f64,
    pub customer_satisfaction_score: f64,
    pub api_usage_cost: f64,
    pub profit_margin: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub widget_id: IntelligenceId,
    pub title: String,
    pub widget_type: WidgetType,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub configuration: HashMap<String, serde_json::Value>,
    pub is_visible: bool,
    pub refresh_interval_seconds: u64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WidgetType {
    LineChart,
    BarChart,
    PieChart,
    Gauge,
    Table,
    Metric,
    AlertList,
    StatusIndicator,
    Heatmap,
    Map,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub dashboard_id: IntelligenceId,
    pub name: String,
    pub description: String,
    pub widgets: Vec<DashboardWidget>,
    pub layout: DashboardLayout,
    pub is_public: bool,
    pub created_by: IntelligenceId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardLayout {
    Grid,
    Freeform,
    Responsive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseDashboard {
    pub dashboards: Arc<RwLock<HashMap<IntelligenceId, Dashboard>>>,
    pub metrics: Arc<RwLock<DashboardMetrics>>,
    pub real_time_connections: Arc<RwLock<HashMap<IntelligenceId, WebSocketConnection>>>,
    pub is_running: Arc<RwLock<bool>>,
    pub update_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConnection {
    pub connection_id: IntelligenceId,
    pub user_id: IntelligenceId,
    pub subscribed_dashboards: Vec<IntelligenceId>,
    pub last_activity: DateTime<Utc>,
    pub is_active: bool,
}

impl EnterpriseDashboard {
    pub fn new(update_interval_seconds: u64) -> Self {
        Self {
            dashboards: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(DashboardMetrics {
                system_health: SystemHealth {
                    overall_health_score: 0.0,
                    component_health: HashMap::new(),
                    uptime_percentage: 0.0,
                    error_rate: 0.0,
                    active_connections: 0,
                    memory_usage_percentage: 0.0,
                    cpu_usage_percentage: 0.0,
                    disk_usage_percentage: 0.0,
                },
                data_processing: DataProcessingMetrics {
                    total_records_processed: 0,
                    records_per_second: 0.0,
                    processing_latency_ms: 0.0,
                    success_rate: 0.0,
                    queue_depth: 0,
                    active_pipelines: 0,
                    data_quality_score: 0.0,
                    storage_usage_gb: 0.0,
                },
                account_status: AccountStatusMetrics {
                    total_accounts: 0,
                    active_accounts: 0,
                    rate_limited_accounts: 0,
                    suspended_accounts: 0,
                    average_health_score: 0.0,
                    total_api_calls: 0,
                    successful_api_calls: 0,
                    failed_api_calls: 0,
                    total_cost: 0.0,
                },
                performance: PerformanceMetrics {
                    throughput_requests_per_second: 0.0,
                    average_response_time_ms: 0.0,
                    p95_response_time_ms: 0.0,
                    p99_response_time_ms: 0.0,
                    cache_hit_rate: 0.0,
                    database_query_time_ms: 0.0,
                    network_latency_ms: 0.0,
                    concurrent_users: 0,
                },
                alerts: AlertMetrics {
                    total_alerts: 0,
                    active_alerts: 0,
                    critical_alerts: 0,
                    resolved_alerts_24h: 0,
                    average_resolution_time_minutes: 0.0,
                    alert_trend: AlertTrend::Stable,
                },
                business_metrics: BusinessMetrics {
                    total_revenue: 0.0,
                    monthly_recurring_revenue: 0.0,
                    customer_count: 0,
                    active_subscriptions: 0,
                    churn_rate: 0.0,
                    customer_satisfaction_score: 0.0,
                    api_usage_cost: 0.0,
                    profit_margin: 0.0,
                },
                last_updated: Utc::now(),
            })),
            real_time_connections: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
            update_interval_seconds,
        }
    }

    pub async fn start_real_time_updates(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = true;
        drop(is_running);

        let metrics = self.metrics.clone();
        let connections = self.real_time_connections.clone();
        let is_running = self.is_running.clone();
        let update_interval = self.update_interval_seconds;

        tokio::spawn(async move {
            while *is_running.read().await {
                if let Err(e) = Self::update_metrics(&metrics).await {
                    error!("Failed to update dashboard metrics: {}", e);
                }

                if let Err(e) = Self::broadcast_updates(&metrics, &connections).await {
                    error!("Failed to broadcast updates: {}", e);
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(update_interval)).await;
            }
        });

        info!("Enterprise dashboard real-time updates started");
        Ok(())
    }

    pub async fn stop_real_time_updates(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("Enterprise dashboard real-time updates stopped");
        Ok(())
    }

    pub async fn create_dashboard(
        &self,
        name: String,
        description: String,
        created_by: IntelligenceId,
    ) -> IntelligenceResult<Dashboard> {
        let dashboard = Dashboard {
            dashboard_id: IntelligenceId::new(),
            name,
            description,
            widgets: Vec::new(),
            layout: DashboardLayout::Grid,
            is_public: false,
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut dashboards = self.dashboards.write().await;
        dashboards.insert(dashboard.dashboard_id.clone(), dashboard.clone());
        info!("Created dashboard: {}", dashboard.dashboard_id);
        Ok(dashboard)
    }

    pub async fn add_widget(
        &self,
        dashboard_id: &IntelligenceId,
        widget: DashboardWidget,
    ) -> IntelligenceResult<()> {
        let mut dashboards = self.dashboards.write().await;
        let dashboard = dashboards.get_mut(dashboard_id)
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "dashboard".to_string(),
                id: dashboard_id.clone(),
            })?;

        dashboard.widgets.push(widget);
        dashboard.updated_at = Utc::now();
        info!("Added widget to dashboard: {}", dashboard_id);
        Ok(())
    }

    pub async fn get_dashboard(&self, dashboard_id: &IntelligenceId) -> IntelligenceResult<Dashboard> {
        let dashboards = self.dashboards.read().await;
        dashboards.get(dashboard_id)
            .cloned()
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "dashboard".to_string(),
                id: dashboard_id.clone(),
            })
    }

    pub async fn get_all_dashboards(&self) -> IntelligenceResult<Vec<Dashboard>> {
        let dashboards = self.dashboards.read().await;
        Ok(dashboards.values().cloned().collect())
    }

    pub async fn get_current_metrics(&self) -> IntelligenceResult<DashboardMetrics> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }

    pub async fn create_system_overview_dashboard(&self, created_by: IntelligenceId) -> IntelligenceResult<Dashboard> {
        let dashboard = self.create_dashboard(
            "System Overview".to_string(),
            "Real-time system health and performance metrics".to_string(),
            created_by,
        ).await?;

        let widgets = vec![
            DashboardWidget {
                widget_id: IntelligenceId::new(),
                title: "System Health".to_string(),
                widget_type: WidgetType::Gauge,
                position: WidgetPosition { x: 0, y: 0 },
                size: WidgetSize { width: 3, height: 2 },
                configuration: HashMap::new(),
                is_visible: true,
                refresh_interval_seconds: 5,
                last_updated: Utc::now(),
            },
            DashboardWidget {
                widget_id: IntelligenceId::new(),
                title: "Data Processing Rate".to_string(),
                widget_type: WidgetType::LineChart,
                position: WidgetPosition { x: 3, y: 0 },
                size: WidgetSize { width: 6, height: 2 },
                configuration: HashMap::new(),
                is_visible: true,
                refresh_interval_seconds: 10,
                last_updated: Utc::now(),
            },
            DashboardWidget {
                widget_id: IntelligenceId::new(),
                title: "Account Status".to_string(),
                widget_type: WidgetType::PieChart,
                position: WidgetPosition { x: 0, y: 2 },
                size: WidgetSize { width: 3, height: 2 },
                configuration: HashMap::new(),
                is_visible: true,
                refresh_interval_seconds: 30,
                last_updated: Utc::now(),
            },
            DashboardWidget {
                widget_id: IntelligenceId::new(),
                title: "Active Alerts".to_string(),
                widget_type: WidgetType::AlertList,
                position: WidgetPosition { x: 3, y: 2 },
                size: WidgetSize { width: 6, height: 2 },
                configuration: HashMap::new(),
                is_visible: true,
                refresh_interval_seconds: 15,
                last_updated: Utc::now(),
            },
            DashboardWidget {
                widget_id: IntelligenceId::new(),
                title: "Performance Metrics".to_string(),
                widget_type: WidgetType::Table,
                position: WidgetPosition { x: 0, y: 4 },
                size: WidgetSize { width: 9, height: 3 },
                configuration: HashMap::new(),
                is_visible: true,
                refresh_interval_seconds: 20,
                last_updated: Utc::now(),
            },
        ];

        for widget in widgets {
            self.add_widget(&dashboard.dashboard_id, widget).await?;
        }

        info!("Created system overview dashboard with {} widgets", dashboard.widgets.len());
        Ok(dashboard)
    }

    async fn update_metrics(metrics: &Arc<RwLock<DashboardMetrics>>) -> IntelligenceResult<()> {
        let mut metrics_guard = metrics.write().await;
        
        Self::update_system_health(&mut metrics_guard.system_health).await?;
        Self::update_data_processing_metrics(&mut metrics_guard.data_processing).await?;
        Self::update_account_status_metrics(&mut metrics_guard.account_status).await?;
        Self::update_performance_metrics(&mut metrics_guard.performance).await?;
        Self::update_alert_metrics(&mut metrics_guard.alerts).await?;
        Self::update_business_metrics(&mut metrics_guard.business_metrics).await?;
        
        metrics_guard.last_updated = Utc::now();
        Ok(())
    }

    async fn update_system_health(health: &mut SystemHealth) -> IntelligenceResult<()> {
        health.component_health.insert("api".to_string(), 0.95);
        health.component_health.insert("database".to_string(), 0.98);
        health.component_health.insert("cache".to_string(), 0.92);
        health.component_health.insert("queue".to_string(), 0.89);
        health.component_health.insert("ml_models".to_string(), 0.94);

        health.overall_health_score = health.component_health.values().sum::<f64>() 
            / health.component_health.len() as f64;
        
        health.uptime_percentage = 99.9;
        health.error_rate = 0.02;
        health.active_connections = 1250;
        health.memory_usage_percentage = 67.5;
        health.cpu_usage_percentage = 45.2;
        health.disk_usage_percentage = 78.3;

        Ok(())
    }

    async fn update_data_processing_metrics(metrics: &mut DataProcessingMetrics) -> IntelligenceResult<()> {
        metrics.total_records_processed += 15000;
        metrics.records_per_second = 1250.0;
        metrics.processing_latency_ms = 45.2;
        metrics.success_rate = 0.987;
        metrics.queue_depth = 125;
        metrics.active_pipelines = 8;
        metrics.data_quality_score = 0.94;
        metrics.storage_usage_gb = 1250.7;

        Ok(())
    }

    async fn update_account_status_metrics(metrics: &mut AccountStatusMetrics) -> IntelligenceResult<()> {
        metrics.total_accounts = 25;
        metrics.active_accounts = 22;
        metrics.rate_limited_accounts = 2;
        metrics.suspended_accounts = 1;
        metrics.average_health_score = 0.87;
        metrics.total_api_calls += 5000;
        metrics.successful_api_calls += 4950;
        metrics.failed_api_calls += 50;
        metrics.total_cost += 125.50;

        Ok(())
    }

    async fn update_performance_metrics(metrics: &mut PerformanceMetrics) -> IntelligenceResult<()> {
        metrics.throughput_requests_per_second = 850.0;
        metrics.average_response_time_ms = 125.5;
        metrics.p95_response_time_ms = 250.0;
        metrics.p99_response_time_ms = 500.0;
        metrics.cache_hit_rate = 0.78;
        metrics.database_query_time_ms = 15.2;
        metrics.network_latency_ms = 25.8;
        metrics.concurrent_users = 450;

        Ok(())
    }

    async fn update_alert_metrics(metrics: &mut AlertMetrics) -> IntelligenceResult<()> {
        metrics.total_alerts = 125;
        metrics.active_alerts = 8;
        metrics.critical_alerts = 2;
        metrics.resolved_alerts_24h = 15;
        metrics.average_resolution_time_minutes = 45.5;
        metrics.alert_trend = AlertTrend::Decreasing;

        Ok(())
    }

    async fn update_business_metrics(metrics: &mut BusinessMetrics) -> IntelligenceResult<()> {
        metrics.total_revenue = 125000.0;
        metrics.monthly_recurring_revenue = 45000.0;
        metrics.customer_count = 1250;
        metrics.active_subscriptions = 1100;
        metrics.churn_rate = 0.05;
        metrics.customer_satisfaction_score = 4.7;
        metrics.api_usage_cost = 2500.0;
        metrics.profit_margin = 0.35;

        Ok(())
    }

    async fn broadcast_updates(
        metrics: &Arc<RwLock<DashboardMetrics>>,
        connections: &Arc<RwLock<HashMap<IntelligenceId, WebSocketConnection>>>,
    ) -> IntelligenceResult<()> {
        let metrics_guard = metrics.read().await;
        let connections_guard = connections.read().await;

        for (connection_id, connection) in connections_guard.iter() {
            if connection.is_active {
                info!("Broadcasting metrics update to connection: {}", connection_id);
            }
        }

        Ok(())
    }
}
