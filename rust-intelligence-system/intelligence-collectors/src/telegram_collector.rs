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

/// High-performance Telegram data collector

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub api_id: i32,
    pub api_hash: String,
    pub phone_number: String,
    pub session_string: Option<String>,
    pub max_concurrent_sessions: usize,
    pub rate_limit_per_second: u32,
    pub batch_size: usize,
    pub collection_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramMessage {
    pub id: i32,
    pub chat_id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub text: String,
    pub date: DateTime<Utc>,
    pub reply_to_message_id: Option<i32>,
    pub forward_from: Option<i64>,
    pub media_type: Option<String>,
    pub entities: Vec<MessageEntity>,
    pub views: Option<i32>,
    pub forwards: Option<i32>,
    pub reactions: Vec<Reaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEntity {
    pub entity_type: String,
    pub offset: i32,
    pub length: i32,
    pub url: Option<String>,
    pub user_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: String,
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUser {
    pub id: i64,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub is_bot: bool,
    pub is_verified: bool,
    pub is_premium: bool,
    pub last_seen: Option<DateTime<Utc>>,
    pub bio: Option<String>,
    pub common_chats_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramChat {
    pub id: i64,
    pub title: Option<String>,
    pub username: Option<String>,
    pub chat_type: String,
    pub members_count: Option<i32>,
    pub description: Option<String>,
    pub invite_link: Option<String>,
    pub is_verified: bool,
    pub is_scam: bool,
    pub is_fake: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    pub messages_collected: u64,
    pub users_collected: u64,
    pub chats_collected: u64,
    pub errors_count: u64,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub throughput_per_second: f64,
}

/// High-performance Telegram collector
pub struct TelegramCollector {
    config: TelegramConfig,
    client: Option<telegram_bot::Api>,
    stats: Arc<RwLock<CollectionStats>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    message_buffer: Arc<RwLock<Vec<TelegramMessage>>>,
    user_cache: Arc<RwLock<HashMap<i64, TelegramUser>>>,
    chat_cache: Arc<RwLock<HashMap<i64, TelegramChat>>>,
    is_running: Arc<RwLock<bool>>,
}

struct RateLimiter {
    last_request: std::time::Instant,
    requests_per_second: u32,
}

impl TelegramCollector {
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            config,
            client: None,
            stats: Arc::new(RwLock::new(CollectionStats {
                messages_collected: 0,
                users_collected: 0,
                chats_collected: 0,
                errors_count: 0,
                start_time: Utc::now(),
                last_update: Utc::now(),
                throughput_per_second: 0.0,
            })),
            rate_limiter: Arc::new(RwLock::new(RateLimiter {
                last_request: std::time::Instant::now(),
                requests_per_second: 30, // Telegram API limit
            })),
            message_buffer: Arc::new(RwLock::new(Vec::new())),
            user_cache: Arc::new(RwLock::new(HashMap::new())),
            chat_cache: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize Telegram client
    pub async fn initialize(&mut self) -> IntelligenceResult<()> {
        info!("Initializing Telegram collector...");
        
        // Create Telegram client
        let client = telegram_bot::Api::new(&self.config.api_hash)
            .map_err(|e| intelligence_core::IntelligenceError::ExternalService {
                service: "telegram".to_string(),
                message: e.to_string(),
            })?;

        self.client = Some(client);
        info!("Telegram collector initialized successfully");
        Ok(())
    }

    /// Start continuous data collection
    pub async fn start_collection(&self, target_chats: Vec<i64>) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = true;
        drop(is_running);

        info!("Starting Telegram data collection for {} chats", target_chats.len());

        // Start collection tasks
        let mut tasks = Vec::new();
        
        for chat_id in target_chats {
            let collector = self.clone_for_task();
            let task = tokio::spawn(async move {
                collector.collect_chat_messages(chat_id).await
            });
            tasks.push(task);
        }

        // Start user profile collection
        let collector = self.clone_for_task();
        let user_task = tokio::spawn(async move {
            collector.collect_user_profiles().await
        });
        tasks.push(user_task);

        // Start chat information collection
        let collector = self.clone_for_task();
        let chat_task = tokio::spawn(async move {
            collector.collect_chat_information().await
        });
        tasks.push(chat_task);

        // Start batch processor
        let collector = self.clone_for_task();
        let batch_task = tokio::spawn(async move {
            collector.process_batches().await
        });
        tasks.push(batch_task);

        // Wait for all tasks
        for task in tasks {
            if let Err(e) = task.await {
                error!("Collection task failed: {}", e);
            }
        }

        Ok(())
    }

    /// Stop data collection
    pub async fn stop_collection(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("Telegram collection stopped");
        Ok(())
    }

    /// Collect messages from a specific chat
    async fn collect_chat_messages(&self, chat_id: i64) -> IntelligenceResult<()> {
        let mut offset_id = 0;
        let mut total_collected = 0;

        while *self.is_running.read().await {
            match self.fetch_messages_batch(chat_id, offset_id, self.config.batch_size).await {
                Ok(messages) => {
                    if messages.is_empty() {
                        // No more messages, wait before next poll
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }

                    // Add to buffer
                    {
                        let mut buffer = self.message_buffer.write().await;
                        buffer.extend(messages.clone());
                    }

                    // Update stats
                    {
                        let mut stats = self.stats.write().await;
                        stats.messages_collected += messages.len() as u64;
                        stats.last_update = Utc::now();
                        total_collected += messages.len();
                    }

                    offset_id = messages.last().map(|m| m.id).unwrap_or(offset_id);
                    
                    debug!("Collected {} messages from chat {}, total: {}", 
                           messages.len(), chat_id, total_collected);

                    // Rate limiting
                    self.enforce_rate_limit().await;
                }
                Err(e) => {
                    error!("Failed to fetch messages from chat {}: {}", chat_id, e);
                    {
                        let mut stats = self.stats.write().await;
                        stats.errors_count += 1;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        }

        Ok(())
    }

    /// Fetch a batch of messages from Telegram
    async fn fetch_messages_batch(
        &self,
        chat_id: i64,
        offset_id: i32,
        limit: usize,
    ) -> IntelligenceResult<Vec<TelegramMessage>> {
        // Simulate Telegram API call (replace with actual implementation)
        let mut messages = Vec::new();
        
        for i in 0..limit {
            let message = TelegramMessage {
                id: offset_id + i as i32,
                chat_id,
                user_id: Some(12345 + i as i64),
                username: Some(format!("user_{}", i)),
                text: format!("Sample message {} from chat {}", i, chat_id),
                date: Utc::now() - chrono::Duration::minutes(i as i64),
                reply_to_message_id: None,
                forward_from: None,
                media_type: None,
                entities: vec![],
                views: Some(100 + i as i32),
                forwards: Some(i as i32),
                reactions: vec![],
            };
            messages.push(message);
        }

        Ok(messages)
    }

    /// Collect user profiles
    async fn collect_user_profiles(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            // Get unique user IDs from messages
            let user_ids = {
                let buffer = self.message_buffer.read().await;
                buffer.iter()
                    .filter_map(|msg| msg.user_id)
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
            };

            for user_id in user_ids {
                if self.user_cache.read().await.contains_key(&user_id) {
                    continue; // Already cached
                }

                match self.fetch_user_profile(user_id).await {
                    Ok(user) => {
                        self.user_cache.write().await.insert(user_id, user);
                        {
                            let mut stats = self.stats.write().await;
                            stats.users_collected += 1;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to fetch user profile {}: {}", user_id, e);
                    }
                }

                self.enforce_rate_limit().await;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }

        Ok(())
    }

    /// Fetch user profile from Telegram
    async fn fetch_user_profile(&self, user_id: i64) -> IntelligenceResult<TelegramUser> {
        // Simulate Telegram API call (replace with actual implementation)
        let user = TelegramUser {
            id: user_id,
            username: Some(format!("user_{}", user_id)),
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            phone: Some("+1234567890".to_string()),
            is_bot: false,
            is_verified: false,
            is_premium: false,
            last_seen: Some(Utc::now()),
            bio: Some("Sample bio".to_string()),
            common_chats_count: Some(5),
        };

        Ok(user)
    }

    /// Collect chat information
    async fn collect_chat_information(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            // Get unique chat IDs from messages
            let chat_ids = {
                let buffer = self.message_buffer.read().await;
                buffer.iter()
                    .map(|msg| msg.chat_id)
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
            };

            for chat_id in chat_ids {
                if self.chat_cache.read().await.contains_key(&chat_id) {
                    continue; // Already cached
                }

                match self.fetch_chat_info(chat_id).await {
                    Ok(chat) => {
                        self.chat_cache.write().await.insert(chat_id, chat);
                        {
                            let mut stats = self.stats.write().await;
                            stats.chats_collected += 1;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to fetch chat info {}: {}", chat_id, e);
                    }
                }

                self.enforce_rate_limit().await;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }

        Ok(())
    }

    /// Fetch chat information from Telegram
    async fn fetch_chat_info(&self, chat_id: i64) -> IntelligenceResult<TelegramChat> {
        // Simulate Telegram API call (replace with actual implementation)
        let chat = TelegramChat {
            id: chat_id,
            title: Some(format!("Chat {}", chat_id)),
            username: Some(format!("chat_{}", chat_id)),
            chat_type: "supergroup".to_string(),
            members_count: Some(1000),
            description: Some("Sample chat description".to_string()),
            invite_link: Some(format!("https://t.me/chat_{}", chat_id)),
            is_verified: false,
            is_scam: false,
            is_fake: false,
        };

        Ok(chat)
    }

    /// Process message batches and convert to IntelligenceData
    async fn process_batches(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let batch = {
                let mut buffer = self.message_buffer.write().await;
                if buffer.len() >= self.config.batch_size {
                    let batch = buffer.drain(0..self.config.batch_size).collect::<Vec<_>>();
                    batch
                } else {
                    Vec::new()
                }
            };

            if !batch.is_empty() {
                let intelligence_data = self.convert_messages_to_intelligence_data(batch).await?;
                
                // Here you would send to the data processing pipeline
                info!("Processed batch of {} messages into {} intelligence data points", 
                      batch.len(), intelligence_data.len());
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Convert Telegram messages to IntelligenceData format
    async fn convert_messages_to_intelligence_data(
        &self,
        messages: Vec<TelegramMessage>,
    ) -> IntelligenceResult<Vec<IntelligenceData>> {
        let mut intelligence_data = Vec::new();

        for message in messages {
            let content = serde_json::json!({
                "message_id": message.id,
                "chat_id": message.chat_id,
                "user_id": message.user_id,
                "username": message.username,
                "text": message.text,
                "date": message.date,
                "views": message.views,
                "forwards": message.forwards,
                "entities": message.entities,
                "reactions": message.reactions
            });

            let data = IntelligenceData {
                id: IntelligenceId::new(),
                content: content.to_string(),
                source: DataSource::Telegram,
                classification: DataClassification::Internal,
                metadata: HashMap::new(),
                timestamp: message.date,
                confidence: 1.0,
                quality_score: 0.9,
            };

            intelligence_data.push(data);
        }

        Ok(intelligence_data)
    }

    /// Enforce rate limiting
    async fn enforce_rate_limit(&self) {
        let mut limiter = self.rate_limiter.write().await;
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(limiter.last_request);
        
        let min_interval = std::time::Duration::from_millis(1000 / limiter.requests_per_second as u64);
        if elapsed < min_interval {
            tokio::time::sleep(min_interval - elapsed).await;
        }
        
        limiter.last_request = std::time::Instant::now();
    }

    /// Get collection statistics
    pub async fn get_stats(&self) -> CollectionStats {
        self.stats.read().await.clone()
    }

    /// Clone collector for task execution
    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: None, // Will be recreated in each task
            stats: Arc::clone(&self.stats),
            rate_limiter: Arc::clone(&self.rate_limiter),
            message_buffer: Arc::clone(&self.message_buffer),
            user_cache: Arc::clone(&self.user_cache),
            chat_cache: Arc::clone(&self.chat_cache),
            is_running: Arc::clone(&self.is_running),
        }
    }
}
