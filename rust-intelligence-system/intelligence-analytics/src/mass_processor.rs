use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, warn, error, debug};

use intelligence_core::{
    IntelligenceData, AnalysisResult, AnalysisType, IntelligenceId, Result as IntelligenceResult
};

/// High-throughput mass data processor

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassProcessingConfig {
    pub max_concurrent_processors: usize,
    pub batch_size: usize,
    pub processing_timeout_seconds: u64,
    pub memory_limit_mb: usize,
    pub cpu_limit_percent: f64,
    pub enable_parallel_processing: bool,
    pub enable_distributed_processing: bool,
    pub chunk_size: usize,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingJob {
    pub job_id: Uuid,
    pub job_type: ProcessingJobType,
    pub input_data: Vec<IntelligenceData>,
    pub configuration: HashMap<String, serde_json::Value>,
    pub priority: u32,
    pub created_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub status: JobStatus,
    pub progress: f64,
    pub result: Option<ProcessingJobResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessingJobType {
    DataValidation,
    DataTransformation,
    FeatureExtraction,
    MLInference,
    PatternRecognition,
    AnomalyDetection,
    ThreatAnalysis,
    BehavioralAnalysis,
    NetworkAnalysis,
    ReportGeneration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingJobResult {
    pub job_id: Uuid,
    pub output_data: Vec<IntelligenceData>,
    pub analysis_results: Vec<AnalysisResult>,
    pub processing_time_ms: u64,
    pub memory_used_mb: f64,
    pub cpu_time_seconds: f64,
    pub success_count: usize,
    pub failure_count: usize,
    pub errors: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub total_jobs_processed: u64,
    pub successful_jobs: u64,
    pub failed_jobs: u64,
    pub total_data_points_processed: u64,
    pub average_processing_time_ms: f64,
    pub throughput_per_second: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub job_type_stats: HashMap<ProcessingJobType, JobTypeStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobTypeStats {
    pub jobs_processed: u64,
    pub successful_jobs: u64,
    pub failed_jobs: u64,
    pub average_processing_time_ms: f64,
    pub throughput_per_second: f64,
}

/// High-performance mass data processor
pub struct MassProcessor {
    config: MassProcessingConfig,
    job_queue: Arc<RwLock<Vec<ProcessingJob>>>,
    active_jobs: Arc<RwLock<HashMap<Uuid, ProcessingJob>>>,
    completed_jobs: Arc<RwLock<HashMap<Uuid, ProcessingJob>>>,
    stats: Arc<RwLock<ProcessingStats>>,
    is_running: Arc<RwLock<bool>>,
    processors: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

impl MassProcessor {
    pub fn new(config: MassProcessingConfig) -> Self {
        Self {
            config,
            job_queue: Arc::new(RwLock::new(Vec::new())),
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
            completed_jobs: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ProcessingStats {
                total_jobs_processed: 0,
                successful_jobs: 0,
                failed_jobs: 0,
                total_data_points_processed: 0,
                average_processing_time_ms: 0.0,
                throughput_per_second: 0.0,
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
                start_time: Utc::now(),
                last_update: Utc::now(),
                job_type_stats: HashMap::new(),
            })),
            is_running: Arc::new(RwLock::new(false)),
            processors: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start mass processing
    pub async fn start(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = true;
        drop(is_running);

        info!("Starting mass processor with {} concurrent processors", 
              self.config.max_concurrent_processors);

        // Start job processors
        let mut tasks = Vec::new();
        for i in 0..self.config.max_concurrent_processors {
            let processor = self.clone_for_task();
            let task = tokio::spawn(async move {
                processor.process_jobs().await
            });
            tasks.push(task);
        }

        // Start stats updater
        let processor = self.clone_for_task();
        let stats_task = tokio::spawn(async move {
            processor.update_stats().await
        });
        tasks.push(stats_task);

        // Start job scheduler
        let processor = self.clone_for_task();
        let scheduler_task = tokio::spawn(async move {
            processor.schedule_jobs().await
        });
        tasks.push(scheduler_task);

        // Store processor tasks
        {
            let mut processors = self.processors.write().await;
            processors.extend(tasks);
        }

        info!("Mass processor started successfully");
        Ok(())
    }

    /// Stop mass processing
    pub async fn stop(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        drop(is_running);

        // Wait for all processors to finish
        let mut processors = self.processors.write().await;
        for processor in processors.drain(..) {
            processor.abort();
        }

        info!("Mass processor stopped");
        Ok(())
    }

    /// Submit a processing job
    pub async fn submit_job(&self, job: ProcessingJob) -> IntelligenceResult<Uuid> {
        let job_id = job.job_id;
        
        // Validate job
        if job.input_data.is_empty() {
            return Err(intelligence_core::IntelligenceError::InvalidInput {
                field: "input_data".to_string(),
                message: "Job must have input data".to_string(),
            });
        }

        // Add to queue
        {
            let mut queue = self.job_queue.write().await;
            queue.push(job);
            
            // Sort by priority (higher priority first)
            queue.sort_by(|a, b| b.priority.cmp(&a.priority));
        }

        info!("Submitted job {} with {} data points", job_id, job.input_data.len());
        Ok(job_id)
    }

    /// Get job status
    pub async fn get_job_status(&self, job_id: Uuid) -> Option<JobStatus> {
        // Check active jobs
        if let Some(job) = self.active_jobs.read().await.get(&job_id) {
            return Some(job.status.clone());
        }

        // Check completed jobs
        if let Some(job) = self.completed_jobs.read().await.get(&job_id) {
            return Some(job.status.clone());
        }

        // Check queue
        let queue = self.job_queue.read().await;
        if let Some(job) = queue.iter().find(|j| j.job_id == job_id) {
            return Some(job.status.clone());
        }

        None
    }

    /// Get job result
    pub async fn get_job_result(&self, job_id: Uuid) -> Option<ProcessingJobResult> {
        if let Some(job) = self.completed_jobs.read().await.get(&job_id) {
            job.result.clone()
        } else {
            None
        }
    }

    /// Process jobs from queue
    async fn process_jobs(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let job = {
                let mut queue = self.job_queue.write().await;
                queue.pop()
            };

            if let Some(mut job) = job {
                // Move to active jobs
                {
                    let mut active_jobs = self.active_jobs.write().await;
                    job.status = JobStatus::Running;
                    active_jobs.insert(job.job_id, job.clone());
                }

                // Process the job
                match self.execute_job(job.clone()).await {
                    Ok(result) => {
                        // Move to completed jobs
                        {
                            let mut active_jobs = self.active_jobs.write().await;
                            active_jobs.remove(&job.job_id);
                        }

                        let mut completed_job = job;
                        completed_job.status = JobStatus::Completed;
                        completed_job.result = Some(result);
                        completed_job.progress = 100.0;

                        {
                            let mut completed_jobs = self.completed_jobs.write().await;
                            completed_jobs.insert(completed_job.job_id, completed_job);
                        }

                        info!("Completed job {} in {}ms", job.job_id, 
                              job.result.as_ref().unwrap().processing_time_ms);
                    }
                    Err(e) => {
                        error!("Failed to process job {}: {}", job.job_id, e);

                        // Move to completed jobs with failure status
                        {
                            let mut active_jobs = self.active_jobs.write().await;
                            active_jobs.remove(&job.job_id);
                        }

                        let mut failed_job = job;
                        failed_job.status = JobStatus::Failed;
                        failed_job.progress = 0.0;

                        {
                            let mut completed_jobs = self.completed_jobs.write().await;
                            completed_jobs.insert(failed_job.job_id, failed_job);
                        }
                    }
                }
            } else {
                // No jobs to process, wait a bit
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }

        Ok(())
    }

    /// Execute a processing job
    async fn execute_job(&self, job: ProcessingJob) -> IntelligenceResult<ProcessingJobResult> {
        let start_time = std::time::Instant::now();
        let start_memory = self.get_memory_usage().await;
        let start_cpu = self.get_cpu_usage().await;

        let mut output_data = Vec::new();
        let mut analysis_results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;
        let mut errors = Vec::new();

        // Process data in chunks for memory efficiency
        let chunks = job.input_data.chunks(self.config.chunk_size);
        let total_chunks = chunks.len();
        let mut processed_chunks = 0;

        for chunk in chunks {
            match self.process_chunk(chunk, &job.job_type, &job.configuration).await {
                Ok((chunk_output, chunk_analysis)) => {
                    output_data.extend(chunk_output);
                    analysis_results.extend(chunk_analysis);
                    success_count += chunk.len();
                }
                Err(e) => {
                    error!("Failed to process chunk: {}", e);
                    failure_count += chunk.len();
                    errors.push(e.to_string());
                }
            }

            processed_chunks += 1;
            
            // Update progress
            let progress = (processed_chunks as f64 / total_chunks as f64) * 100.0;
            {
                let mut active_jobs = self.active_jobs.write().await;
                if let Some(active_job) = active_jobs.get_mut(&job.job_id) {
                    active_job.progress = progress;
                }
            }

            // Check for timeout
            if start_time.elapsed().as_secs() > self.config.processing_timeout_seconds {
                return Err(intelligence_core::IntelligenceError::Timeout {
                    operation: format!("Job {} processing", job.job_id),
                });
            }
        }

        let processing_time = start_time.elapsed().as_millis() as u64;
        let end_memory = self.get_memory_usage().await;
        let end_cpu = self.get_cpu_usage().await;

        let result = ProcessingJobResult {
            job_id: job.job_id,
            output_data,
            analysis_results,
            processing_time_ms: processing_time,
            memory_used_mb: end_memory - start_memory,
            cpu_time_seconds: end_cpu - start_cpu,
            success_count,
            failure_count,
            errors,
            metadata: HashMap::new(),
            completed_at: Utc::now(),
        };

        Ok(result)
    }

    /// Process a chunk of data
    async fn process_chunk(
        &self,
        chunk: &[IntelligenceData],
        job_type: &ProcessingJobType,
        configuration: &HashMap<String, serde_json::Value>,
    ) -> IntelligenceResult<(Vec<IntelligenceData>, Vec<AnalysisResult>)> {
        let mut output_data = Vec::new();
        let mut analysis_results = Vec::new();

        match job_type {
            ProcessingJobType::DataValidation => {
                for data in chunk {
                    if let Ok(validated_data) = self.validate_data(data).await {
                        output_data.push(validated_data);
                    }
                }
            }
            ProcessingJobType::DataTransformation => {
                for data in chunk {
                    if let Ok(transformed_data) = self.transform_data(data, configuration).await {
                        output_data.push(transformed_data);
                    }
                }
            }
            ProcessingJobType::FeatureExtraction => {
                for data in chunk {
                    if let Ok((extracted_data, analysis)) = self.extract_features(data).await {
                        output_data.push(extracted_data);
                        analysis_results.push(analysis);
                    }
                }
            }
            ProcessingJobType::MLInference => {
                for data in chunk {
                    if let Ok((inferred_data, analysis)) = self.run_ml_inference(data).await {
                        output_data.push(inferred_data);
                        analysis_results.push(analysis);
                    }
                }
            }
            ProcessingJobType::PatternRecognition => {
                for data in chunk {
                    if let Ok((pattern_data, analysis)) = self.recognize_patterns(data).await {
                        output_data.push(pattern_data);
                        analysis_results.push(analysis);
                    }
                }
            }
            ProcessingJobType::AnomalyDetection => {
                for data in chunk {
                    if let Ok((anomaly_data, analysis)) = self.detect_anomalies(data).await {
                        output_data.push(anomaly_data);
                        analysis_results.push(analysis);
                    }
                }
            }
            ProcessingJobType::ThreatAnalysis => {
                for data in chunk {
                    if let Ok((threat_data, analysis)) = self.analyze_threats(data).await {
                        output_data.push(threat_data);
                        analysis_results.push(analysis);
                    }
                }
            }
            ProcessingJobType::BehavioralAnalysis => {
                for data in chunk {
                    if let Ok((behavior_data, analysis)) = self.analyze_behavior(data).await {
                        output_data.push(behavior_data);
                        analysis_results.push(analysis);
                    }
                }
            }
            ProcessingJobType::NetworkAnalysis => {
                for data in chunk {
                    if let Ok((network_data, analysis)) = self.analyze_network(data).await {
                        output_data.push(network_data);
                        analysis_results.push(analysis);
                    }
                }
            }
            ProcessingJobType::ReportGeneration => {
                for data in chunk {
                    if let Ok((report_data, analysis)) = self.generate_report(data).await {
                        output_data.push(report_data);
                        analysis_results.push(analysis);
                    }
                }
            }
        }

        Ok((output_data, analysis_results))
    }

    /// Validate data
    async fn validate_data(&self, data: &IntelligenceData) -> IntelligenceResult<IntelligenceData> {
        // Simulate data validation
        let mut validated_data = data.clone();
        
        // Check content quality
        if data.content.is_empty() {
            return Err(intelligence_core::IntelligenceError::InvalidInput {
                field: "content".to_string(),
                message: "Content cannot be empty".to_string(),
            });
        }

        // Update quality score based on validation
        validated_data.quality_score = if data.content.len() > 10 { 0.9 } else { 0.5 };
        
        Ok(validated_data)
    }

    /// Transform data
    async fn transform_data(
        &self,
        data: &IntelligenceData,
        configuration: &HashMap<String, serde_json::Value>,
    ) -> IntelligenceResult<IntelligenceData> {
        // Simulate data transformation
        let mut transformed_data = data.clone();
        
        if let Some(transform_type) = configuration.get("transform_type") {
            if let Some(transform_str) = transform_type.as_str() {
                match transform_str {
                    "uppercase" => {
                        transformed_data.content = data.content.to_uppercase();
                    }
                    "lowercase" => {
                        transformed_data.content = data.content.to_lowercase();
                    }
                    "normalize" => {
                        transformed_data.content = data.content.trim().to_string();
                    }
                    _ => {}
                }
            }
        }
        
        Ok(transformed_data)
    }

    /// Extract features
    async fn extract_features(&self, data: &IntelligenceData) -> IntelligenceResult<(IntelligenceData, AnalysisResult)> {
        // Simulate feature extraction
        let mut feature_data = data.clone();
        
        let features = serde_json::json!({
            "text_length": data.content.len(),
            "word_count": data.content.split_whitespace().count(),
            "has_numbers": data.content.chars().any(|c| c.is_ascii_digit()),
            "has_emails": regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")?
                .is_match(&data.content),
            "confidence": data.confidence,
            "quality_score": data.quality_score
        });

        feature_data.metadata.insert("features".to_string(), features);

        let analysis = AnalysisResult {
            id: IntelligenceId::new(),
            analysis_type: AnalysisType::FeatureExtraction,
            input_data: vec![data.id.clone()],
            results: HashMap::new(),
            confidence: 0.9,
            timestamp: Utc::now(),
            processing_time_ms: 10,
        };

        Ok((feature_data, analysis))
    }

    /// Run ML inference
    async fn run_ml_inference(&self, data: &IntelligenceData) -> IntelligenceResult<(IntelligenceData, AnalysisResult)> {
        // Simulate ML inference
        let mut inferred_data = data.clone();
        
        let prediction = serde_json::json!({
            "sentiment": "positive",
            "confidence": 0.85,
            "threat_level": "low",
            "anomaly_score": 0.2
        });

        inferred_data.metadata.insert("ml_prediction".to_string(), prediction);

        let analysis = AnalysisResult {
            id: IntelligenceId::new(),
            analysis_type: AnalysisType::BehaviorAnalysis,
            input_data: vec![data.id.clone()],
            results: HashMap::new(),
            confidence: 0.85,
            timestamp: Utc::now(),
            processing_time_ms: 50,
        };

        Ok((inferred_data, analysis))
    }

    /// Recognize patterns
    async fn recognize_patterns(&self, data: &IntelligenceData) -> IntelligenceResult<(IntelligenceData, AnalysisResult)> {
        // Simulate pattern recognition
        let mut pattern_data = data.clone();
        
        let patterns = serde_json::json!({
            "repetitive_patterns": 3,
            "sequential_patterns": 1,
            "anomalous_patterns": 0,
            "pattern_confidence": 0.8
        });

        pattern_data.metadata.insert("patterns".to_string(), patterns);

        let analysis = AnalysisResult {
            id: IntelligenceId::new(),
            analysis_type: AnalysisType::PatternRecognition,
            input_data: vec![data.id.clone()],
            results: HashMap::new(),
            confidence: 0.8,
            timestamp: Utc::now(),
            processing_time_ms: 30,
        };

        Ok((pattern_data, analysis))
    }

    /// Detect anomalies
    async fn detect_anomalies(&self, data: &IntelligenceData) -> IntelligenceResult<(IntelligenceData, AnalysisResult)> {
        // Simulate anomaly detection
        let mut anomaly_data = data.clone();
        
        let anomaly_score = if data.content.len() > 1000 { 0.8 } else { 0.2 };
        let is_anomaly = anomaly_score > 0.5;

        let anomaly_info = serde_json::json!({
            "is_anomaly": is_anomaly,
            "anomaly_score": anomaly_score,
            "anomaly_type": if is_anomaly { "unusual_length" } else { "normal" }
        });

        anomaly_data.metadata.insert("anomaly_detection".to_string(), anomaly_info);

        let analysis = AnalysisResult {
            id: IntelligenceId::new(),
            analysis_type: AnalysisType::ThreatDetection,
            input_data: vec![data.id.clone()],
            results: HashMap::new(),
            confidence: anomaly_score,
            timestamp: Utc::now(),
            processing_time_ms: 25,
        };

        Ok((anomaly_data, analysis))
    }

    /// Analyze threats
    async fn analyze_threats(&self, data: &IntelligenceData) -> IntelligenceResult<(IntelligenceData, AnalysisResult)> {
        // Simulate threat analysis
        let mut threat_data = data.clone();
        
        let threat_indicators = serde_json::json!({
            "threat_level": "low",
            "indicators": ["normal_communication"],
            "risk_score": 0.1
        });

        threat_data.metadata.insert("threat_analysis".to_string(), threat_indicators);

        let analysis = AnalysisResult {
            id: IntelligenceId::new(),
            analysis_type: AnalysisType::ThreatDetection,
            input_data: vec![data.id.clone()],
            results: HashMap::new(),
            confidence: 0.9,
            timestamp: Utc::now(),
            processing_time_ms: 40,
        };

        Ok((threat_data, analysis))
    }

    /// Analyze behavior
    async fn analyze_behavior(&self, data: &IntelligenceData) -> IntelligenceResult<(IntelligenceData, AnalysisResult)> {
        // Simulate behavioral analysis
        let mut behavior_data = data.clone();
        
        let behavior_analysis = serde_json::json!({
            "behavior_type": "normal",
            "activity_level": "medium",
            "communication_style": "formal"
        });

        behavior_data.metadata.insert("behavior_analysis".to_string(), behavior_analysis);

        let analysis = AnalysisResult {
            id: IntelligenceId::new(),
            analysis_type: AnalysisType::BehaviorAnalysis,
            input_data: vec![data.id.clone()],
            results: HashMap::new(),
            confidence: 0.8,
            timestamp: Utc::now(),
            processing_time_ms: 35,
        };

        Ok((behavior_data, analysis))
    }

    /// Analyze network
    async fn analyze_network(&self, data: &IntelligenceData) -> IntelligenceResult<(IntelligenceData, AnalysisResult)> {
        // Simulate network analysis
        let mut network_data = data.clone();
        
        let network_analysis = serde_json::json!({
            "network_connections": 5,
            "centrality_score": 0.3,
            "community_id": "community_1"
        });

        network_data.metadata.insert("network_analysis".to_string(), network_analysis);

        let analysis = AnalysisResult {
            id: IntelligenceId::new(),
            analysis_type: AnalysisType::NetworkAnalysis,
            input_data: vec![data.id.clone()],
            results: HashMap::new(),
            confidence: 0.7,
            timestamp: Utc::now(),
            processing_time_ms: 45,
        };

        Ok((network_data, analysis))
    }

    /// Generate report
    async fn generate_report(&self, data: &IntelligenceData) -> IntelligenceResult<(IntelligenceData, AnalysisResult)> {
        // Simulate report generation
        let mut report_data = data.clone();
        
        let report = serde_json::json!({
            "summary": "Data analysis completed",
            "key_findings": ["Normal activity detected"],
            "recommendations": ["Continue monitoring"]
        });

        report_data.metadata.insert("report".to_string(), report);

        let analysis = AnalysisResult {
            id: IntelligenceId::new(),
            analysis_type: AnalysisType::DataInterpretation,
            input_data: vec![data.id.clone()],
            results: HashMap::new(),
            confidence: 0.9,
            timestamp: Utc::now(),
            processing_time_ms: 20,
        };

        Ok((report_data, analysis))
    }

    /// Schedule jobs
    async fn schedule_jobs(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            // Check for jobs that have exceeded their deadline
            let now = Utc::now();
            let mut jobs_to_cancel = Vec::new();

            {
                let active_jobs = self.active_jobs.read().await;
                for (job_id, job) in active_jobs.iter() {
                    if let Some(deadline) = job.deadline {
                        if now > deadline {
                            jobs_to_cancel.push(*job_id);
                        }
                    }
                }
            }

            // Cancel timed out jobs
            for job_id in jobs_to_cancel {
                {
                    let mut active_jobs = self.active_jobs.write().await;
                    if let Some(mut job) = active_jobs.remove(&job_id) {
                        job.status = JobStatus::Timeout;
                        let mut completed_jobs = self.completed_jobs.write().await;
                        completed_jobs.insert(job_id, job);
                    }
                }
                warn!("Job {} timed out and was cancelled", job_id);
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }

        Ok(())
    }

    /// Update processing statistics
    async fn update_stats(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let mut stats = self.stats.write().await;
            
            // Update job counts
            let completed_jobs = self.completed_jobs.read().await;
            stats.total_jobs_processed = completed_jobs.len() as u64;
            stats.successful_jobs = completed_jobs.values()
                .filter(|job| job.status == JobStatus::Completed)
                .count() as u64;
            stats.failed_jobs = completed_jobs.values()
                .filter(|job| job.status == JobStatus::Failed || job.status == JobStatus::Timeout)
                .count() as u64;

            // Update data points processed
            stats.total_data_points_processed = completed_jobs.values()
                .filter(|job| job.status == JobStatus::Completed)
                .map(|job| job.input_data.len() as u64)
                .sum();

            // Calculate throughput
            let elapsed = Utc::now().signed_duration_since(stats.start_time);
            if elapsed.num_seconds() > 0 {
                stats.throughput_per_second = stats.total_data_points_processed as f64 / elapsed.num_seconds() as f64;
            }

            // Update system metrics
            stats.memory_usage_mb = self.get_memory_usage().await;
            stats.cpu_usage_percent = self.get_cpu_usage().await;
            stats.last_update = Utc::now();

            drop(stats);
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        Ok(())
    }

    /// Get memory usage
    async fn get_memory_usage(&self) -> f64 {
        // Simulate memory usage (in real implementation, use system APIs)
        100.0 + (rand::random::<f64>() * 50.0)
    }

    /// Get CPU usage
    async fn get_cpu_usage(&self) -> f64 {
        // Simulate CPU usage (in real implementation, use system APIs)
        20.0 + (rand::random::<f64>() * 30.0)
    }

    /// Get processing statistics
    pub async fn get_stats(&self) -> ProcessingStats {
        self.stats.read().await.clone()
    }

    /// Clone processor for task execution
    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            job_queue: Arc::clone(&self.job_queue),
            active_jobs: Arc::clone(&self.active_jobs),
            completed_jobs: Arc::clone(&self.completed_jobs),
            stats: Arc::clone(&self.stats),
            is_running: Arc::clone(&self.is_running),
            processors: Arc::clone(&self.processors),
        }
    }
}
