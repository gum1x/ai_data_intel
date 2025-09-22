use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, warn, error, debug};
use reqwest::{Client, ClientBuilder, header::{HeaderMap, HeaderValue, USER_AGENT}};
use scraper::{Html, Selector};
use regex::Regex;
use intelligence_core::{
    IntelligenceData, DataSource, DataClassification, IntelligenceId, Result as IntelligenceResult
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealScrapingConfig {
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
    Email,
    Phone,
    Date,
    Number,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingResult {
    pub target_id: String,
    pub url: String,
    pub extracted_data: HashMap<String, serde_json::Value>,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub content_length: usize,
    pub timestamp: DateTime<Utc>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_data_extracted: u64,
    pub average_response_time_ms: f64,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
}

pub struct RealWebScraper {
    config: RealScrapingConfig,
    http_client: Client,
    targets: Arc<RwLock<HashMap<String, ScrapingTarget>>>,
    results: Arc<RwLock<Vec<ScrapingResult>>>,
    stats: Arc<RwLock<ScrapingStats>>,
    is_running: Arc<RwLock<bool>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

struct RateLimiter {
    last_request: std::time::Instant,
    requests_per_second: u32,
}

impl RealWebScraper {
    pub fn new(config: RealScrapingConfig) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
        ));

        let client = ClientBuilder::new()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(config.request_timeout_seconds))
            .redirect(reqwest::redirect::Policy::limited(config.max_redirects as usize))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client: client,
            targets: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(ScrapingStats {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                total_data_extracted: 0,
                average_response_time_ms: 0.0,
                start_time: Utc::now(),
                last_update: Utc::now(),
            })),
            is_running: Arc::new(RwLock::new(false)),
            rate_limiter: Arc::new(RwLock::new(RateLimiter {
                last_request: std::time::Instant::now(),
                requests_per_second: 1,
            })),
        }
    }

    pub async fn add_target(&self, target: ScrapingTarget) -> IntelligenceResult<()> {
        let mut targets = self.targets.write().await;
        targets.insert(target.id.clone(), target);
        info!("Added scraping target: {}", target.url);
        Ok(())
    }

    pub async fn start_scraping(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = true;
        drop(is_running);

        info!("Starting web scraping with {} targets", self.targets.read().await.len());

        // Start scraping tasks
        let scraper = self.clone_for_task();
        let scraping_task = tokio::spawn(async move {
            scraper.scraping_loop().await
        });

        // Start stats update task
        let scraper = self.clone_for_task();
        let stats_task = tokio::spawn(async move {
            scraper.update_stats().await
        });

        // Wait for tasks
        tokio::try_join!(scraping_task, stats_task)
            .map_err(|e| intelligence_core::IntelligenceError::Internal {
                message: format!("Scraping task failed: {}", e),
            })?;

        Ok(())
    }

    pub async fn stop_scraping(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("Web scraping stopped");
        Ok(())
    }

    async fn scraping_loop(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let targets = {
                let targets = self.targets.read().await;
                targets.values()
                    .filter(|target| target.enabled)
                    .filter(|target| {
                        if let Some(last_scraped) = target.last_scraped {
                            let now = Utc::now();
                            now.signed_duration_since(last_scraped).num_seconds() >= target.scrape_interval_seconds as i64
                        } else {
                            true
                        }
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            };

            if targets.is_empty() {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                continue;
            }

            // Process targets in batches
            for batch in targets.chunks(self.config.batch_size) {
                let mut tasks = Vec::new();
                
                for target in batch {
                    let scraper = self.clone_for_task();
                    let target = target.clone();
                    let task = tokio::spawn(async move {
                        scraper.scrape_target(target).await
                    });
                    tasks.push(task);
                }

                // Wait for batch to complete
                for task in tasks {
                    if let Err(e) = task.await {
                        error!("Scraping task failed: {}", e);
                    }
                }

                // Rate limiting between batches
                self.enforce_rate_limit().await;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        Ok(())
    }

    async fn scrape_target(&self, target: ScrapingTarget) -> IntelligenceResult<()> {
        let start_time = std::time::Instant::now();
        
        match self.fetch_page(&target.url).await {
            Ok(html_content) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                
                let extracted_data = self.extract_data(&html_content, &target.extraction_rules).await;
                
                let result = ScrapingResult {
                    target_id: target.id.clone(),
                    url: target.url.clone(),
                    extracted_data,
                    status_code: 200,
                    response_time_ms: response_time,
                    content_length: html_content.len(),
                    timestamp: Utc::now(),
                    errors: Vec::new(),
                };

                // Store result
                {
                    let mut results = self.results.write().await;
                    results.push(result);
                    
                    // Keep only last 1000 results
                    if results.len() > 1000 {
                        results.drain(0..results.len() - 1000);
                    }
                }

                // Update target last scraped time
                {
                    let mut targets = self.targets.write().await;
                    if let Some(target) = targets.get_mut(&target.id) {
                        target.last_scraped = Some(Utc::now());
                    }
                }

                // Convert to intelligence data
                self.convert_to_intelligence_data(&target, &extracted_data).await?;

                info!("Successfully scraped target: {}", target.url);
            }
            Err(e) => {
                error!("Failed to scrape target {}: {}", target.url, e);
                
                let result = ScrapingResult {
                    target_id: target.id.clone(),
                    url: target.url.clone(),
                    extracted_data: HashMap::new(),
                    status_code: 0,
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    content_length: 0,
                    timestamp: Utc::now(),
                    errors: vec![e.to_string()],
                };

                let mut results = self.results.write().await;
                results.push(result);
            }
        }

        Ok(())
    }

    async fn fetch_page(&self, url: &str) -> Result<String> {
        self.enforce_rate_limit().await;

        let response = self.http_client
            .get(url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        let content = response.text().await?;
        Ok(content)
    }

    async fn extract_data(&self, html_content: &str, rules: &[ExtractionRule]) -> HashMap<String, serde_json::Value> {
        let mut extracted_data = HashMap::new();
        let document = Html::parse_document(html_content);

        for rule in rules {
            match self.extract_with_rule(&document, rule).await {
                Ok(value) => {
                    extracted_data.insert(rule.name.clone(), value);
                }
                Err(e) => {
                    if rule.required {
                        warn!("Failed to extract required field '{}': {}", rule.name, e);
                    }
                }
            }
        }

        extracted_data
    }

    async fn extract_with_rule(&self, document: &Html, rule: &ExtractionRule) -> Result<serde_json::Value> {
        let selector = Selector::parse(&rule.selector)
            .map_err(|e| anyhow::anyhow!("Invalid selector '{}': {}", rule.selector, e))?;

        let elements: Vec<_> = document.select(&selector).collect();
        
        if elements.is_empty() {
            return Err(anyhow::anyhow!("No elements found for selector: {}", rule.selector));
        }

        let mut results = Vec::new();
        
        for element in elements {
            let value = match rule.data_type {
                DataType::Text => {
                    serde_json::Value::String(element.text().collect::<String>().trim().to_string())
                }
                DataType::Html => {
                    serde_json::Value::String(element.html())
                }
                DataType::Link => {
                    if let Some(href) = element.value().attr("href") {
                        serde_json::Value::String(href.to_string())
                    } else {
                        continue;
                    }
                }
                DataType::Image => {
                    if let Some(src) = element.value().attr("src") {
                        serde_json::Value::String(src.to_string())
                    } else {
                        continue;
                    }
                }
                DataType::Email => {
                    let text = element.text().collect::<String>();
                    if let Ok(regex) = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}") {
                        if let Some(email) = regex.find(&text) {
                            serde_json::Value::String(email.as_str().to_string())
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                DataType::Phone => {
                    let text = element.text().collect::<String>();
                    if let Ok(regex) = Regex::new(r"\+?[\d\s\-\(\)]{10,}") {
                        if let Some(phone) = regex.find(&text) {
                            serde_json::Value::String(phone.as_str().to_string())
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                DataType::Date => {
                    serde_json::Value::String(element.text().collect::<String>().trim().to_string())
                }
                DataType::Number => {
                    let text = element.text().collect::<String>();
                    if let Ok(number) = text.trim().parse::<f64>() {
                        serde_json::Value::Number(serde_json::Number::from_f64(number).unwrap())
                    } else {
                        continue;
                    }
                }
                DataType::Json => {
                    let text = element.text().collect::<String>();
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        json
                    } else {
                        continue;
                    }
                }
            };

            if let Some(attribute) = &rule.attribute {
                if let Some(attr_value) = element.value().attr(attribute) {
                    results.push(serde_json::Value::String(attr_value.to_string()));
                }
            } else {
                results.push(value);
            }
        }

        if results.is_empty() {
            return Err(anyhow::anyhow!("No data extracted for rule: {}", rule.name));
        }

        if results.len() == 1 {
            Ok(results.into_iter().next().unwrap())
        } else {
            Ok(serde_json::Value::Array(results))
        }
    }

    async fn convert_to_intelligence_data(&self, target: &ScrapingTarget, extracted_data: &HashMap<String, serde_json::Value>) -> IntelligenceResult<()> {
        let content = serde_json::json!({
            "url": target.url,
            "extracted_data": extracted_data,
            "scraped_at": Utc::now(),
            "target_id": target.id
        });

        let intelligence_data = IntelligenceData {
            id: IntelligenceId::new(),
            content: content.to_string(),
            source: DataSource::WebScraping,
            classification: DataClassification::Public,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            confidence: 0.9,
            quality_score: 0.8,
        };

        // Here you would typically send this to a message queue or store it
        info!("Converted scraped data to intelligence data: {}", intelligence_data.id.0);
        Ok(())
    }

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

    async fn update_stats(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let mut stats = self.stats.write().await;
            let results = self.results.read().await;
            
            stats.total_requests = results.len() as u64;
            stats.successful_requests = results.iter().filter(|r| r.errors.is_empty()).count() as u64;
            stats.failed_requests = results.iter().filter(|r| !r.errors.is_empty()).count() as u64;
            stats.total_data_extracted = results.iter().map(|r| r.extracted_data.len() as u64).sum();
            
            if !results.is_empty() {
                stats.average_response_time_ms = results.iter()
                    .map(|r| r.response_time_ms as f64)
                    .sum::<f64>() / results.len() as f64;
            }
            
            stats.last_update = Utc::now();
            drop(stats);
            drop(results);
            
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
        Ok(())
    }

    pub async fn get_stats(&self) -> ScrapingStats {
        self.stats.read().await.clone()
    }

    pub async fn get_results(&self) -> Vec<ScrapingResult> {
        self.results.read().await.clone()
    }

    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            http_client: self.http_client.clone(),
            targets: Arc::clone(&self.targets),
            results: Arc::clone(&self.results),
            stats: Arc::clone(&self.stats),
            is_running: Arc::clone(&self.is_running),
            rate_limiter: Arc::clone(&self.rate_limiter),
        }
    }
}
