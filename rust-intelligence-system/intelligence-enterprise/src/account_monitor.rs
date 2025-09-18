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
pub struct AccountStatus {
    pub account_id: IntelligenceId,
    pub provider: String,
    pub api_key_hash: String,
    pub status: AccountState,
    pub rate_limits: RateLimitStatus,
    pub usage_metrics: UsageMetrics,
    pub health_score: f64,
    pub last_checked: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountState {
    Active,
    RateLimited,
    Suspended,
    Expired,
    Invalid,
    Maintenance,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub current_minute_usage: u32,
    pub current_hour_usage: u32,
    pub current_day_usage: u32,
    pub reset_time_minute: DateTime<Utc>,
    pub reset_time_hour: DateTime<Utc>,
    pub reset_time_day: DateTime<Utc>,
    pub is_limited: bool,
    pub limit_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub last_request_time: Option<DateTime<Utc>>,
    pub error_rate: f64,
    pub cost_per_request: f64,
    pub total_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountAlert {
    pub alert_id: IntelligenceId,
    pub account_id: IntelligenceId,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub threshold_value: f64,
    pub current_value: f64,
    pub created_at: DateTime<Utc>,
    pub is_resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    RateLimitApproaching,
    RateLimitExceeded,
    HighErrorRate,
    AccountSuspended,
    CostThresholdExceeded,
    ResponseTimeHigh,
    AccountExpiring,
    UnusualActivity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountMonitor {
    pub accounts: Arc<RwLock<HashMap<IntelligenceId, AccountStatus>>>,
    pub alerts: Arc<RwLock<Vec<AccountAlert>>>,
    pub monitoring_config: MonitoringConfig,
    pub is_running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub check_interval_seconds: u64,
    pub rate_limit_warning_threshold: f64,
    pub error_rate_threshold: f64,
    pub response_time_threshold_ms: f64,
    pub cost_threshold: f64,
    pub health_check_timeout_seconds: u64,
    pub max_retries: u32,
    pub alert_cooldown_minutes: u64,
}

impl AccountMonitor {
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            monitoring_config: config,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn add_account(&self, account: AccountStatus) -> IntelligenceResult<()> {
        let mut accounts = self.accounts.write().await;
        accounts.insert(account.account_id.clone(), account);
        info!("Added account to monitoring: {}", account.account_id);
        Ok(())
    }

    pub async fn start_monitoring(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = true;
        drop(is_running);

        let accounts = self.accounts.clone();
        let alerts = self.alerts.clone();
        let config = self.monitoring_config.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            while *is_running.read().await {
                let accounts_guard = accounts.read().await;
                let account_ids: Vec<IntelligenceId> = accounts_guard.keys().cloned().collect();
                drop(accounts_guard);

                for account_id in account_ids {
                    if let Err(e) = Self::check_account_health(
                        &accounts,
                        &alerts,
                        &config,
                        &account_id,
                    ).await {
                        error!("Failed to check account health: {}", e);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(config.check_interval_seconds)).await;
            }
        });

        info!("Account monitoring started");
        Ok(())
    }

    pub async fn stop_monitoring(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("Account monitoring stopped");
        Ok(())
    }

    pub async fn get_account_status(&self, account_id: &IntelligenceId) -> IntelligenceResult<AccountStatus> {
        let accounts = self.accounts.read().await;
        accounts.get(account_id)
            .cloned()
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "account".to_string(),
                id: account_id.clone(),
            })
    }

    pub async fn get_all_accounts(&self) -> IntelligenceResult<Vec<AccountStatus>> {
        let accounts = self.accounts.read().await;
        Ok(accounts.values().cloned().collect())
    }

    pub async fn get_active_alerts(&self) -> IntelligenceResult<Vec<AccountAlert>> {
        let alerts = self.alerts.read().await;
        Ok(alerts.iter()
            .filter(|alert| !alert.is_resolved)
            .cloned()
            .collect())
    }

    pub async fn get_account_health_summary(&self) -> IntelligenceResult<AccountHealthSummary> {
        let accounts = self.accounts.read().await;
        let alerts = self.alerts.read().await;

        let total_accounts = accounts.len();
        let active_accounts = accounts.values()
            .filter(|account| account.status == AccountState::Active)
            .count();
        let rate_limited_accounts = accounts.values()
            .filter(|account| account.status == AccountState::RateLimited)
            .count();
        let suspended_accounts = accounts.values()
            .filter(|account| account.status == AccountState::Suspended)
            .count();

        let active_alerts = alerts.iter()
            .filter(|alert| !alert.is_resolved)
            .count();
        let critical_alerts = alerts.iter()
            .filter(|alert| !alert.is_resolved && alert.severity == AlertSeverity::Critical)
            .count();

        let average_health_score = if total_accounts > 0 {
            accounts.values()
                .map(|account| account.health_score)
                .sum::<f64>() / total_accounts as f64
        } else {
            0.0
        };

        Ok(AccountHealthSummary {
            total_accounts,
            active_accounts,
            rate_limited_accounts,
            suspended_accounts,
            active_alerts,
            critical_alerts,
            average_health_score,
            last_updated: Utc::now(),
        })
    }

    async fn check_account_health(
        accounts: &Arc<RwLock<HashMap<IntelligenceId, AccountStatus>>>,
        alerts: &Arc<RwLock<Vec<AccountAlert>>>,
        config: &MonitoringConfig,
        account_id: &IntelligenceId,
    ) -> IntelligenceResult<()> {
        let mut accounts_guard = accounts.write().await;
        let account = accounts_guard.get_mut(account_id)
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "account".to_string(),
                id: account_id.clone(),
            })?;

        let previous_status = account.status.clone();
        let previous_health_score = account.health_score;

        Self::update_account_status(account, config).await?;

        if account.status != previous_status || (account.health_score - previous_health_score).abs() > 0.1 {
            account.updated_at = Utc::now();
            Self::check_for_alerts(account, alerts, config).await?;
        }

        Ok(())
    }

    async fn update_account_status(
        account: &mut AccountStatus,
        config: &MonitoringConfig,
    ) -> IntelligenceResult<()> {
        let now = Utc::now();

        Self::update_rate_limits(account, &now).await?;
        Self::calculate_health_score(account, config).await?;
        Self::determine_account_state(account, config).await?;

        account.last_checked = now;
        Ok(())
    }

    async fn update_rate_limits(
        account: &mut AccountStatus,
        now: &DateTime<Utc>,
    ) -> IntelligenceResult<()> {
        let rate_limits = &mut account.rate_limits;

        if now >= &rate_limits.reset_time_minute {
            rate_limits.current_minute_usage = 0;
            rate_limits.reset_time_minute = *now + chrono::Duration::minutes(1);
        }

        if now >= &rate_limits.reset_time_hour {
            rate_limits.current_hour_usage = 0;
            rate_limits.reset_time_hour = *now + chrono::Duration::hours(1);
        }

        if now >= &rate_limits.reset_time_day {
            rate_limits.current_day_usage = 0;
            rate_limits.reset_time_day = *now + chrono::Duration::days(1);
        }

        rate_limits.is_limited = rate_limits.current_minute_usage >= rate_limits.requests_per_minute
            || rate_limits.current_hour_usage >= rate_limits.requests_per_hour
            || rate_limits.current_day_usage >= rate_limits.requests_per_day;

        if rate_limits.is_limited {
            rate_limits.limit_reason = Some("Rate limit exceeded".to_string());
        } else {
            rate_limits.limit_reason = None;
        }

        Ok(())
    }

    async fn calculate_health_score(
        account: &mut AccountStatus,
        config: &MonitoringConfig,
    ) -> IntelligenceResult<()> {
        let mut score = 1.0;

        if account.rate_limits.is_limited {
            score -= 0.3;
        }

        let rate_limit_usage = (account.rate_limits.current_minute_usage as f64 
            / account.rate_limits.requests_per_minute as f64).min(1.0);
        if rate_limit_usage > config.rate_limit_warning_threshold {
            score -= 0.2 * (rate_limit_usage - config.rate_limit_warning_threshold);
        }

        if account.usage_metrics.error_rate > config.error_rate_threshold {
            score -= 0.3 * (account.usage_metrics.error_rate - config.error_rate_threshold);
        }

        if account.usage_metrics.average_response_time_ms > config.response_time_threshold_ms {
            score -= 0.2 * ((account.usage_metrics.average_response_time_ms - config.response_time_threshold_ms) 
                / config.response_time_threshold_ms).min(1.0);
        }

        account.health_score = score.max(0.0).min(1.0);
        Ok(())
    }

    async fn determine_account_state(
        account: &mut AccountStatus,
        config: &MonitoringConfig,
    ) -> IntelligenceResult<()> {
        if account.rate_limits.is_limited {
            account.status = AccountState::RateLimited;
        } else if account.usage_metrics.error_rate > 0.5 {
            account.status = AccountState::Suspended;
        } else if account.health_score < 0.3 {
            account.status = AccountState::Invalid;
        } else if account.health_score < 0.7 {
            account.status = AccountState::Maintenance;
        } else {
            account.status = AccountState::Active;
        }

        Ok(())
    }

    async fn check_for_alerts(
        account: &AccountStatus,
        alerts: &Arc<RwLock<Vec<AccountAlert>>>,
        config: &MonitoringConfig,
    ) -> IntelligenceResult<()> {
        let mut alerts_guard = alerts.write().await;
        let now = Utc::now();

        let rate_limit_usage = account.rate_limits.current_minute_usage as f64 
            / account.rate_limits.requests_per_minute as f64;

        if rate_limit_usage > config.rate_limit_warning_threshold {
            Self::create_alert_if_needed(
                &mut alerts_guard,
                account,
                AlertType::RateLimitApproaching,
                AlertSeverity::Warning,
                format!("Rate limit usage at {:.1}%", rate_limit_usage * 100.0),
                rate_limit_usage,
                rate_limit_usage,
                config,
            ).await?;
        }

        if account.rate_limits.is_limited {
            Self::create_alert_if_needed(
                &mut alerts_guard,
                account,
                AlertType::RateLimitExceeded,
                AlertSeverity::Critical,
                "Rate limit exceeded".to_string(),
                1.0,
                1.0,
                config,
            ).await?;
        }

        if account.usage_metrics.error_rate > config.error_rate_threshold {
            Self::create_alert_if_needed(
                &mut alerts_guard,
                account,
                AlertType::HighErrorRate,
                AlertSeverity::Warning,
                format!("High error rate: {:.1}%", account.usage_metrics.error_rate * 100.0),
                config.error_rate_threshold,
                account.usage_metrics.error_rate,
                config,
            ).await?;
        }

        if account.status == AccountState::Suspended {
            Self::create_alert_if_needed(
                &mut alerts_guard,
                account,
                AlertType::AccountSuspended,
                AlertSeverity::Critical,
                "Account has been suspended".to_string(),
                0.0,
                1.0,
                config,
            ).await?;
        }

        if account.usage_metrics.total_cost > config.cost_threshold {
            Self::create_alert_if_needed(
                &mut alerts_guard,
                account,
                AlertType::CostThresholdExceeded,
                AlertSeverity::Warning,
                format!("Cost threshold exceeded: ${:.2}", account.usage_metrics.total_cost),
                config.cost_threshold,
                account.usage_metrics.total_cost,
                config,
            ).await?;
        }

        Ok(())
    }

    async fn create_alert_if_needed(
        alerts: &mut Vec<AccountAlert>,
        account: &AccountStatus,
        alert_type: AlertType,
        severity: AlertSeverity,
        message: String,
        threshold_value: f64,
        current_value: f64,
        config: &MonitoringConfig,
    ) -> IntelligenceResult<()> {
        let cooldown_time = now - chrono::Duration::minutes(config.alert_cooldown_minutes as i64);
        
        let recent_alert_exists = alerts.iter().any(|alert| {
            alert.account_id == account.account_id
                && alert.alert_type == alert_type
                && !alert.is_resolved
                && alert.created_at > cooldown_time
        });

        if !recent_alert_exists {
            let alert = AccountAlert {
                alert_id: IntelligenceId::new(),
                account_id: account.account_id.clone(),
                alert_type,
                severity,
                message,
                threshold_value,
                current_value,
                created_at: now,
                is_resolved: false,
                resolved_at: None,
            };

            alerts.push(alert);
            info!("Created alert for account {}: {}", account.account_id, message);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountHealthSummary {
    pub total_accounts: usize,
    pub active_accounts: usize,
    pub rate_limited_accounts: usize,
    pub suspended_accounts: usize,
    pub active_alerts: usize,
    pub critical_alerts: usize,
    pub average_health_score: f64,
    pub last_updated: DateTime<Utc>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 60,
            rate_limit_warning_threshold: 0.8,
            error_rate_threshold: 0.1,
            response_time_threshold_ms: 5000.0,
            cost_threshold: 1000.0,
            health_check_timeout_seconds: 30,
            max_retries: 3,
            alert_cooldown_minutes: 15,
        }
    }
}
