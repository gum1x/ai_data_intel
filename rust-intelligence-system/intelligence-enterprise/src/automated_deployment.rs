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
pub struct DeploymentConfig {
    pub config_id: IntelligenceId,
    pub name: String,
    pub description: String,
    pub environment: Environment,
    pub deployment_strategy: DeploymentStrategy,
    pub scaling_config: ScalingConfig,
    pub health_check_config: HealthCheckConfig,
    pub resource_limits: ResourceLimits,
    pub secrets: HashMap<String, String>,
    pub environment_variables: HashMap<String, String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
    Testing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeploymentStrategy {
    Rolling,
    BlueGreen,
    Canary,
    Recreate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_utilization: f64,
    pub target_memory_utilization: f64,
    pub scale_up_cooldown_seconds: u64,
    pub scale_down_cooldown_seconds: u64,
    pub custom_metrics: Vec<CustomMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetric {
    pub metric_name: String,
    pub target_value: f64,
    pub metric_type: MetricType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricType {
    AverageValue,
    Value,
    Utilization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub initial_delay_seconds: u64,
    pub period_seconds: u64,
    pub timeout_seconds: u64,
    pub success_threshold: u32,
    pub failure_threshold: u32,
    pub health_check_path: String,
    pub health_check_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_limit: String,
    pub memory_limit: String,
    pub storage_limit: String,
    pub network_bandwidth_limit: String,
    pub gpu_limit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub deployment_id: IntelligenceId,
    pub config_id: IntelligenceId,
    pub version: String,
    pub status: DeploymentStatus,
    pub environment: Environment,
    pub replicas: u32,
    pub ready_replicas: u32,
    pub available_replicas: u32,
    pub unavailable_replicas: u32,
    pub deployment_strategy: DeploymentStrategy,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeploymentStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    RollingBack,
    RolledBack,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentPipeline {
    pub pipeline_id: IntelligenceId,
    pub name: String,
    pub description: String,
    pub stages: Vec<PipelineStage>,
    pub triggers: Vec<Trigger>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub stage_id: IntelligenceId,
    pub name: String,
    pub stage_type: StageType,
    pub order: u32,
    pub configuration: HashMap<String, serde_json::Value>,
    pub timeout_seconds: u64,
    pub retry_count: u32,
    pub is_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StageType {
    Build,
    Test,
    SecurityScan,
    Package,
    Deploy,
    Verify,
    Rollback,
    Notify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub trigger_id: IntelligenceId,
    pub trigger_type: TriggerType,
    pub configuration: HashMap<String, serde_json::Value>,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerType {
    GitPush,
    Schedule,
    Manual,
    Webhook,
    ApiCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentExecution {
    pub execution_id: IntelligenceId,
    pub pipeline_id: IntelligenceId,
    pub deployment_id: IntelligenceId,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub stages: Vec<StageExecution>,
    pub logs: Vec<ExecutionLog>,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageExecution {
    pub stage_id: IntelligenceId,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_seconds: u64,
    pub logs: Vec<String>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLog {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub stage_id: Option<IntelligenceId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub artifact_id: IntelligenceId,
    pub name: String,
    pub artifact_type: ArtifactType,
    pub size_bytes: u64,
    pub checksum: String,
    pub storage_location: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArtifactType {
    DockerImage,
    Binary,
    Configuration,
    Documentation,
    TestReport,
    SecurityReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatedDeploymentSystem {
    pub deployment_configs: Arc<RwLock<HashMap<IntelligenceId, DeploymentConfig>>>,
    pub deployments: Arc<RwLock<HashMap<IntelligenceId, Deployment>>>,
    pub pipelines: Arc<RwLock<HashMap<IntelligenceId, DeploymentPipeline>>>,
    pub executions: Arc<RwLock<HashMap<IntelligenceId, DeploymentExecution>>>,
    pub kubernetes_client: KubernetesClient,
    pub docker_registry: DockerRegistry,
    pub is_running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesClient {
    pub cluster_url: String,
    pub namespace: String,
    pub api_token: String,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerRegistry {
    pub registry_url: String,
    pub username: String,
    pub password: String,
    pub is_authenticated: bool,
}

impl AutomatedDeploymentSystem {
    pub fn new(kubernetes_client: KubernetesClient, docker_registry: DockerRegistry) -> Self {
        Self {
            deployment_configs: Arc::new(RwLock::new(HashMap::new())),
            deployments: Arc::new(RwLock::new(HashMap::new())),
            pipelines: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(HashMap::new())),
            kubernetes_client,
            docker_registry,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn create_deployment_config(
        &self,
        name: String,
        description: String,
        environment: Environment,
        deployment_strategy: DeploymentStrategy,
        scaling_config: ScalingConfig,
        health_check_config: HealthCheckConfig,
        resource_limits: ResourceLimits,
    ) -> IntelligenceResult<DeploymentConfig> {
        let config = DeploymentConfig {
            config_id: IntelligenceId::new(),
            name,
            description,
            environment,
            deployment_strategy,
            scaling_config,
            health_check_config,
            resource_limits,
            secrets: HashMap::new(),
            environment_variables: HashMap::new(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut configs = self.deployment_configs.write().await;
        configs.insert(config.config_id.clone(), config.clone());
        info!("Created deployment config: {}", config.config_id);
        Ok(config)
    }

    pub async fn create_deployment_pipeline(
        &self,
        name: String,
        description: String,
        stages: Vec<PipelineStage>,
        triggers: Vec<Trigger>,
    ) -> IntelligenceResult<DeploymentPipeline> {
        let pipeline = DeploymentPipeline {
            pipeline_id: IntelligenceId::new(),
            name,
            description,
            stages,
            triggers,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut pipelines = self.pipelines.write().await;
        pipelines.insert(pipeline.pipeline_id.clone(), pipeline.clone());
        info!("Created deployment pipeline: {}", pipeline.pipeline_id);
        Ok(pipeline)
    }

    pub async fn deploy(
        &self,
        config_id: IntelligenceId,
        version: String,
        pipeline_id: Option<IntelligenceId>,
    ) -> IntelligenceResult<Deployment> {
        let configs = self.deployment_configs.read().await;
        let config = configs.get(&config_id)
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "deployment_config".to_string(),
                id: config_id.clone(),
            })?;

        let deployment = Deployment {
            deployment_id: IntelligenceId::new(),
            config_id: config_id.clone(),
            version,
            status: DeploymentStatus::Pending,
            environment: config.environment.clone(),
            replicas: config.scaling_config.min_replicas,
            ready_replicas: 0,
            available_replicas: 0,
            unavailable_replicas: 0,
            deployment_strategy: config.deployment_strategy.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        };

        let mut deployments = self.deployments.write().await;
        deployments.insert(deployment.deployment_id.clone(), deployment.clone());

        if let Some(pipe_id) = pipeline_id {
            self.execute_pipeline(pipe_id, deployment.deployment_id.clone()).await?;
        } else {
            self.deploy_directly(deployment.deployment_id.clone()).await?;
        }

        info!("Started deployment: {}", deployment.deployment_id);
        Ok(deployment)
    }

    pub async fn execute_pipeline(
        &self,
        pipeline_id: IntelligenceId,
        deployment_id: IntelligenceId,
    ) -> IntelligenceResult<DeploymentExecution> {
        let pipelines = self.pipelines.read().await;
        let pipeline = pipelines.get(&pipeline_id)
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "deployment_pipeline".to_string(),
                id: pipeline_id.clone(),
            })?;

        let execution = DeploymentExecution {
            execution_id: IntelligenceId::new(),
            pipeline_id: pipeline_id.clone(),
            deployment_id: deployment_id.clone(),
            status: ExecutionStatus::Queued,
            started_at: Utc::now(),
            completed_at: None,
            stages: Vec::new(),
            logs: Vec::new(),
            artifacts: Vec::new(),
        };

        let mut executions = self.executions.write().await;
        executions.insert(execution.execution_id.clone(), execution.clone());

        let execution_id = execution.execution_id.clone();
        let pipeline_stages = pipeline.stages.clone();
        let executions_ref = self.executions.clone();
        let deployments_ref = self.deployments.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::run_pipeline_stages(
                &executions_ref,
                &deployments_ref,
                &execution_id,
                &pipeline_stages,
            ).await {
                error!("Pipeline execution failed: {}", e);
            }
        });

        info!("Started pipeline execution: {}", execution.execution_id);
        Ok(execution)
    }

    pub async fn deploy_directly(&self, deployment_id: IntelligenceId) -> IntelligenceResult<()> {
        let mut deployments = self.deployments.write().await;
        let deployment = deployments.get_mut(&deployment_id)
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "deployment".to_string(),
                id: deployment_id.clone(),
            })?;

        deployment.status = DeploymentStatus::InProgress;
        deployment.updated_at = Utc::now();

        let configs = self.deployment_configs.read().await;
        let config = configs.get(&deployment.config_id)
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "deployment_config".to_string(),
                id: deployment.config_id.clone(),
            })?;

        match self.kubernetes_client.deploy(&config, &deployment.version).await {
            Ok(_) => {
                deployment.status = DeploymentStatus::Completed;
                deployment.completed_at = Some(Utc::now());
                info!("Deployment completed successfully: {}", deployment_id);
            }
            Err(e) => {
                deployment.status = DeploymentStatus::Failed;
                error!("Deployment failed: {}", e);
            }
        }

        deployment.updated_at = Utc::now();
        Ok(())
    }

    pub async fn scale_deployment(
        &self,
        deployment_id: IntelligenceId,
        target_replicas: u32,
    ) -> IntelligenceResult<()> {
        let mut deployments = self.deployments.write().await;
        let deployment = deployments.get_mut(&deployment_id)
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "deployment".to_string(),
                id: deployment_id.clone(),
            })?;

        let configs = self.deployment_configs.read().await;
        let config = configs.get(&deployment.config_id)
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "deployment_config".to_string(),
                id: deployment.config_id.clone(),
            })?;

        if target_replicas < config.scaling_config.min_replicas 
            || target_replicas > config.scaling_config.max_replicas {
            return Err(IntelligenceError::InvalidOperation { 
                message: format!("Target replicas {} is outside allowed range [{}, {}]", 
                    target_replicas, 
                    config.scaling_config.min_replicas, 
                    config.scaling_config.max_replicas),
            });
        }

        match self.kubernetes_client.scale(&deployment_id, target_replicas).await {
            Ok(_) => {
                deployment.replicas = target_replicas;
                deployment.updated_at = Utc::now();
                info!("Scaled deployment {} to {} replicas", deployment_id, target_replicas);
            }
            Err(e) => {
                error!("Failed to scale deployment {}: {}", deployment_id, e);
                return Err(e.into());
            }
        }

        Ok(())
    }

    pub async fn rollback_deployment(
        &self,
        deployment_id: IntelligenceId,
        target_version: Option<String>,
    ) -> IntelligenceResult<()> {
        let mut deployments = self.deployments.write().await;
        let deployment = deployments.get_mut(&deployment_id)
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "deployment".to_string(),
                id: deployment_id.clone(),
            })?;

        deployment.status = DeploymentStatus::RollingBack;
        deployment.updated_at = Utc::now();

        let rollback_version = target_version.unwrap_or_else(|| "previous".to_string());

        match self.kubernetes_client.rollback(&deployment_id, &rollback_version).await {
            Ok(_) => {
                deployment.status = DeploymentStatus::RolledBack;
                deployment.completed_at = Some(Utc::now());
                info!("Rolled back deployment {} to version {}", deployment_id, rollback_version);
            }
            Err(e) => {
                deployment.status = DeploymentStatus::Failed;
                error!("Rollback failed for deployment {}: {}", deployment_id, e);
                return Err(e.into());
            }
        }

        deployment.updated_at = Utc::now();
        Ok(())
    }

    pub async fn get_deployment_status(&self, deployment_id: IntelligenceId) -> IntelligenceResult<Deployment> {
        let deployments = self.deployments.read().await;
        deployments.get(&deployment_id)
            .cloned()
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "deployment".to_string(),
                id: deployment_id,
            })
    }

    pub async fn get_all_deployments(&self) -> IntelligenceResult<Vec<Deployment>> {
        let deployments = self.deployments.read().await;
        Ok(deployments.values().cloned().collect())
    }

    async fn run_pipeline_stages(
        executions: &Arc<RwLock<HashMap<IntelligenceId, DeploymentExecution>>>,
        deployments: &Arc<RwLock<HashMap<IntelligenceId, Deployment>>>,
        execution_id: &IntelligenceId,
        stages: &[PipelineStage],
    ) -> IntelligenceResult<()> {
        let mut executions_guard = executions.write().await;
        let execution = executions_guard.get_mut(execution_id)
            .ok_or_else(|| IntelligenceError::NotFound { 
                resource: "execution".to_string(),
                id: execution_id.clone(),
            })?;

        execution.status = ExecutionStatus::Running;

        for stage in stages {
            let stage_execution = StageExecution {
                stage_id: stage.stage_id.clone(),
                status: ExecutionStatus::Running,
                started_at: Utc::now(),
                completed_at: None,
                duration_seconds: 0,
                logs: Vec::new(),
                exit_code: None,
            };

            execution.stages.push(stage_execution);
        }

        drop(executions_guard);

        for (index, stage) in stages.iter().enumerate() {
            let mut executions_guard = executions.write().await;
            let execution = executions_guard.get_mut(execution_id).unwrap();
            let stage_execution = &mut execution.stages[index];

            stage_execution.status = ExecutionStatus::Running;
            stage_execution.started_at = Utc::now();

            drop(executions_guard);

            let stage_result = Self::execute_stage(stage).await;

            let mut executions_guard = executions.write().await;
            let execution = executions_guard.get_mut(execution_id).unwrap();
            let stage_execution = &mut execution.stages[index];

            stage_execution.completed_at = Some(Utc::now());
            stage_execution.duration_seconds = (stage_execution.completed_at.unwrap() 
                - stage_execution.started_at).num_seconds() as u64;

            match stage_result {
                Ok(_) => {
                    stage_execution.status = ExecutionStatus::Completed;
                    stage_execution.exit_code = Some(0);
                    info!("Stage {} completed successfully", stage.name);
                }
                Err(e) => {
                    stage_execution.status = ExecutionStatus::Failed;
                    stage_execution.exit_code = Some(1);
                    stage_execution.logs.push(format!("Error: {}", e));
                    error!("Stage {} failed: {}", stage.name, e);

                    if stage.is_required {
                        execution.status = ExecutionStatus::Failed;
                        execution.completed_at = Some(Utc::now());
                        return Err(e);
                    }
                }
            }
        }

        let mut executions_guard = executions.write().await;
        let execution = executions_guard.get_mut(execution_id).unwrap();
        execution.status = ExecutionStatus::Completed;
        execution.completed_at = Some(Utc::now());

        let mut deployments_guard = deployments.write().await;
        if let Some(deployment) = deployments_guard.get_mut(&execution.deployment_id) {
            deployment.status = DeploymentStatus::Completed;
            deployment.completed_at = Some(Utc::now());
            deployment.updated_at = Utc::now();
        }

        info!("Pipeline execution completed: {}", execution_id);
        Ok(())
    }

    async fn execute_stage(stage: &PipelineStage) -> IntelligenceResult<()> {
        match stage.stage_type {
            StageType::Build => {
                info!("Executing build stage: {}", stage.name);
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            }
            StageType::Test => {
                info!("Executing test stage: {}", stage.name);
                tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
            }
            StageType::SecurityScan => {
                info!("Executing security scan stage: {}", stage.name);
                tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
            }
            StageType::Package => {
                info!("Executing package stage: {}", stage.name);
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
            StageType::Deploy => {
                info!("Executing deploy stage: {}", stage.name);
                tokio::time::sleep(tokio::time::Duration::from_secs(45)).await;
            }
            StageType::Verify => {
                info!("Executing verify stage: {}", stage.name);
                tokio::time::sleep(tokio::time::Duration::from_secs(25)).await;
            }
            StageType::Rollback => {
                info!("Executing rollback stage: {}", stage.name);
                tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
            }
            StageType::Notify => {
                info!("Executing notify stage: {}", stage.name);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }

        Ok(())
    }
}

impl KubernetesClient {
    pub async fn deploy(&self, _config: &DeploymentConfig, _version: &str) -> IntelligenceResult<()> {
        info!("Deploying to Kubernetes cluster: {}", self.cluster_url);
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        Ok(())
    }

    pub async fn scale(&self, _deployment_id: &IntelligenceId, _replicas: u32) -> IntelligenceResult<()> {
        info!("Scaling deployment to {} replicas", _replicas);
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
        Ok(())
    }

    pub async fn rollback(&self, _deployment_id: &IntelligenceId, _version: &str) -> IntelligenceResult<()> {
        info!("Rolling back deployment to version: {}", _version);
        tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
        Ok(())
    }
}
