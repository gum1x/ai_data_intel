use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, warn, error, debug};

use intelligence_core::{
    IntelligenceData, DataSource, DataClassification, IntelligenceId, Result as IntelligenceResult
};

/// High-throughput streaming data processor

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    pub kafka_bootstrap_servers: Vec<String>,
    pub consumer_group_id: String,
    pub topics: Vec<String>,
    pub batch_size: usize,
    pub processing_timeout_ms: u64,
    pub max_concurrent_processors: usize,
    pub buffer_size: usize,
    pub commit_interval_ms: u64,
    pub auto_offset_reset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessage {
    pub id: Uuid,
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub key: Option<Vec<u8>>,
    pub value: Vec<u8>,
    pub headers: HashMap<String, Vec<u8>>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub message_id: Uuid,
    pub success: bool,
    pub processing_time_ms: u64,
    pub output_data: Vec<IntelligenceData>,
    pub errors: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStats {
    pub messages_processed: u64,
    pub successful_processing: u64,
    pub failed_processing: u64,
    pub total_throughput_per_second: f64,
    pub average_processing_time: f64,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub topic_stats: HashMap<String, TopicStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicStats {
    pub messages_processed: u64,
    pub successful_processing: u64,
    pub failed_processing: u64,
    pub throughput_per_second: f64,
    pub average_processing_time: f64,
}

/// High-performance stream processor
pub struct StreamProcessor {
    config: StreamConfig,
    stats: Arc<RwLock<StreamStats>>,
    message_buffer: Arc<RwLock<Vec<StreamMessage>>>,
    is_running: Arc<RwLock<bool>>,
    processors: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

impl StreamProcessor {
    pub fn new(config: StreamConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(StreamStats {
                messages_processed: 0,
                successful_processing: 0,
                failed_processing: 0,
                total_throughput_per_second: 0.0,
                average_processing_time: 0.0,
                start_time: Utc::now(),
                last_update: Utc::now(),
                topic_stats: HashMap::new(),
            })),
            message_buffer: Arc::new(RwLock::new(Vec::new())),
            is_running: Arc::new(RwLock::new(false)),
            processors: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start stream processing
    pub async fn start(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = true;
        drop(is_running);

        info!("Starting stream processor for topics: {:?}", self.config.topics);

        // Start Kafka consumer
        let processor = self.clone_for_task();
        let consumer_task = tokio::spawn(async move {
            processor.consume_messages().await
        });

        // Start message processors
        let mut tasks = Vec::new();
        for i in 0..self.config.max_concurrent_processors {
            let processor = self.clone_for_task();
            let task = tokio::spawn(async move {
                processor.process_messages().await
            });
            tasks.push(task);
        }

        // Start stats updater
        let processor = self.clone_for_task();
        let stats_task = tokio::spawn(async move {
            processor.update_stats().await
        });
        tasks.push(stats_task);

        // Store processor tasks
        {
            let mut processors = self.processors.write().await;
            processors.push(consumer_task);
            processors.extend(tasks);
        }

        info!("Stream processor started with {} concurrent processors", 
              self.config.max_concurrent_processors);

        Ok(())
    }

    /// Stop stream processing
    pub async fn stop(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        drop(is_running);

        // Wait for all processors to finish
        let mut processors = self.processors.write().await;
        for processor in processors.drain(..) {
            processor.abort();
        }

        info!("Stream processor stopped");
        Ok(())
    }

    /// Consume messages from Kafka
    async fn consume_messages(&self) -> IntelligenceResult<()> {
        // Simulate Kafka consumer (replace with actual Kafka consumer implementation)
        let mut message_id = 0;
        
        while *self.is_running.read().await {
            // Simulate receiving messages
            let messages = self.simulate_kafka_messages().await;
            
            for message in messages {
                // Add to buffer
                {
                    let mut buffer = self.message_buffer.write().await;
                    buffer.push(message);
                    
                    // Limit buffer size
                    if buffer.len() > self.config.buffer_size {
                        buffer.drain(0..buffer.len() - self.config.buffer_size);
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        Ok(())
    }

    /// Simulate Kafka messages (replace with actual Kafka consumer)
    async fn simulate_kafka_messages(&self) -> Vec<StreamMessage> {
        let mut messages = Vec::new();
        
        for topic in &self.config.topics {
            for i in 0..10 { // Simulate batch of messages
                let message = StreamMessage {
                    id: Uuid::new_v4(),
                    topic: topic.clone(),
                    partition: 0,
                    offset: i,
                    key: Some(format!("key_{}", i).into_bytes()),
                    value: serde_json::json!({
                        "message_id": i,
                        "content": format!("Sample message {} from topic {}", i, topic),
                        "timestamp": Utc::now(),
                        "source": topic,
                        "data": {
                            "user_id": 12345 + i,
                            "action": "message_sent",
                            "metadata": {}
                        }
                    }).to_string().into_bytes(),
                    headers: HashMap::new(),
                    timestamp: Utc::now(),
                };
                messages.push(message);
            }
        }
        
        messages
    }

    /// Process messages from buffer
    async fn process_messages(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let batch = {
                let mut buffer = self.message_buffer.write().await;
                if buffer.len() >= self.config.batch_size {
                    let batch = buffer.drain(0..self.config.batch_size).collect::<Vec<_>>();
                    batch
                } else if !buffer.is_empty() {
                    let batch = buffer.drain(..).collect::<Vec<_>>();
                    batch
                } else {
                    Vec::new()
                }
            };

            if !batch.is_empty() {
                for message in batch {
                    let start_time = std::time::Instant::now();
                    
                    match self.process_message(message).await {
                        Ok(result) => {
                            let processing_time = start_time.elapsed().as_millis() as u64;
                            
                            // Update stats
                            {
                                let mut stats = self.stats.write().await;
                                stats.messages_processed += 1;
                                if result.success {
                                    stats.successful_processing += 1;
                                } else {
                                    stats.failed_processing += 1;
                                }
                                
                                // Update topic stats
                                let topic_stats = stats.topic_stats.entry(result.metadata.get("topic")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown")
                                    .to_string())
                                    .or_insert_with(|| TopicStats {
                                        messages_processed: 0,
                                        successful_processing: 0,
                                        failed_processing: 0,
                                        throughput_per_second: 0.0,
                                        average_processing_time: 0.0,
                                    });
                                
                                topic_stats.messages_processed += 1;
                                if result.success {
                                    topic_stats.successful_processing += 1;
                                } else {
                                    topic_stats.failed_processing += 1;
                                }
                                
                                stats.last_update = Utc::now();
                            }
                            
                            debug!("Processed message {} in {}ms", result.message_id, processing_time);
                        }
                        Err(e) => {
                            error!("Failed to process message: {}", e);
                            
                            // Update stats
                            {
                                let mut stats = self.stats.write().await;
                                stats.messages_processed += 1;
                                stats.failed_processing += 1;
                                stats.last_update = Utc::now();
                            }
                        }
                    }
                }
            } else {
                // No messages to process, wait a bit
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }

        Ok(())
    }

    /// Process a single message
    async fn process_message(&self, message: StreamMessage) -> IntelligenceResult<ProcessingResult> {
        let start_time = std::time::Instant::now();
        let mut errors = Vec::new();
        let mut output_data = Vec::new();

        // Parse message value
        let message_data: serde_json::Value = match serde_json::from_slice(&message.value) {
            Ok(data) => data,
            Err(e) => {
                errors.push(format!("Failed to parse message JSON: {}", e));
                return Ok(ProcessingResult {
                    message_id: message.id,
                    success: false,
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    output_data: Vec::new(),
                    errors,
                    metadata: HashMap::new(),
                });
            }
        };

        // Process based on topic
        match message.topic.as_str() {
            "telegram_messages" => {
                output_data = self.process_telegram_message(message_data).await?;
            }
            "web_scraping" => {
                output_data = self.process_web_scraping_data(message_data).await?;
            }
            "api_data" => {
                output_data = self.process_api_data(message_data).await?;
            }
            "blockchain_data" => {
                output_data = self.process_blockchain_data(message_data).await?;
            }
            _ => {
                output_data = self.process_generic_data(message_data).await?;
            }
        }

        let processing_time = start_time.elapsed().as_millis() as u64;
        let success = errors.is_empty();

        let mut metadata = HashMap::new();
        metadata.insert("topic".to_string(), serde_json::Value::String(message.topic));
        metadata.insert("partition".to_string(), serde_json::Value::Number(serde_json::Number::from(message.partition)));
        metadata.insert("offset".to_string(), serde_json::Value::Number(serde_json::Number::from(message.offset)));

        Ok(ProcessingResult {
            message_id: message.id,
            success,
            processing_time_ms: processing_time,
            output_data,
            errors,
            metadata,
        })
    }

    /// Process Telegram message data
    async fn process_telegram_message(&self, data: serde_json::Value) -> IntelligenceResult<Vec<IntelligenceData>> {
        let mut intelligence_data = Vec::new();

        let content = serde_json::json!({
            "message_id": data.get("message_id"),
            "content": data.get("content"),
            "timestamp": data.get("timestamp"),
            "source": "telegram",
            "raw_data": data
        });

        let intel_data = IntelligenceData {
            id: IntelligenceId::new(),
            content: content.to_string(),
            source: DataSource::Telegram,
            classification: DataClassification::Internal,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            confidence: 0.95,
            quality_score: 0.9,
        };

        intelligence_data.push(intel_data);
        Ok(intelligence_data)
    }

    /// Process web scraping data
    async fn process_web_scraping_data(&self, data: serde_json::Value) -> IntelligenceResult<Vec<IntelligenceData>> {
        let mut intelligence_data = Vec::new();

        let content = serde_json::json!({
            "url": data.get("url"),
            "content": data.get("content"),
            "extracted_data": data.get("extracted_data"),
            "timestamp": data.get("timestamp"),
            "source": "web_scraping",
            "raw_data": data
        });

        let intel_data = IntelligenceData {
            id: IntelligenceId::new(),
            content: content.to_string(),
            source: DataSource::WebScraping,
            classification: DataClassification::Public,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            confidence: 0.85,
            quality_score: 0.8,
        };

        intelligence_data.push(intel_data);
        Ok(intelligence_data)
    }

    /// Process API data
    async fn process_api_data(&self, data: serde_json::Value) -> IntelligenceResult<Vec<IntelligenceData>> {
        let mut intelligence_data = Vec::new();

        let content = serde_json::json!({
            "api_response": data,
            "timestamp": Utc::now(),
            "source": "api"
        });

        let intel_data = IntelligenceData {
            id: IntelligenceId::new(),
            content: content.to_string(),
            source: DataSource::Api,
            classification: DataClassification::Internal,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            confidence: 0.9,
            quality_score: 0.85,
        };

        intelligence_data.push(intel_data);
        Ok(intelligence_data)
    }

    /// Process blockchain data
    async fn process_blockchain_data(&self, data: serde_json::Value) -> IntelligenceResult<Vec<IntelligenceData>> {
        let mut intelligence_data = Vec::new();

        let content = serde_json::json!({
            "blockchain_data": data,
            "timestamp": Utc::now(),
            "source": "blockchain"
        });

        let intel_data = IntelligenceData {
            id: IntelligenceId::new(),
            content: content.to_string(),
            source: DataSource::Blockchain,
            classification: DataClassification::Public,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            confidence: 1.0,
            quality_score: 0.95,
        };

        intelligence_data.push(intel_data);
        Ok(intelligence_data)
    }

    /// Process generic data
    async fn process_generic_data(&self, data: serde_json::Value) -> IntelligenceResult<Vec<IntelligenceData>> {
        let mut intelligence_data = Vec::new();

        let content = serde_json::json!({
            "data": data,
            "timestamp": Utc::now(),
            "source": "generic"
        });

        let intel_data = IntelligenceData {
            id: IntelligenceId::new(),
            content: content.to_string(),
            source: DataSource::Database,
            classification: DataClassification::Internal,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            confidence: 0.8,
            quality_score: 0.7,
        };

        intelligence_data.push(intel_data);
        Ok(intelligence_data)
    }

    /// Update processing statistics
    async fn update_stats(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let mut stats = self.stats.write().await;
            
            // Calculate throughput
            let elapsed = Utc::now().signed_duration_since(stats.start_time);
            if elapsed.num_seconds() > 0 {
                stats.total_throughput_per_second = stats.messages_processed as f64 / elapsed.num_seconds() as f64;
                
                // Update topic throughputs
                for topic_stats in stats.topic_stats.values_mut() {
                    topic_stats.throughput_per_second = topic_stats.messages_processed as f64 / elapsed.num_seconds() as f64;
                }
            }
            
            stats.last_update = Utc::now();
            drop(stats);

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        Ok(())
    }

    /// Get processing statistics
    pub async fn get_stats(&self) -> StreamStats {
        self.stats.read().await.clone()
    }

    /// Clone processor for task execution
    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            stats: Arc::clone(&self.stats),
            message_buffer: Arc::clone(&self.message_buffer),
            is_running: Arc::clone(&self.is_running),
            processors: Arc::clone(&self.processors),
        }
    }
}
