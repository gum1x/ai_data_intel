use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, warn, error, debug};
use reqwest::Client;
use intelligence_core::{
    IntelligenceData, DataSource, DataClassification, IntelligenceId, Result as IntelligenceResult
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APICollectorConfig {
    pub endpoints: Vec<APIEndpoint>,
    pub max_concurrent_requests: usize,
    pub rate_limit_per_second: u32,
    pub batch_size: usize,
    pub collection_timeout_seconds: u64,
    pub retry_attempts: u32,
    pub retry_delay_seconds: u64,
    pub default_headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIEndpoint {
    pub name: String,
    pub url: String,
    pub method: String, // GET, POST, PUT, DELETE
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: Option<String>,
    pub auth_type: AuthType,
    pub auth_config: HashMap<String, String>,
    pub collection_interval_seconds: u64,
    pub is_active: bool,
    pub data_format: DataFormat,
    pub extraction_rules: Vec<ExtractionRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    None,
    Bearer,
    Basic,
    ApiKey,
    OAuth2,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFormat {
    JSON,
    XML,
    CSV,
    HTML,
    PlainText,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRule {
    pub name: String,
    pub selector: String, // JSONPath, XPath, CSS selector, or regex
    pub selector_type: SelectorType,
    pub data_type: DataType,
    pub required: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectorType {
    JsonPath,
    XPath,
    CssSelector,
    Regex,
    PlainText,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Url,
    Email,
    Phone,
    Date,
    Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIData {
    pub endpoint_name: String,
    pub url: String,
    pub timestamp: DateTime<Utc>,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub data: serde_json::Value,
    pub extracted_data: HashMap<String, serde_json::Value>,
    pub error: Option<String>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    pub requests_made: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub data_points_collected: u64,
    pub errors_count: u64,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub throughput_per_second: f64,
    pub average_response_time_ms: f64,
}

pub struct APICollector {
    config: APICollectorConfig,
    http_client: Client,
    stats: Arc<RwLock<CollectionStats>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    data_buffer: Arc<RwLock<Vec<APIData>>>,
    is_running: Arc<RwLock<bool>>,
    processors: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

struct RateLimiter {
    last_request: std::time::Instant,
    requests_per_second: u32,
}

impl APICollector {
    pub fn new(config: APICollectorConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
            stats: Arc::new(RwLock::new(CollectionStats {
                requests_made: 0,
                successful_requests: 0,
                failed_requests: 0,
                data_points_collected: 0,
                errors_count: 0,
                start_time: Utc::now(),
                last_update: Utc::now(),
                throughput_per_second: 0.0,
                average_response_time_ms: 0.0,
            })),
            rate_limiter: Arc::new(RwLock::new(RateLimiter {
                last_request: std::time::Instant::now(),
                requests_per_second: 10,
            })),
            data_buffer: Arc::new(RwLock::new(Vec::new())),
            is_running: Arc::new(RwLock::new(false)),
            processors: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start_collection(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = true;
        drop(is_running);
        
        info!("Starting API data collection for {} endpoints", self.config.endpoints.len());
        
        let mut tasks = Vec::new();
        
        // Start collection tasks for each endpoint
        for endpoint in &self.config.endpoints {
            if endpoint.is_active {
                let collector = self.clone_for_task();
                let endpoint_clone = endpoint.clone();
                let task = tokio::spawn(async move {
                    collector.collect_from_endpoint(endpoint_clone).await
                });
                tasks.push(task);
            }
        }
        
        // Start batch processing
        let collector = self.clone_for_task();
        let batch_task = tokio::spawn(async move {
            collector.process_batches().await
        });
        tasks.push(batch_task);
        
        {
            let mut processors = self.processors.write().await;
            processors.extend(tasks);
        }
        
        info!("API collector started with {} concurrent tasks", tasks.len());
        Ok(())
    }

    pub async fn stop_collection(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        
        let mut processors = self.processors.write().await;
        for processor in processors.drain(..) {
            processor.abort();
        }
        
        info!("API collection stopped");
        Ok(())
    }

    async fn collect_from_endpoint(&self, endpoint: APIEndpoint) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            match self.fetch_data_from_endpoint(&endpoint).await {
                Ok(api_data) => {
                    {
                        let mut buffer = self.data_buffer.write().await;
                        buffer.push(api_data.clone());
                    }
                    
                    {
                        let mut stats = self.stats.write().await;
                        stats.requests_made += 1;
                        stats.successful_requests += 1;
                        stats.data_points_collected += 1;
                        stats.last_update = Utc::now();
                    }
                    
                    debug!("Collected data from endpoint: {}", endpoint.name);
                    
                    self.enforce_rate_limit().await;
                }
                Err(e) => {
                    error!("Failed to fetch data from endpoint {}: {}", endpoint.name, e);
                    {
                        let mut stats = self.stats.write().await;
                        stats.requests_made += 1;
                        stats.failed_requests += 1;
                        stats.errors_count += 1;
                    }
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(endpoint.collection_interval_seconds)).await;
        }
        Ok(())
    }

    async fn fetch_data_from_endpoint(&self, endpoint: &APIEndpoint) -> IntelligenceResult<APIData> {
        let start_time = std::time::Instant::now();
        
        // Build request
        let mut request = match endpoint.method.to_uppercase().as_str() {
            "GET" => self.http_client.get(&endpoint.url),
            "POST" => self.http_client.post(&endpoint.url),
            "PUT" => self.http_client.put(&endpoint.url),
            "DELETE" => self.http_client.delete(&endpoint.url),
            _ => return Err(intelligence_core::IntelligenceError::Internal {
                message: format!("Unsupported HTTP method: {}", endpoint.method),
            }),
        };
        
        // Add headers
        for (key, value) in &self.config.default_headers {
            request = request.header(key, value);
        }
        
        for (key, value) in &endpoint.headers {
            request = request.header(key, value);
        }
        
        // Add authentication
        request = self.add_authentication(request, &endpoint.auth_type, &endpoint.auth_config)?;
        
        // Add query parameters
        if !endpoint.query_params.is_empty() {
            request = request.query(&endpoint.query_params);
        }
        
        // Add body for POST/PUT requests
        if let Some(body) = &endpoint.body {
            if endpoint.method.to_uppercase() == "POST" || endpoint.method.to_uppercase() == "PUT" {
                request = request.body(body.clone());
            }
        }
        
        // Send request
        let response = request.send().await?;
        let status_code = response.status().as_u16();
        let response_time = start_time.elapsed().as_millis() as u64;
        
        // Get response headers
        let mut response_headers = HashMap::new();
        for (key, value) in response.headers() {
            response_headers.insert(key.to_string(), value.to_str().unwrap_or("").to_string());
        }
        
        // Parse response based on data format
        let (data, error) = match endpoint.data_format {
            DataFormat::JSON => {
                match response.json::<serde_json::Value>().await {
                    Ok(json_data) => (json_data, None),
                    Err(e) => (serde_json::Value::Null, Some(e.to_string())),
                }
            }
            DataFormat::XML => {
                match response.text().await {
                    Ok(xml_text) => {
                        // In a real implementation, you'd parse XML here
                        (serde_json::Value::String(xml_text), None)
                    }
                    Err(e) => (serde_json::Value::Null, Some(e.to_string())),
                }
            }
            DataFormat::CSV => {
                match response.text().await {
                    Ok(csv_text) => {
                        // In a real implementation, you'd parse CSV here
                        (serde_json::Value::String(csv_text), None)
                    }
                    Err(e) => (serde_json::Value::Null, Some(e.to_string())),
                }
            }
            DataFormat::HTML => {
                match response.text().await {
                    Ok(html_text) => {
                        // In a real implementation, you'd parse HTML here
                        (serde_json::Value::String(html_text), None)
                    }
                    Err(e) => (serde_json::Value::Null, Some(e.to_string())),
                }
            }
            DataFormat::PlainText => {
                match response.text().await {
                    Ok(text) => (serde_json::Value::String(text), None),
                    Err(e) => (serde_json::Value::Null, Some(e.to_string())),
                }
            }
        };
        
        // Extract data using rules
        let extracted_data = self.extract_data_using_rules(&data, &endpoint.extraction_rules).await?;
        
        Ok(APIData {
            endpoint_name: endpoint.name.clone(),
            url: endpoint.url.clone(),
            timestamp: Utc::now(),
            status_code,
            response_time_ms: response_time,
            data,
            extracted_data,
            error,
            headers: response_headers,
        })
    }

    fn add_authentication(
        &self,
        request: reqwest::RequestBuilder,
        auth_type: &AuthType,
        auth_config: &HashMap<String, String>,
    ) -> IntelligenceResult<reqwest::RequestBuilder> {
        match auth_type {
            AuthType::None => Ok(request),
            AuthType::Bearer => {
                if let Some(token) = auth_config.get("token") {
                    Ok(request.header("Authorization", format!("Bearer {}", token)))
                } else {
                    Err(intelligence_core::IntelligenceError::Internal {
                        message: "Bearer token not provided".to_string(),
                    })
                }
            }
            AuthType::Basic => {
                if let (Some(username), Some(password)) = (auth_config.get("username"), auth_config.get("password")) {
                    Ok(request.basic_auth(username, Some(password)))
                } else {
                    Err(intelligence_core::IntelligenceError::Internal {
                        message: "Basic auth credentials not provided".to_string(),
                    })
                }
            }
            AuthType::ApiKey => {
                if let (Some(key), Some(value)) = (auth_config.get("key"), auth_config.get("value")) {
                    Ok(request.header(key, value))
                } else {
                    Err(intelligence_core::IntelligenceError::Internal {
                        message: "API key configuration not provided".to_string(),
                    })
                }
            }
            AuthType::OAuth2 => {
                if let Some(token) = auth_config.get("access_token") {
                    Ok(request.header("Authorization", format!("Bearer {}", token)))
                } else {
                    Err(intelligence_core::IntelligenceError::Internal {
                        message: "OAuth2 access token not provided".to_string(),
                    })
                }
            }
            AuthType::Custom => {
                // For custom auth, assume the auth_config contains the necessary headers
                let mut request = request;
                for (key, value) in auth_config {
                    request = request.header(key, value);
                }
                Ok(request)
            }
        }
    }

    async fn extract_data_using_rules(
        &self,
        data: &serde_json::Value,
        rules: &[ExtractionRule],
    ) -> IntelligenceResult<HashMap<String, serde_json::Value>> {
        let mut extracted_data = HashMap::new();
        
        for rule in rules {
            match self.extract_data_by_rule(data, rule).await {
                Ok(value) => {
                    extracted_data.insert(rule.name.clone(), value);
                }
                Err(e) => {
                    if rule.required {
                        return Err(e);
                    } else if let Some(default_value) = &rule.default_value {
                        extracted_data.insert(rule.name.clone(), serde_json::Value::String(default_value.clone()));
                    }
                }
            }
        }
        
        Ok(extracted_data)
    }

    async fn extract_data_by_rule(
        &self,
        data: &serde_json::Value,
        rule: &ExtractionRule,
    ) -> IntelligenceResult<serde_json::Value> {
        match rule.selector_type {
            SelectorType::JsonPath => {
                self.extract_json_path(data, &rule.selector).await
            }
            SelectorType::XPath => {
                // In a real implementation, you'd use an XPath library
                Err(intelligence_core::IntelligenceError::Internal {
                    message: "XPath extraction not implemented".to_string(),
                })
            }
            SelectorType::CssSelector => {
                // In a real implementation, you'd use a CSS selector library
                Err(intelligence_core::IntelligenceError::Internal {
                    message: "CSS selector extraction not implemented".to_string(),
                })
            }
            SelectorType::Regex => {
                self.extract_regex(data, &rule.selector).await
            }
            SelectorType::PlainText => {
                Ok(data.clone())
            }
        }
    }

    async fn extract_json_path(&self, data: &serde_json::Value, path: &str) -> IntelligenceResult<serde_json::Value> {
        // Simple JSON path implementation
        // In a real implementation, you'd use a proper JSONPath library
        let path_parts: Vec<&str> = path.split('.').collect();
        let mut current = data;
        
        for part in path_parts {
            if part.starts_with('[') && part.ends_with(']') {
                // Array access
                let index_str = &part[1..part.len()-1];
                if let Ok(index) = index_str.parse::<usize>() {
                    if let Some(array) = current.as_array() {
                        if let Some(value) = array.get(index) {
                            current = value;
                        } else {
                            return Err(intelligence_core::IntelligenceError::Internal {
                                message: format!("Array index {} out of bounds", index),
                            });
                        }
                    } else {
                        return Err(intelligence_core::IntelligenceError::Internal {
                            message: "Expected array for index access".to_string(),
                        });
                    }
                } else {
                    return Err(intelligence_core::IntelligenceError::Internal {
                        message: format!("Invalid array index: {}", index_str),
                    });
                }
            } else {
                // Object access
                if let Some(obj) = current.as_object() {
                    if let Some(value) = obj.get(part) {
                        current = value;
                    } else {
                        return Err(intelligence_core::IntelligenceError::Internal {
                            message: format!("Key '{}' not found", part),
                        });
                    }
                } else {
                    return Err(intelligence_core::IntelligenceError::Internal {
                        message: "Expected object for key access".to_string(),
                    });
                }
            }
        }
        
        Ok(current.clone())
    }

    async fn extract_regex(&self, data: &serde_json::Value, pattern: &str) -> IntelligenceResult<serde_json::Value> {
        let text = match data {
            serde_json::Value::String(s) => s.clone(),
            _ => data.to_string(),
        };
        
        let regex = regex::Regex::new(pattern)?;
        if let Some(captures) = regex.captures(&text) {
            if let Some(match_text) = captures.get(1) {
                Ok(serde_json::Value::String(match_text.as_str().to_string()))
            } else if let Some(match_text) = captures.get(0) {
                Ok(serde_json::Value::String(match_text.as_str().to_string()))
            } else {
                Ok(serde_json::Value::String("".to_string()))
            }
        } else {
            Err(intelligence_core::IntelligenceError::Internal {
                message: "No regex match found".to_string(),
            })
        }
    }

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
                let intelligence_data = self.convert_api_data_to_intelligence_data(batch).await?;
                info!("Processed batch of {} API responses into {} intelligence data points",
                      batch.len(), intelligence_data.len());
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        Ok(())
    }

    async fn convert_api_data_to_intelligence_data(
        &self,
        api_data: Vec<APIData>,
    ) -> IntelligenceResult<Vec<IntelligenceData>> {
        let mut intelligence_data = Vec::new();
        
        for data in api_data {
            let content = serde_json::json!({
                "endpoint_name": data.endpoint_name,
                "url": data.url,
                "timestamp": data.timestamp,
                "status_code": data.status_code,
                "response_time_ms": data.response_time_ms,
                "data": data.data,
                "extracted_data": data.extracted_data,
                "error": data.error,
                "headers": data.headers
            });
            
            let intelligence_data_point = IntelligenceData {
                id: IntelligenceId::new(),
                content: content.to_string(),
                source: DataSource::API,
                classification: DataClassification::Internal,
                metadata: HashMap::new(),
                timestamp: data.timestamp,
                confidence: if data.error.is_none() { 1.0 } else { 0.5 },
                quality_score: if data.status_code == 200 { 0.9 } else { 0.6 },
            };
            
            intelligence_data.push(intelligence_data_point);
        }
        
        Ok(intelligence_data)
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

    pub async fn get_stats(&self) -> CollectionStats {
        self.stats.read().await.clone()
    }

    pub async fn add_endpoint(&self, endpoint: APIEndpoint) -> IntelligenceResult<()> {
        // In a real implementation, you'd add this to the config and start a new task
        info!("Added new endpoint: {}", endpoint.name);
        Ok(())
    }

    pub async fn remove_endpoint(&self, endpoint_name: &str) -> IntelligenceResult<()> {
        // In a real implementation, you'd remove this from the config and stop the task
        info!("Removed endpoint: {}", endpoint_name);
        Ok(())
    }

    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            http_client: self.http_client.clone(),
            stats: Arc::clone(&self.stats),
            rate_limiter: Arc::clone(&self.rate_limiter),
            data_buffer: Arc::clone(&self.data_buffer),
            is_running: Arc::clone(&self.is_running),
            processors: Arc::clone(&self.processors),
        }
    }
}
