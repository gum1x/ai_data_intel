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
pub struct SocialMediaConfig {
    pub twitter_bearer_token: String,
    pub reddit_client_id: String,
    pub reddit_client_secret: String,
    pub reddit_user_agent: String,
    pub instagram_access_token: String,
    pub facebook_access_token: String,
    pub linkedin_access_token: String,
    pub max_concurrent_requests: usize,
    pub rate_limit_per_second: u32,
    pub batch_size: usize,
    pub collection_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMediaPost {
    pub id: String,
    pub platform: String,
    pub author_id: String,
    pub author_username: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub likes: u64,
    pub shares: u64,
    pub comments: u64,
    pub hashtags: Vec<String>,
    pub mentions: Vec<String>,
    pub urls: Vec<String>,
    pub media_urls: Vec<String>,
    pub location: Option<String>,
    pub language: Option<String>,
    pub sentiment: Option<String>,
    pub engagement_rate: f64,
    pub reach: Option<u64>,
    pub impressions: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMediaUser {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub followers_count: u64,
    pub following_count: u64,
    pub posts_count: u64,
    pub verified: bool,
    pub location: Option<String>,
    pub website: Option<String>,
    pub join_date: Option<DateTime<Utc>>,
    pub last_active: Option<DateTime<Utc>>,
    pub platform: String,
    pub profile_image_url: Option<String>,
    pub cover_image_url: Option<String>,
    pub is_private: bool,
    pub is_business: bool,
    pub engagement_rate: f64,
    pub avg_likes_per_post: f64,
    pub avg_comments_per_post: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    pub posts_collected: u64,
    pub users_analyzed: u64,
    pub platforms_monitored: u64,
    pub errors_count: u64,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub throughput_per_second: f64,
}

pub struct SocialMediaCollector {
    config: SocialMediaConfig,
    http_client: Client,
    stats: Arc<RwLock<CollectionStats>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    post_buffer: Arc<RwLock<Vec<SocialMediaPost>>>,
    user_cache: Arc<RwLock<HashMap<String, SocialMediaUser>>>,
    is_running: Arc<RwLock<bool>>,
    processors: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

struct RateLimiter {
    last_request: std::time::Instant,
    requests_per_second: u32,
}

impl SocialMediaCollector {
    pub fn new(config: SocialMediaConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
            stats: Arc::new(RwLock::new(CollectionStats {
                posts_collected: 0,
                users_analyzed: 0,
                platforms_monitored: 0,
                errors_count: 0,
                start_time: Utc::now(),
                last_update: Utc::now(),
                throughput_per_second: 0.0,
            })),
            rate_limiter: Arc::new(RwLock::new(RateLimiter {
                last_request: std::time::Instant::now(),
                requests_per_second: 5, // Conservative rate limit
            })),
            post_buffer: Arc::new(RwLock::new(Vec::new())),
            user_cache: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
            processors: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start_collection(&self, target_keywords: Vec<String>, target_users: Vec<String>) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = true;
        drop(is_running);
        
        info!("Starting social media collection for {} keywords and {} users", 
              target_keywords.len(), target_users.len());
        
        let mut tasks = Vec::new();
        
        // Start Twitter monitoring
        if !self.config.twitter_bearer_token.is_empty() {
            for keyword in target_keywords.clone() {
                let collector = self.clone_for_task();
                let task = tokio::spawn(async move {
                    collector.monitor_twitter_keyword(&keyword).await
                });
                tasks.push(task);
            }
            
            for user in target_users.clone() {
                let collector = self.clone_for_task();
                let task = tokio::spawn(async move {
                    collector.monitor_twitter_user(&user).await
                });
                tasks.push(task);
            }
        }
        
        // Start Reddit monitoring
        if !self.config.reddit_client_id.is_empty() {
            for keyword in target_keywords.clone() {
                let collector = self.clone_for_task();
                let task = tokio::spawn(async move {
                    collector.monitor_reddit_keyword(&keyword).await
                });
                tasks.push(task);
            }
        }
        
        // Start user analysis
        let collector = self.clone_for_task();
        let user_task = tokio::spawn(async move {
            collector.analyze_users().await
        });
        tasks.push(user_task);
        
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
        
        info!("Social media collector started with {} concurrent tasks", tasks.len());
        Ok(())
    }

    pub async fn stop_collection(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        
        let mut processors = self.processors.write().await;
        for processor in processors.drain(..) {
            processor.abort();
        }
        
        info!("Social media collection stopped");
        Ok(())
    }

    async fn monitor_twitter_keyword(&self, keyword: &str) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            match self.fetch_twitter_posts_by_keyword(keyword).await {
                Ok(posts) => {
                    if !posts.is_empty() {
                        {
                            let mut buffer = self.post_buffer.write().await;
                            buffer.extend(posts.clone());
                        }
                        
                        {
                            let mut stats = self.stats.write().await;
                            stats.posts_collected += posts.len() as u64;
                            stats.last_update = Utc::now();
                        }
                        
                        debug!("Collected {} Twitter posts for keyword: {}", posts.len(), keyword);
                    }
                    
                    self.enforce_rate_limit().await;
                }
                Err(e) => {
                    error!("Failed to fetch Twitter posts for keyword {}: {}", keyword, e);
                    {
                        let mut stats = self.stats.write().await;
                        stats.errors_count += 1;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
        Ok(())
    }

    async fn monitor_twitter_user(&self, username: &str) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            match self.fetch_twitter_user_posts(username).await {
                Ok(posts) => {
                    if !posts.is_empty() {
                        {
                            let mut buffer = self.post_buffer.write().await;
                            buffer.extend(posts.clone());
                        }
                        
                        {
                            let mut stats = self.stats.write().await;
                            stats.posts_collected += posts.len() as u64;
                            stats.last_update = Utc::now();
                        }
                        
                        debug!("Collected {} Twitter posts for user: {}", posts.len(), username);
                    }
                    
                    self.enforce_rate_limit().await;
                }
                Err(e) => {
                    error!("Failed to fetch Twitter posts for user {}: {}", username, e);
                    {
                        let mut stats = self.stats.write().await;
                        stats.errors_count += 1;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
        Ok(())
    }

    async fn fetch_twitter_posts_by_keyword(&self, keyword: &str) -> IntelligenceResult<Vec<SocialMediaPost>> {
        let url = "https://api.twitter.com/2/tweets/search/recent";
        let query = format!("{} -is:retweet", keyword);
        
        let params = [
            ("query", query.as_str()),
            ("max_results", "100"),
            ("tweet.fields", "created_at,public_metrics,author_id,context_annotations,entities"),
            ("user.fields", "username,name,verified,public_metrics"),
            ("expansions", "author_id"),
        ];
        
        let response = self.http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.config.twitter_bearer_token))
            .query(&params)
            .send()
            .await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            self.parse_twitter_response(data).await
        } else {
            Err(intelligence_core::IntelligenceError::ExternalService {
                service: "twitter".to_string(),
                message: format!("HTTP error: {}", response.status()),
            })
        }
    }

    async fn fetch_twitter_user_posts(&self, username: &str) -> IntelligenceResult<Vec<SocialMediaPost>> {
        // First get user ID
        let user_id = self.get_twitter_user_id(username).await?;
        
        let url = format!("https://api.twitter.com/2/users/{}/tweets", user_id);
        let params = [
            ("max_results", "100"),
            ("tweet.fields", "created_at,public_metrics,context_annotations,entities"),
            ("exclude", "retweets,replies"),
        ];
        
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.twitter_bearer_token))
            .query(&params)
            .send()
            .await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            self.parse_twitter_response(data).await
        } else {
            Err(intelligence_core::IntelligenceError::ExternalService {
                service: "twitter".to_string(),
                message: format!("HTTP error: {}", response.status()),
            })
        }
    }

    async fn get_twitter_user_id(&self, username: &str) -> IntelligenceResult<String> {
        let url = "https://api.twitter.com/2/users/by/username/".to_string() + username;
        let params = [("user.fields", "id,username,name")];
        
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.twitter_bearer_token))
            .query(&params)
            .send()
            .await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(user_data) = data.get("data") {
                if let Some(id) = user_data.get("id").and_then(|i| i.as_str()) {
                    Ok(id.to_string())
                } else {
                    Err(intelligence_core::IntelligenceError::ExternalService {
                        service: "twitter".to_string(),
                        message: "No user ID found".to_string(),
                    })
                }
            } else {
                Err(intelligence_core::IntelligenceError::ExternalService {
                    service: "twitter".to_string(),
                    message: "No user data found".to_string(),
                })
            }
        } else {
            Err(intelligence_core::IntelligenceError::ExternalService {
                service: "twitter".to_string(),
                message: format!("HTTP error: {}", response.status()),
            })
        }
    }

    async fn parse_twitter_response(&self, data: serde_json::Value) -> IntelligenceResult<Vec<SocialMediaPost>> {
        let mut posts = Vec::new();
        
        if let Some(tweets) = data.get("data").and_then(|d| d.as_array()) {
            let users = data.get("includes")
                .and_then(|i| i.get("users"))
                .and_then(|u| u.as_array())
                .unwrap_or(&Vec::new());
            
            let mut user_map = HashMap::new();
            for user in users {
                if let (Some(id), Some(username), Some(name)) = (
                    user.get("id").and_then(|i| i.as_str()),
                    user.get("username").and_then(|u| u.as_str()),
                    user.get("name").and_then(|n| n.as_str()),
                ) {
                    user_map.insert(id, (username, name));
                }
            }
            
            for tweet in tweets {
                if let Ok(post) = self.parse_twitter_tweet(tweet, &user_map).await {
                    posts.push(post);
                }
            }
        }
        
        Ok(posts)
    }

    async fn parse_twitter_tweet(&self, tweet: &serde_json::Value, user_map: &HashMap<&str, (&str, &str)>) -> IntelligenceResult<SocialMediaPost> {
        let id = tweet.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();
        let author_id = tweet.get("author_id").and_then(|a| a.as_str()).unwrap_or("").to_string();
        let content = tweet.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string();
        
        let (author_username, _) = user_map.get(author_id.as_str()).unwrap_or(&("unknown", "Unknown"));
        
        let timestamp = if let Some(created_at) = tweet.get("created_at").and_then(|c| c.as_str()) {
            DateTime::parse_from_rfc3339(created_at).unwrap_or_else(|_| Utc::now()).with_timezone(&Utc)
        } else {
            Utc::now()
        };
        
        let public_metrics = tweet.get("public_metrics").unwrap_or(&serde_json::Value::Null);
        let likes = public_metrics.get("like_count").and_then(|l| l.as_u64()).unwrap_or(0);
        let shares = public_metrics.get("retweet_count").and_then(|r| r.as_u64()).unwrap_or(0);
        let comments = public_metrics.get("reply_count").and_then(|c| c.as_u64()).unwrap_or(0);
        
        let entities = tweet.get("entities").unwrap_or(&serde_json::Value::Null);
        let hashtags = self.extract_hashtags(entities);
        let mentions = self.extract_mentions(entities);
        let urls = self.extract_urls(entities);
        
        let engagement_rate = self.calculate_engagement_rate(likes, shares, comments);
        
        Ok(SocialMediaPost {
            id,
            platform: "twitter".to_string(),
            author_id,
            author_username: author_username.to_string(),
            content,
            timestamp,
            likes,
            shares,
            comments,
            hashtags,
            mentions,
            urls,
            media_urls: Vec::new(), // Would need to parse media entities
            location: None,
            language: None,
            sentiment: None,
            engagement_rate,
            reach: None,
            impressions: None,
        })
    }

    fn extract_hashtags(&self, entities: &serde_json::Value) -> Vec<String> {
        let mut hashtags = Vec::new();
        if let Some(hashtag_array) = entities.get("hashtags").and_then(|h| h.as_array()) {
            for hashtag in hashtag_array {
                if let Some(tag) = hashtag.get("tag").and_then(|t| t.as_str()) {
                    hashtags.push(tag.to_string());
                }
            }
        }
        hashtags
    }

    fn extract_mentions(&self, entities: &serde_json::Value) -> Vec<String> {
        let mut mentions = Vec::new();
        if let Some(mention_array) = entities.get("mentions").and_then(|m| m.as_array()) {
            for mention in mention_array {
                if let Some(username) = mention.get("username").and_then(|u| u.as_str()) {
                    mentions.push(username.to_string());
                }
            }
        }
        mentions
    }

    fn extract_urls(&self, entities: &serde_json::Value) -> Vec<String> {
        let mut urls = Vec::new();
        if let Some(url_array) = entities.get("urls").and_then(|u| u.as_array()) {
            for url in url_array {
                if let Some(expanded_url) = url.get("expanded_url").and_then(|u| u.as_str()) {
                    urls.push(expanded_url.to_string());
                }
            }
        }
        urls
    }

    fn calculate_engagement_rate(&self, likes: u64, shares: u64, comments: u64) -> f64 {
        let total_engagement = likes + shares + comments;
        // This is a simplified calculation - in reality you'd need follower count
        total_engagement as f64 / 1000.0 // Assuming 1000 followers as baseline
    }

    async fn monitor_reddit_keyword(&self, keyword: &str) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            match self.fetch_reddit_posts_by_keyword(keyword).await {
                Ok(posts) => {
                    if !posts.is_empty() {
                        {
                            let mut buffer = self.post_buffer.write().await;
                            buffer.extend(posts.clone());
                        }
                        
                        {
                            let mut stats = self.stats.write().await;
                            stats.posts_collected += posts.len() as u64;
                            stats.last_update = Utc::now();
                        }
                        
                        debug!("Collected {} Reddit posts for keyword: {}", posts.len(), keyword);
                    }
                    
                    self.enforce_rate_limit().await;
                }
                Err(e) => {
                    error!("Failed to fetch Reddit posts for keyword {}: {}", keyword, e);
                    {
                        let mut stats = self.stats.write().await;
                        stats.errors_count += 1;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
        Ok(())
    }

    async fn fetch_reddit_posts_by_keyword(&self, keyword: &str) -> IntelligenceResult<Vec<SocialMediaPost>> {
        // First get access token
        let access_token = self.get_reddit_access_token().await?;
        
        let url = "https://oauth.reddit.com/search";
        let params = [
            ("q", keyword),
            ("sort", "new"),
            ("limit", "100"),
            ("raw_json", "1"),
        ];
        
        let response = self.http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", &self.config.reddit_user_agent)
            .query(&params)
            .send()
            .await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            self.parse_reddit_response(data).await
        } else {
            Err(intelligence_core::IntelligenceError::ExternalService {
                service: "reddit".to_string(),
                message: format!("HTTP error: {}", response.status()),
            })
        }
    }

    async fn get_reddit_access_token(&self) -> IntelligenceResult<String> {
        let url = "https://www.reddit.com/api/v1/access_token";
        let params = [
            ("grant_type", "client_credentials"),
        ];
        
        let response = self.http_client
            .post(url)
            .basic_auth(&self.config.reddit_client_id, Some(&self.config.reddit_client_secret))
            .header("User-Agent", &self.config.reddit_user_agent)
            .form(&params)
            .send()
            .await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(access_token) = data.get("access_token").and_then(|t| t.as_str()) {
                Ok(access_token.to_string())
            } else {
                Err(intelligence_core::IntelligenceError::ExternalService {
                    service: "reddit".to_string(),
                    message: "No access token in response".to_string(),
                })
            }
        } else {
            Err(intelligence_core::IntelligenceError::ExternalService {
                service: "reddit".to_string(),
                message: format!("HTTP error: {}", response.status()),
            })
        }
    }

    async fn parse_reddit_response(&self, data: serde_json::Value) -> IntelligenceResult<Vec<SocialMediaPost>> {
        let mut posts = Vec::new();
        
        if let Some(children) = data.get("data")
            .and_then(|d| d.get("children"))
            .and_then(|c| c.as_array()) {
            
            for child in children {
                if let Some(post_data) = child.get("data") {
                    if let Ok(post) = self.parse_reddit_post(post_data).await {
                        posts.push(post);
                    }
                }
            }
        }
        
        Ok(posts)
    }

    async fn parse_reddit_post(&self, post_data: &serde_json::Value) -> IntelligenceResult<SocialMediaPost> {
        let id = post_data.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();
        let author_username = post_data.get("author").and_then(|a| a.as_str()).unwrap_or("").to_string();
        let content = post_data.get("selftext").and_then(|s| s.as_str()).unwrap_or("").to_string();
        
        let timestamp = if let Some(created_utc) = post_data.get("created_utc").and_then(|c| c.as_f64()) {
            DateTime::from_timestamp(created_utc as i64, 0).unwrap_or_else(Utc::now)
        } else {
            Utc::now()
        };
        
        let likes = post_data.get("ups").and_then(|u| u.as_u64()).unwrap_or(0);
        let shares = post_data.get("num_crossposts").and_then(|c| c.as_u64()).unwrap_or(0);
        let comments = post_data.get("num_comments").and_then(|c| c.as_u64()).unwrap_or(0);
        
        let subreddit = post_data.get("subreddit").and_then(|s| s.as_str()).unwrap_or("");
        let title = post_data.get("title").and_then(|t| t.as_str()).unwrap_or("");
        let full_content = if content.is_empty() { title.to_string() } else { content };
        
        let engagement_rate = self.calculate_engagement_rate(likes, shares, comments);
        
        Ok(SocialMediaPost {
            id,
            platform: "reddit".to_string(),
            author_id: author_username.clone(),
            author_username,
            content: full_content,
            timestamp,
            likes,
            shares,
            comments,
            hashtags: Vec::new(), // Reddit doesn't use hashtags
            mentions: Vec::new(), // Would need to parse u/ mentions
            urls: Vec::new(), // Would need to parse URLs
            media_urls: Vec::new(),
            location: None,
            language: None,
            sentiment: None,
            engagement_rate,
            reach: None,
            impressions: None,
        })
    }

    async fn analyze_users(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let user_ids = {
                let buffer = self.post_buffer.read().await;
                let mut user_ids = std::collections::HashSet::new();
                for post in buffer.iter() {
                    user_ids.insert((post.author_id.clone(), post.platform.clone()));
                }
                user_ids.into_iter().collect::<Vec<_>>()
            };
            
            for (user_id, platform) in user_ids {
                let cache_key = format!("{}:{}", platform, user_id);
                if self.user_cache.read().await.contains_key(&cache_key) {
                    continue;
                }
                
                match self.analyze_user(&user_id, &platform).await {
                    Ok(user_info) => {
                        self.user_cache.write().await.insert(cache_key, user_info);
                        {
                            let mut stats = self.stats.write().await;
                            stats.users_analyzed += 1;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to analyze user {} on {}: {}", user_id, platform, e);
                    }
                }
                
                self.enforce_rate_limit().await;
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
        Ok(())
    }

    async fn analyze_user(&self, user_id: &str, platform: &str) -> IntelligenceResult<SocialMediaUser> {
        match platform {
            "twitter" => self.analyze_twitter_user(user_id).await,
            "reddit" => self.analyze_reddit_user(user_id).await,
            _ => Err(intelligence_core::IntelligenceError::Internal {
                message: format!("Unsupported platform: {}", platform),
            }),
        }
    }

    async fn analyze_twitter_user(&self, user_id: &str) -> IntelligenceResult<SocialMediaUser> {
        let url = format!("https://api.twitter.com/2/users/{}", user_id);
        let params = [
            ("user.fields", "created_at,description,entities,id,location,name,pinned_tweet_id,profile_image_url,protected,public_metrics,url,username,verified,verified_type"),
        ];
        
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.twitter_bearer_token))
            .query(&params)
            .send()
            .await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(user_data) = data.get("data") {
                self.parse_twitter_user(user_data).await
            } else {
                Err(intelligence_core::IntelligenceError::ExternalService {
                    service: "twitter".to_string(),
                    message: "No user data found".to_string(),
                })
            }
        } else {
            Err(intelligence_core::IntelligenceError::ExternalService {
                service: "twitter".to_string(),
                message: format!("HTTP error: {}", response.status()),
            })
        }
    }

    async fn parse_twitter_user(&self, user_data: &serde_json::Value) -> IntelligenceResult<SocialMediaUser> {
        let id = user_data.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();
        let username = user_data.get("username").and_then(|u| u.as_str()).unwrap_or("").to_string();
        let display_name = user_data.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
        let bio = user_data.get("description").and_then(|d| d.as_str()).map(|s| s.to_string());
        let location = user_data.get("location").and_then(|l| l.as_str()).map(|s| s.to_string());
        let website = user_data.get("url").and_then(|u| u.as_str()).map(|s| s.to_string());
        let verified = user_data.get("verified").and_then(|v| v.as_bool()).unwrap_or(false);
        let is_private = user_data.get("protected").and_then(|p| p.as_bool()).unwrap_or(false);
        let profile_image_url = user_data.get("profile_image_url").and_then(|p| p.as_str()).map(|s| s.to_string());
        
        let public_metrics = user_data.get("public_metrics").unwrap_or(&serde_json::Value::Null);
        let followers_count = public_metrics.get("followers_count").and_then(|f| f.as_u64()).unwrap_or(0);
        let following_count = public_metrics.get("following_count").and_then(|f| f.as_u64()).unwrap_or(0);
        let posts_count = public_metrics.get("tweet_count").and_then(|t| t.as_u64()).unwrap_or(0);
        
        let join_date = if let Some(created_at) = user_data.get("created_at").and_then(|c| c.as_str()) {
            DateTime::parse_from_rfc3339(created_at).unwrap_or_else(|_| Utc::now()).with_timezone(&Utc).into()
        } else {
            None
        };
        
        let engagement_rate = self.calculate_user_engagement_rate(followers_count, posts_count);
        
        Ok(SocialMediaUser {
            id,
            username,
            display_name,
            bio,
            followers_count,
            following_count,
            posts_count,
            verified,
            location,
            website,
            join_date,
            last_active: None,
            platform: "twitter".to_string(),
            profile_image_url,
            cover_image_url: None,
            is_private,
            is_business: false, // Would need to check if it's a business account
            engagement_rate,
            avg_likes_per_post: 0.0, // Would need to calculate from posts
            avg_comments_per_post: 0.0, // Would need to calculate from posts
        })
    }

    async fn analyze_reddit_user(&self, username: &str) -> IntelligenceResult<SocialMediaUser> {
        let access_token = self.get_reddit_access_token().await?;
        let url = format!("https://oauth.reddit.com/user/{}/about", username);
        
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", &self.config.reddit_user_agent)
            .send()
            .await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(user_data) = data.get("data") {
                self.parse_reddit_user(user_data).await
            } else {
                Err(intelligence_core::IntelligenceError::ExternalService {
                    service: "reddit".to_string(),
                    message: "No user data found".to_string(),
                })
            }
        } else {
            Err(intelligence_core::IntelligenceError::ExternalService {
                service: "reddit".to_string(),
                message: format!("HTTP error: {}", response.status()),
            })
        }
    }

    async fn parse_reddit_user(&self, user_data: &serde_json::Value) -> IntelligenceResult<SocialMediaUser> {
        let id = user_data.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();
        let username = user_data.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
        let display_name = username.clone();
        let bio = user_data.get("subreddit").and_then(|s| s.get("public_description")).and_then(|d| d.as_str()).map(|s| s.to_string());
        let verified = user_data.get("verified").and_then(|v| v.as_bool()).unwrap_or(false);
        let is_private = user_data.get("subreddit").and_then(|s| s.get("subreddit_type")).and_then(|t| t.as_str()) == Some("private");
        
        let followers_count = user_data.get("subreddit").and_then(|s| s.get("subscribers")).and_then(|s| s.as_u64()).unwrap_or(0);
        let posts_count = user_data.get("link_karma").and_then(|l| l.as_u64()).unwrap_or(0) + 
                         user_data.get("comment_karma").and_then(|c| c.as_u64()).unwrap_or(0);
        
        let join_date = if let Some(created_utc) = user_data.get("created_utc").and_then(|c| c.as_f64()) {
            DateTime::from_timestamp(created_utc as i64, 0).unwrap_or_else(Utc::now).into()
        } else {
            None
        };
        
        let engagement_rate = self.calculate_user_engagement_rate(followers_count, posts_count);
        
        Ok(SocialMediaUser {
            id,
            username,
            display_name,
            bio,
            followers_count,
            following_count: 0, // Reddit doesn't have following count
            posts_count,
            verified,
            location: None,
            website: None,
            join_date,
            last_active: None,
            platform: "reddit".to_string(),
            profile_image_url: None,
            cover_image_url: None,
            is_private,
            is_business: false,
            engagement_rate,
            avg_likes_per_post: 0.0,
            avg_comments_per_post: 0.0,
        })
    }

    fn calculate_user_engagement_rate(&self, followers: u64, posts: u64) -> f64 {
        if followers > 0 && posts > 0 {
            (posts as f64 / followers as f64) * 100.0
        } else {
            0.0
        }
    }

    async fn process_batches(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let batch = {
                let mut buffer = self.post_buffer.write().await;
                if buffer.len() >= self.config.batch_size {
                    let batch = buffer.drain(0..self.config.batch_size).collect::<Vec<_>>();
                    batch
                } else {
                    Vec::new()
                }
            };
            
            if !batch.is_empty() {
                let intelligence_data = self.convert_posts_to_intelligence_data(batch).await?;
                info!("Processed batch of {} posts into {} intelligence data points",
                      batch.len(), intelligence_data.len());
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        Ok(())
    }

    async fn convert_posts_to_intelligence_data(
        &self,
        posts: Vec<SocialMediaPost>,
    ) -> IntelligenceResult<Vec<IntelligenceData>> {
        let mut intelligence_data = Vec::new();
        
        for post in posts {
            let content = serde_json::json!({
                "id": post.id,
                "platform": post.platform,
                "author_id": post.author_id,
                "author_username": post.author_username,
                "content": post.content,
                "timestamp": post.timestamp,
                "likes": post.likes,
                "shares": post.shares,
                "comments": post.comments,
                "hashtags": post.hashtags,
                "mentions": post.mentions,
                "urls": post.urls,
                "engagement_rate": post.engagement_rate,
                "location": post.location,
                "language": post.language,
                "sentiment": post.sentiment
            });
            
            let data = IntelligenceData {
                id: IntelligenceId::new(),
                content: content.to_string(),
                source: DataSource::SocialMedia,
                classification: DataClassification::Internal,
                metadata: HashMap::new(),
                timestamp: post.timestamp,
                confidence: 1.0,
                quality_score: 0.9,
            };
            
            intelligence_data.push(data);
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

    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            http_client: self.http_client.clone(),
            stats: Arc::clone(&self.stats),
            rate_limiter: Arc::clone(&self.rate_limiter),
            post_buffer: Arc::clone(&self.post_buffer),
            user_cache: Arc::clone(&self.user_cache),
            is_running: Arc::clone(&self.is_running),
            processors: Arc::clone(&self.processors),
        }
    }
}
