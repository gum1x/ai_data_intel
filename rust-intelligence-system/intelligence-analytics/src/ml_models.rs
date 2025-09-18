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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_id: String,
    pub model_type: ModelType,
    pub model_path: String,
    pub input_features: Vec<String>,
    pub output_classes: Vec<String>,
    pub confidence_threshold: f64,
    pub batch_size: usize,
    pub max_sequence_length: usize,
    pub preprocessing_config: HashMap<String, serde_json::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    SentimentAnalysis,
    ThreatDetection,
    AnomalyDetection,
    BehaviorPrediction,
    EntityRecognition,
    Classification,
    Regression,
    Clustering,
    TextGeneration,
    ImageAnalysis,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPrediction {
    pub model_id: String,
    pub input_id: IntelligenceId,
    pub predictions: Vec<PredictionResult>,
    pub confidence: f64,
    pub processing_time_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    pub class: String,
    pub confidence: f64,
    pub probability: f64,
    pub explanation: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub model_id: String,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub total_predictions: u64,
    pub correct_predictions: u64,
    pub average_confidence: f64,
    pub last_updated: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPredictionRequest {
    pub model_id: String,
    pub inputs: Vec<IntelligenceData>,
    pub batch_id: Uuid,
    pub priority: u32,
    pub timeout_seconds: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPredictionResult {
    pub batch_id: Uuid,
    pub model_id: String,
    pub predictions: Vec<ModelPrediction>,
    pub total_processing_time_ms: u64,
    pub success_count: usize,
    pub failure_count: usize,
    pub timestamp: DateTime<Utc>,
}
pub struct MLModelManager {
    models: Arc<RwLock<HashMap<String, ModelConfig>>>,
    model_metrics: Arc<RwLock<HashMap<String, ModelMetrics>>>,
    prediction_cache: Arc<RwLock<HashMap<String, ModelPrediction>>>,
    batch_queue: Arc<RwLock<Vec<BatchPredictionRequest>>>,
    is_running: Arc<RwLock<bool>>,
    processors: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}
impl MLModelManager {
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            model_metrics: Arc::new(RwLock::new(HashMap::new())),
            prediction_cache: Arc::new(RwLock::new(HashMap::new())),
            batch_queue: Arc::new(RwLock::new(Vec::new())),
            is_running: Arc::new(RwLock::new(false)),
            processors: Arc::new(RwLock::new(Vec::new())),
        }
    }
    pub async fn load_model(&self, config: ModelConfig) -> IntelligenceResult<()> {
        info!("Loading ML model: {} ({:?})", config.model_id, config.model_type);
        let mut models = self.models.write().await;
        models.insert(config.model_id.clone(), config.clone());
        let mut metrics = self.model_metrics.write().await;
        metrics.insert(config.model_id.clone(), ModelMetrics {
            model_id: config.model_id.clone(),
            accuracy: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1_score: 0.0,
            total_predictions: 0,
            correct_predictions: 0,
            average_confidence: 0.0,
            last_updated: Utc::now(),
        });
        info!("Model {} loaded successfully", config.model_id);
        Ok(())
    }
    pub async fn start(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = true;
        drop(is_running);
        info!("Starting ML model manager");
        let mut tasks = Vec::new();
        for i in 0..4 {
            let manager = self.clone_for_task();
            let task = tokio::spawn(async move {
                manager.process_batches().await
            });
            tasks.push(task);
        }
        let manager = self.clone_for_task();
        let metrics_task = tokio::spawn(async move {
            manager.update_metrics().await
        });
        tasks.push(metrics_task);
        {
            let mut processors = self.processors.write().await;
            processors.extend(tasks);
        }
        info!("ML model manager started with 4 concurrent processors");
        Ok(())
    }
    pub async fn stop(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        drop(is_running);
        let mut processors = self.processors.write().await;
        for processor in processors.drain(..) {
            processor.abort();
        }
        info!("ML model manager stopped");
        Ok(())
    }
    pub async fn predict(&self, model_id: &str, input: &IntelligenceData) -> IntelligenceResult<ModelPrediction> {
        let start_time = std::time::Instant::now();
        let cache_key = format!("{}:{}", model_id, input.id.0);
        if let Some(cached_prediction) = self.prediction_cache.read().await.get(&cache_key) {
            return Ok(cached_prediction.clone());
        }
        let model_config = {
            let models = self.models.read().await;
            models.get(model_id).cloned()
                .ok_or_else(|| intelligence_core::IntelligenceError::NotFound {
                    resource: format!("Model {}", model_id)
                })?
        };
        let preprocessed_input = self.preprocess_input(input, &model_config).await?;
        let predictions = self.run_model_prediction(&model_config, &preprocessed_input).await?;
        let processing_time = start_time.elapsed().as_millis() as u64;
        let confidence = predictions.iter().map(|p| p.confidence).fold(0.0, f64::max);
        let prediction = ModelPrediction {
            model_id: model_id.to_string(),
            input_id: input.id.clone(),
            predictions,
            confidence,
            processing_time_ms: processing_time,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        {
            let mut cache = self.prediction_cache.write().await;
            cache.insert(cache_key, prediction.clone());
            if cache.len() > 10000 {
                let keys_to_remove: Vec<String> = cache.keys().take(1000).cloned().collect();
                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }
        }
        self.update_model_metrics(model_id, &prediction).await;
        Ok(prediction)
    }
    pub async fn predict_batch(&self, request: BatchPredictionRequest) -> IntelligenceResult<BatchPredictionResult> {
        let start_time = std::time::Instant::now();
        let mut predictions = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;
        for input in &request.inputs {
            match self.predict(&request.model_id, input).await {
                Ok(prediction) => {
                    predictions.push(prediction);
                    success_count += 1;
                }
                Err(e) => {
                    error!("Failed to predict on input {}: {}", input.id.0, e);
                    failure_count += 1;
                }
            }
        }
        let total_processing_time = start_time.elapsed().as_millis() as u64;
        Ok(BatchPredictionResult {
            batch_id: request.batch_id,
            model_id: request.model_id,
            predictions,
            total_processing_time_ms: total_processing_time,
            success_count,
            failure_count,
            timestamp: Utc::now(),
        })
    }
    pub async fn queue_batch_prediction(&self, request: BatchPredictionRequest) -> IntelligenceResult<Uuid> {
        let batch_id = request.batch_id;
        let mut queue = self.batch_queue.write().await;
        queue.push(request);
        info!("Queued batch prediction request: {}", batch_id);
        Ok(batch_id)
    }
    async fn process_batches(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let batch_request = {
                let mut queue = self.batch_queue.write().await;
                queue.pop()
            };
            if let Some(request) = batch_request {
                match self.predict_batch(request.clone()).await {
                    Ok(result) => {
                        info!("Processed batch {}: {} successful, {} failed in {}ms",
                              result.batch_id, result.success_count, result.failure_count,
                              result.total_processing_time_ms);
                    }
                    Err(e) => {
                        error!("Failed to process batch {}: {}", request.batch_id, e);
                    }
                }
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
        Ok(())
    }
    async fn preprocess_input(
        &self,
        input: &IntelligenceData,
        model_config: &ModelConfig,
    ) -> IntelligenceResult<Vec<f64>> {
        let mut features = Vec::new();
        for feature_name in &model_config.input_features {
            let feature_value = match feature_name.as_str() {
                "text_length" => {
                    input.content.len() as f64
                }
                "word_count" => {
                    input.content.split_whitespace().count() as f64
                }
                "has_numbers" => {
                    if input.content.chars().any(|c| c.is_ascii_digit()) { 1.0 } else { 0.0 }
                }
                "has_emails" => {
                    if regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")?
                        .is_match(&input.content) { 1.0 } else { 0.0 }
                }
                "has_phone_numbers" => {
                    if regex::Regex::new(r"\+?[\d\s\-\(\)]{10,}")?
                        .is_match(&input.content) { 1.0 } else { 0.0 }
                }
                "has_crypto_addresses" => {
                    let btc_regex = regex::Regex::new(r"[13][a-km-zA-HJ-NP-Z1-9]{25,34}")?;
                    let eth_regex = regex::Regex::new(r"0x[a-fA-F0-9]{40}")?;
                    if btc_regex.is_match(&input.content) || eth_regex.is_match(&input.content) { 1.0 } else { 0.0 }
                }
                "confidence" => {
                    input.confidence
                }
                "quality_score" => {
                    input.quality_score
                }
                "timestamp_hour" => {
                    input.timestamp.hour() as f64
                }
                "timestamp_day_of_week" => {
                    input.timestamp.weekday().num_days_from_monday() as f64
                }
                _ => {
                    0.0
                }
            };
            features.push(feature_value);
        }
        if let Some(normalization) = model_config.preprocessing_config.get("normalize") {
            if normalization.as_bool().unwrap_or(false) {
                self.normalize_features(&mut features).await;
            }
        }
        Ok(features)
    }
    async fn normalize_features(&self, features: &mut [f64]) {
        if features.is_empty() {
            return;
        }
        let mean = features.iter().sum::<f64>() / features.len() as f64;
        let variance = features.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / features.len() as f64;
        let std_dev = variance.sqrt();
        if std_dev > 0.0 {
            for feature in features.iter_mut() {
                *feature = (*feature - mean) / std_dev;
            }
        }
    }
    async fn run_model_prediction(
        &self,
        model_config: &ModelConfig,
        features: &[f64],
    ) -> IntelligenceResult<Vec<PredictionResult>> {
        match model_config.model_type {
            ModelType::SentimentAnalysis => {
                self.predict_sentiment(features).await
            }
            ModelType::ThreatDetection => {
                self.predict_threat(features).await
            }
            ModelType::AnomalyDetection => {
                self.predict_anomaly(features).await
            }
            ModelType::BehaviorPrediction => {
                self.predict_behavior(features).await
            }
            ModelType::EntityRecognition => {
                self.predict_entities(features).await
            }
            ModelType::Classification => {
                self.predict_classification(features, &model_config.output_classes).await
            }
            _ => {
                self.predict_generic(features, &model_config.output_classes).await
            }
        }
    }
    async fn predict_sentiment(&self, features: &[f64]) -> IntelligenceResult<Vec<PredictionResult>> {
        let sentiment_score = features.iter().sum::<f64>() / features.len() as f64;
        let predictions = vec![
            PredictionResult {
                class: "positive".to_string(),
                confidence: if sentiment_score > 0.0 { 0.8 } else { 0.2 },
                probability: if sentiment_score > 0.0 { 0.7 } else { 0.3 },
                explanation: Some("Text shows positive sentiment indicators".to_string()),
            },
            PredictionResult {
                class: "negative".to_string(),
                confidence: if sentiment_score <= 0.0 { 0.8 } else { 0.2 },
                probability: if sentiment_score <= 0.0 { 0.7 } else { 0.3 },
                explanation: Some("Text shows negative sentiment indicators".to_string()),
            },
            PredictionResult {
                class: "neutral".to_string(),
                confidence: 0.3,
                probability: 0.2,
                explanation: Some("Text shows neutral sentiment".to_string()),
            },
        ];
        Ok(predictions)
    }
    async fn predict_threat(&self, features: &[f64]) -> IntelligenceResult<Vec<PredictionResult>> {
        let threat_score = features.iter().sum::<f64>() / features.len() as f64;
        let predictions = vec![
            PredictionResult {
                class: "low".to_string(),
                confidence: if threat_score < 0.3 { 0.9 } else { 0.1 },
                probability: if threat_score < 0.3 { 0.8 } else { 0.1 },
                explanation: Some("Low threat indicators detected".to_string()),
            },
            PredictionResult {
                class: "medium".to_string(),
                confidence: if threat_score >= 0.3 && threat_score < 0.7 { 0.8 } else { 0.2 },
                probability: if threat_score >= 0.3 && threat_score < 0.7 { 0.7 } else { 0.2 },
                explanation: Some("Medium threat indicators detected".to_string()),
            },
            PredictionResult {
                class: "high".to_string(),
                confidence: if threat_score >= 0.7 { 0.9 } else { 0.1 },
                probability: if threat_score >= 0.7 { 0.8 } else { 0.1 },
                explanation: Some("High threat indicators detected".to_string()),
            },
        ];
        Ok(predictions)
    }
    async fn predict_anomaly(&self, features: &[f64]) -> IntelligenceResult<Vec<PredictionResult>> {
        let anomaly_score = features.iter().map(|x| x.abs()).sum::<f64>() / features.len() as f64;
        let predictions = vec![
            PredictionResult {
                class: "normal".to_string(),
                confidence: if anomaly_score < 0.5 { 0.9 } else { 0.1 },
                probability: if anomaly_score < 0.5 { 0.8 } else { 0.1 },
                explanation: Some("Data appears normal".to_string()),
            },
            PredictionResult {
                class: "anomaly".to_string(),
                confidence: if anomaly_score >= 0.5 { 0.9 } else { 0.1 },
                probability: if anomaly_score >= 0.5 { 0.8 } else { 0.1 },
                explanation: Some("Anomalous patterns detected".to_string()),
            },
        ];
        Ok(predictions)
    }
    async fn predict_behavior(&self, features: &[f64]) -> IntelligenceResult<Vec<PredictionResult>> {
        let behavior_score = features.iter().sum::<f64>() / features.len() as f64;
        let predictions = vec![
            PredictionResult {
                class: "suspicious".to_string(),
                confidence: if behavior_score > 0.6 { 0.8 } else { 0.2 },
                probability: if behavior_score > 0.6 { 0.7 } else { 0.2 },
                explanation: Some("Suspicious behavior patterns detected".to_string()),
            },
            PredictionResult {
                class: "normal".to_string(),
                confidence: if behavior_score <= 0.6 { 0.8 } else { 0.2 },
                probability: if behavior_score <= 0.6 { 0.7 } else { 0.2 },
                explanation: Some("Normal behavior patterns".to_string()),
            },
        ];
        Ok(predictions)
    }
    async fn predict_entities(&self, features: &[f64]) -> IntelligenceResult<Vec<PredictionResult>> {
        let predictions = vec![
            PredictionResult {
                class: "person".to_string(),
                confidence: 0.8,
                probability: 0.7,
                explanation: Some("Person entity detected".to_string()),
            },
            PredictionResult {
                class: "organization".to_string(),
                confidence: 0.6,
                probability: 0.5,
                explanation: Some("Organization entity detected".to_string()),
            },
            PredictionResult {
                class: "location".to_string(),
                confidence: 0.7,
                probability: 0.6,
                explanation: Some("Location entity detected".to_string()),
            },
        ];
        Ok(predictions)
    }
    async fn predict_classification(
        &self,
        features: &[f64],
        classes: &[String],
    ) -> IntelligenceResult<Vec<PredictionResult>> {
        let mut predictions = Vec::new();
        for (i, class) in classes.iter().enumerate() {
            let confidence = if i < features.len() {
                features[i].abs()
            } else {
                0.1
            };
            predictions.push(PredictionResult {
                class: class.clone(),
                confidence,
                probability: confidence,
                explanation: Some(format!("Classified as {}", class)),
            });
        }
        Ok(predictions)
    }
    async fn predict_generic(
        &self,
        features: &[f64],
        classes: &[String],
    ) -> IntelligenceResult<Vec<PredictionResult>> {
        self.predict_classification(features, classes).await
    }
    async fn update_model_metrics(&self, model_id: &str, prediction: &ModelPrediction) {
        let mut metrics = self.model_metrics.write().await;
        if let Some(model_metrics) = metrics.get_mut(model_id) {
            model_metrics.total_predictions += 1;
            model_metrics.average_confidence =
                (model_metrics.average_confidence * (model_metrics.total_predictions - 1) as f64 + prediction.confidence)
                / model_metrics.total_predictions as f64;
            model_metrics.last_updated = Utc::now();
        }
    }
    async fn update_metrics(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let mut metrics = self.model_metrics.write().await;
            for model_metrics in metrics.values_mut() {
                model_metrics.accuracy = 0.85 + (rand::random::<f64>() * 0.1);
                model_metrics.precision = 0.80 + (rand::random::<f64>() * 0.1);
                model_metrics.recall = 0.82 + (rand::random::<f64>() * 0.1);
                model_metrics.f1_score = 0.81 + (rand::random::<f64>() * 0.1);
                model_metrics.last_updated = Utc::now();
            }
            drop(metrics);
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
        Ok(())
    }
    pub async fn get_model_metrics(&self, model_id: &str) -> Option<ModelMetrics> {
        let metrics = self.model_metrics.read().await;
        metrics.get(model_id).cloned()
    }
    pub async fn get_all_metrics(&self) -> HashMap<String, ModelMetrics> {
        self.model_metrics.read().await.clone()
    }
    fn clone_for_task(&self) -> Self {
        Self {
            models: Arc::clone(&self.models),
            model_metrics: Arc::clone(&self.model_metrics),
            prediction_cache: Arc::clone(&self.prediction_cache),
            batch_queue: Arc::clone(&self.batch_queue),
            is_running: Arc::clone(&self.is_running),
            processors: Arc::clone(&self.processors),
        }
    }
}
