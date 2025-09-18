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

/// High-performance web scraper with advanced capabilities

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingConfig {
    pub max_concurrent_requests: usize,
    pub request_timeout_seconds: u64,
    pub rate_limit_per_second: u32,
    pub user_agents: Vec<String>,
    pub proxy_list: Vec<String>,
    pub respect_robots_txt: bool,
    pub follow_redirects: bool,
    pub max_redirects: u32,
    pub batch_size: usize,
    pub retry_attempts: u32,
    pub retry_delay_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingTarget {
    pub id: String,
    pub url: String,
    pub selector: Option<String>,
    pub extraction_rules: Vec<ExtractionRule>,
    pub priority: u32,
    pub enabled: bool,
    pub last_scraped: Option<DateTime<Utc>>,
    pub scrape_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRule {
    pub name: String,
    pub selector: String,
    pub attribute: Option<String>,
    pub regex: Option<String>,
    pub data_type: DataType,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataType {
    Text,
    Html,
    Link,
    Image,
    Number,
    Date,
    Email,
    Phone,
    SocialMedia,
    Cryptocurrency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedData {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub extracted_data: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub response_time_ms: u64,
    pub status_code: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingStats {
    pub urls_scraped: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub data_points_extracted: u64,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub throughput_per_second: f64,
    pub average_response_time: f64,
}

/// High-performance web scraper
pub struct WebScraper {
    config: ScrapingConfig,
    targets: Arc<RwLock<HashMap<String, ScrapingTarget>>>,
    stats: Arc<RwLock<ScrapingStats>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    data_buffer: Arc<RwLock<Vec<ScrapedData>>>,
    is_running: Arc<RwLock<bool>>,
    client: reqwest::Client,
}

struct RateLimiter {
    last_request: std::time::Instant,
    requests_per_second: u32,
}

impl WebScraper {
    pub fn new(config: ScrapingConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.request_timeout_seconds))
            .redirect(reqwest::redirect::Policy::limited(config.max_redirects as usize))
            .user_agent(&config.user_agents[0])
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            targets: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ScrapingStats {
                urls_scraped: 0,
                successful_requests: 0,
                failed_requests: 0,
                data_points_extracted: 0,
                start_time: Utc::now(),
                last_update: Utc::now(),
                throughput_per_second: 0.0,
                average_response_time: 0.0,
            })),
            rate_limiter: Arc::new(RwLock::new(RateLimiter {
                last_request: std::time::Instant::now(),
                requests_per_second: 30,
            })),
            data_buffer: Arc::new(RwLock::new(Vec::new())),
            is_running: Arc::new(RwLock::new(false)),
            client,
        }
    }

    /// Add scraping target
    pub async fn add_target(&self, target: ScrapingTarget) -> IntelligenceResult<()> {
        let mut targets = self.targets.write().await;
        targets.insert(target.id.clone(), target);
        info!("Added scraping target: {}", target.url);
        Ok(())
    }

    /// Remove scraping target
    pub async fn remove_target(&self, target_id: &str) -> IntelligenceResult<()> {
        let mut targets = self.targets.write().await;
        targets.remove(target_id);
        info!("Removed scraping target: {}", target_id);
        Ok(())
    }

    /// Start scraping process
    pub async fn start_scraping(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = true;
        drop(is_running);

        info!("Starting web scraping process");

        // Start scraping tasks
        let mut tasks = Vec::new();
        
        // Start target processor
        let scraper = self.clone_for_task();
        let target_task = tokio::spawn(async move {
            scraper.process_targets().await
        });
        tasks.push(target_task);

        // Start batch processor
        let scraper = self.clone_for_task();
        let batch_task = tokio::spawn(async move {
            scraper.process_batches().await
        });
        tasks.push(batch_task);

        // Wait for all tasks
        for task in tasks {
            if let Err(e) = task.await {
                error!("Scraping task failed: {}", e);
            }
        }

        Ok(())
    }

    /// Stop scraping process
    pub async fn stop_scraping(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("Web scraping stopped");
        Ok(())
    }

    /// Process all targets
    async fn process_targets(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let targets = {
                let targets = self.targets.read().await;
                targets.values().cloned().collect::<Vec<_>>()
            };

            let mut tasks = Vec::new();
            
            for target in targets {
                if !target.enabled {
                    continue;
                }

                // Check if it's time to scrape this target
                if let Some(last_scraped) = target.last_scraped {
                    let time_since_last = Utc::now().signed_duration_since(last_scraped);
                    if time_since_last.num_seconds() < target.scrape_interval_seconds as i64 {
                        continue;
                    }
                }

                let scraper = self.clone_for_task();
                let task = tokio::spawn(async move {
                    scraper.scrape_target(target).await
                });
                tasks.push(task);

                // Limit concurrent requests
                if tasks.len() >= self.config.max_concurrent_requests {
                    // Wait for some tasks to complete
                    for task in tasks.drain(0..self.config.max_concurrent_requests / 2) {
                        if let Err(e) = task.await {
                            error!("Scraping task failed: {}", e);
                        }
                    }
                }
            }

            // Wait for remaining tasks
            for task in tasks {
                if let Err(e) = task.await {
                    error!("Scraping task failed: {}", e);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }

        Ok(())
    }

    /// Scrape a specific target
    async fn scrape_target(&self, target: ScrapingTarget) -> IntelligenceResult<()> {
        let start_time = std::time::Instant::now();
        
        match self.fetch_page(&target.url).await {
            Ok(html_content) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                
                // Parse HTML and extract data
                let extracted_data = self.extract_data(&html_content, &target.extraction_rules).await?;
                
                let scraped_data = ScrapedData {
                    url: target.url.clone(),
                    title: self.extract_title(&html_content).await,
                    content: html_content,
                    extracted_data,
                    metadata: HashMap::new(),
                    timestamp: Utc::now(),
                    response_time_ms: response_time,
                    status_code: 200,
                };

                // Add to buffer
                {
                    let mut buffer = self.data_buffer.write().await;
                    buffer.push(scraped_data);
                }

                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.urls_scraped += 1;
                    stats.successful_requests += 1;
                    stats.data_points_extracted += extracted_data.len() as u64;
                    stats.last_update = Utc::now();
                }

                // Update target last scraped time
                {
                    let mut targets = self.targets.write().await;
                    if let Some(target_ref) = targets.get_mut(&target.id) {
                        target_ref.last_scraped = Some(Utc::now());
                    }
                }

                debug!("Successfully scraped: {}", target.url);
            }
            Err(e) => {
                error!("Failed to scrape {}: {}", target.url, e);
                
                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.failed_requests += 1;
                    stats.last_update = Utc::now();
                }
            }
        }

        // Rate limiting
        self.enforce_rate_limit().await;
        Ok(())
    }

    /// Fetch page content
    async fn fetch_page(&self, url: &str) -> IntelligenceResult<String> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| intelligence_core::IntelligenceError::ExternalService {
                service: "web_scraper".to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(intelligence_core::IntelligenceError::ExternalService {
                service: "web_scraper".to_string(),
                message: format!("HTTP error: {}", response.status()),
            });
        }

        let content = response.text().await
            .map_err(|e| intelligence_core::IntelligenceError::ExternalService {
                service: "web_scraper".to_string(),
                message: e.to_string(),
            })?;

        Ok(content)
    }

    /// Extract data from HTML using rules
    async fn extract_data(
        &self,
        html: &str,
        rules: &[ExtractionRule],
    ) -> IntelligenceResult<HashMap<String, serde_json::Value>> {
        let mut extracted_data = HashMap::new();

        // Parse HTML (simplified - in real implementation, use scraper crate)
        for rule in rules {
            match self.extract_with_rule(html, rule).await {
                Ok(value) => {
                    extracted_data.insert(rule.name.clone(), value);
                }
                Err(e) => {
                    if rule.required {
                        return Err(e);
                    }
                    warn!("Failed to extract required field {}: {}", rule.name, e);
                }
            }
        }

        Ok(extracted_data)
    }

    /// Extract data using a specific rule
    async fn extract_with_rule(
        &self,
        html: &str,
        rule: &ExtractionRule,
    ) -> IntelligenceResult<serde_json::Value> {
        // Simplified extraction (in real implementation, use proper HTML parsing)
        match rule.data_type {
            DataType::Text => {
                // Extract text content
                Ok(serde_json::Value::String("Extracted text".to_string()))
            }
            DataType::Link => {
                // Extract links
                Ok(serde_json::Value::String("https://example.com".to_string()))
            }
            DataType::Email => {
                // Extract email addresses using regex
                let email_regex = regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")?;
                if let Some(captures) = email_regex.find(html) {
                    Ok(serde_json::Value::String(captures.as_str().to_string()))
                } else {
                    Err(anyhow::anyhow!("No email found"))
                }
            }
            DataType::Phone => {
                // Extract phone numbers
                let phone_regex = regex::Regex::new(r"\+?[\d\s\-\(\)]{10,}")?;
                if let Some(captures) = phone_regex.find(html) {
                    Ok(serde_json::Value::String(captures.as_str().to_string()))
                } else {
                    Err(anyhow::anyhow!("No phone number found"))
                }
            }
            DataType::Cryptocurrency => {
                // Extract crypto addresses
                let btc_regex = regex::Regex::new(r"[13][a-km-zA-HJ-NP-Z1-9]{25,34}")?;
                let eth_regex = regex::Regex::new(r"0x[a-fA-F0-9]{40}")?;
                
                if let Some(captures) = btc_regex.find(html) {
                    Ok(serde_json::Value::String(captures.as_str().to_string()))
                } else if let Some(captures) = eth_regex.find(html) {
                    Ok(serde_json::Value::String(captures.as_str().to_string()))
                } else {
                    Err(anyhow::anyhow!("No crypto address found"))
                }
            }
            _ => {
                // Default extraction
                Ok(serde_json::Value::String("Extracted data".to_string()))
            }
        }
    }

    /// Extract page title
    async fn extract_title(&self, html: &str) -> Option<String> {
        // Simplified title extraction (in real implementation, use proper HTML parsing)
        if html.contains("<title>") {
            Some("Page Title".to_string())
        } else {
            None
        }
    }

    /// Process scraped data batches
    async fn process_batches(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let batch = {
                let mut buffer = self.data_buffer.write().await;
                if buffer.len() >= self.config.batch_size {
                    let batch = buffer.drain(0..self.config.batch_size).collect::<Vec<_>>();
                    batch
                } else {
                    Vec::new()
                }
            };

            if !batch.is_empty() {
                let intelligence_data = self.convert_to_intelligence_data(batch).await?;
                
                // Here you would send to the data processing pipeline
                info!("Processed batch of {} scraped pages into {} intelligence data points", 
                      batch.len(), intelligence_data.len());
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Convert scraped data to IntelligenceData format
    async fn convert_to_intelligence_data(
        &self,
        scraped_data: Vec<ScrapedData>,
    ) -> IntelligenceResult<Vec<IntelligenceData>> {
        let mut intelligence_data = Vec::new();

        for data in scraped_data {
            let content = serde_json::json!({
                "url": data.url,
                "title": data.title,
                "content": data.content,
                "extracted_data": data.extracted_data,
                "metadata": data.metadata,
                "response_time_ms": data.response_time_ms,
                "status_code": data.status_code
            });

            let intel_data = IntelligenceData {
                id: IntelligenceId::new(),
                content: content.to_string(),
                source: DataSource::WebScraping,
                classification: DataClassification::Public,
                metadata: HashMap::new(),
                timestamp: data.timestamp,
                confidence: 0.9,
                quality_score: 0.8,
            };

            intelligence_data.push(intel_data);
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

    /// Get scraping statistics
    pub async fn get_stats(&self) -> ScrapingStats {
        self.stats.read().await.clone()
    }

    /// Clone scraper for task execution
    fn clone_for_task(&self) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.request_timeout_seconds))
            .redirect(reqwest::redirect::Policy::limited(self.config.max_redirects as usize))
            .user_agent(&self.config.user_agents[0])
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config: self.config.clone(),
            targets: Arc::clone(&self.targets),
            stats: Arc::clone(&self.stats),
            rate_limiter: Arc::clone(&self.rate_limiter),
            data_buffer: Arc::clone(&self.data_buffer),
            is_running: Arc::clone(&self.is_running),
            client,
        }
    }
}
