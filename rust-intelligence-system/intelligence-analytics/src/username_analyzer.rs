use std::collections::{HashMap, HashSet};
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UsernamePattern {
    Sequential,
    Random,
    Branded,
    Professional,
    Cryptic,
    Suspicious,
    Bot,
    Personal,
    Creative,
    Anonymous,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernameAnalysis {
    pub username: String,
    pub platforms: Vec<Platform>,
    pub pattern_type: UsernamePattern,
    pub confidence: f64,
    pub risk_score: f64,
    pub similarity_matches: Vec<SimilarityMatch>,
    pub cross_platform_matches: Vec<CrossPlatformMatch>,
    pub username_evolution: Vec<UsernameChange>,
    pub associated_entities: Vec<String>,
    pub behavioral_indicators: Vec<BehavioralIndicator>,
    pub timestamp: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub name: String,
    pub url: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub activity_level: ActivityLevel,
    pub verification_status: VerificationStatus,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivityLevel {
    High,
    Medium,
    Low,
    Dormant,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationStatus {
    Verified,
    Unverified,
    Suspended,
    Banned,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityMatch {
    pub username: String,
    pub similarity_score: f64,
    pub platform: String,
    pub match_type: MatchType,
    pub confidence: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MatchType {
    Exact,
    Levenshtein,
    Phonetic,
    Pattern,
    Semantic,
    Typo,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPlatformMatch {
    pub username: String,
    pub platforms: Vec<String>,
    pub confidence: f64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub activity_correlation: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernameChange {
    pub old_username: String,
    pub new_username: String,
    pub platform: String,
    pub change_date: DateTime<Utc>,
    pub change_reason: Option<String>,
    pub similarity_score: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralIndicator {
    pub indicator_type: BehavioralType,
    pub description: String,
    pub confidence: f64,
    pub risk_level: RiskLevel,
    pub evidence: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BehavioralType {
    SuspiciousActivity,
    BotBehavior,
    ProfessionalUse,
    PersonalUse,
    CommercialUse,
    AnonymousBehavior,
    HighRiskActivity,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernameAnalyzer {
    username_database: Arc<RwLock<HashMap<String, UsernameAnalysis>>>,
    cross_platform_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    similarity_cache: Arc<RwLock<HashMap<String, Vec<SimilarityMatch>>>>,
    pattern_models: Arc<RwLock<HashMap<UsernamePattern, PatternModel>>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternModel {
    pub pattern_type: UsernamePattern,
    pub regex_patterns: Vec<String>,
    pub keywords: Vec<String>,
    pub characteristics: Vec<String>,
    pub risk_factors: Vec<RiskFactor>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor: String,
    pub weight: f64,
    pub description: String,
}
impl UsernameAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            username_database: Arc::new(RwLock::new(HashMap::new())),
            cross_platform_index: Arc::new(RwLock::new(HashMap::new())),
            similarity_cache: Arc::new(RwLock::new(HashMap::new())),
            pattern_models: Arc::new(RwLock::new(HashMap::new())),
        };
        analyzer.initialize_pattern_models();
        analyzer
    }
    pub async fn analyze_username(&self, username: &str, platform: &str) -> IntelligenceResult<UsernameAnalysis> {
        info!("Analyzing username: {} on platform: {}", username, platform);
        if let Some(existing_analysis) = self.username_database.read().await.get(username) {
            return self.update_analysis(existing_analysis.clone(), platform).await;
        }
        let pattern_type = self.classify_username_pattern(username).await;
        let risk_score = self.calculate_risk_score(username, &pattern_type).await;
        let similarity_matches = self.find_similar_usernames(username).await?;
        let cross_platform_matches = self.find_cross_platform_matches(username).await?;
        let behavioral_indicators = self.analyze_behavioral_indicators(username, platform).await?;
        let analysis = UsernameAnalysis {
            username: username.to_string(),
            platforms: vec![Platform {
                name: platform.to_string(),
                url: None,
                first_seen: Utc::now(),
                last_seen: Utc::now(),
                activity_level: ActivityLevel::Unknown,
                verification_status: VerificationStatus::Unknown,
            }],
            pattern_type,
            confidence: 0.9,
            risk_score,
            similarity_matches,
            cross_platform_matches,
            username_evolution: Vec::new(),
            associated_entities: Vec::new(),
            behavioral_indicators,
            timestamp: Utc::now(),
        };
        {
            let mut database = self.username_database.write().await;
            database.insert(username.to_string(), analysis.clone());
        }
        self.update_cross_platform_index(&analysis).await;
        Ok(analysis)
    }
    async fn classify_username_pattern(&self, username: &str) -> UsernamePattern {
        let username_lower = username.to_lowercase();
        if self.is_sequential_pattern(&username_lower) {
            return UsernamePattern::Sequential;
        }
        if self.is_random_pattern(&username_lower) {
            return UsernamePattern::Random;
        }
        if self.is_branded_pattern(&username_lower) {
            return UsernamePattern::Branded;
        }
        if self.is_professional_pattern(&username_lower) {
            return UsernamePattern::Professional;
        }
        if self.is_cryptic_pattern(&username_lower) {
            return UsernamePattern::Cryptic;
        }
        if self.is_suspicious_pattern(&username_lower) {
            return UsernamePattern::Suspicious;
        }
        if self.is_bot_pattern(&username_lower) {
            return UsernamePattern::Bot;
        }
        if self.is_personal_pattern(&username_lower) {
            return UsernamePattern::Personal;
        }
        if self.is_anonymous_pattern(&username_lower) {
            return UsernamePattern::Anonymous;
        }
        UsernamePattern::Creative
    }
    fn is_sequential_pattern(&self, username: &str) -> bool {
        let number_regex = regex::Regex::new(r"\d+$").unwrap();
        if number_regex.is_match(username) {
            let number_part = number_regex.find(username).unwrap().as_str();
            if let Ok(num) = number_part.parse::<u32>() {
                return num <= 1000;
            }
        }
        let letter_regex = regex::Regex::new(r"[a-z]+$").unwrap();
        if letter_regex.is_match(username) {
            let letter_part = letter_regex.find(username).unwrap().as_str();
            return letter_part.len() <= 3 && letter_part.chars().all(|c| c.is_ascii_lowercase());
        }
        false
    }
    fn is_random_pattern(&self, username: &str) -> bool {
        let has_numbers = username.chars().any(|c| c.is_ascii_digit());
        let has_letters = username.chars().any(|c| c.is_ascii_alphabetic());
        let has_special = username.chars().any(|c| !c.is_alphanumeric());
        has_numbers && has_letters && (has_special || username.len() > 8)
    }
    fn is_branded_pattern(&self, username: &str) -> bool {
        let brand_keywords = vec![
            "nike", "adidas", "apple", "google", "microsoft", "amazon", "tesla",
            "bitcoin", "ethereum", "crypto", "nft", "defi", "web3"
        ];
        brand_keywords.iter().any(|brand| username.contains(brand))
    }
    fn is_professional_pattern(&self, username: &str) -> bool {
        let parts: Vec<&str> = username.split('_').collect();
        if parts.len() == 2 {
            let first = parts[0];
            let last = parts[1];
            return first.len() >= 2 && last.len() >= 2 &&
                   first.chars().all(|c| c.is_ascii_alphabetic()) &&
                   last.chars().all(|c| c.is_ascii_alphabetic());
        }
        let professional_keywords = vec![
            "ceo", "cto", "cfo", "manager", "director", "executive",
            "consultant", "advisor", "expert", "specialist"
        ];
        professional_keywords.iter().any(|keyword| username.contains(keyword))
    }
    fn is_cryptic_pattern(&self, username: &str) -> bool {
        if username.starts_with("0x") && username.len() > 10 {
            return true;
        }
        if username.starts_with('_') || username.ends_with('_') {
            return true;
        }
        let has_upper = username.chars().any(|c| c.is_ascii_uppercase());
        let has_lower = username.chars().any(|c| c.is_ascii_lowercase());
        let has_numbers = username.chars().any(|c| c.is_ascii_digit());
        has_upper && has_lower && has_numbers && username.len() > 6
    }
    fn is_suspicious_pattern(&self, username: &str) -> bool {
        let suspicious_keywords = vec![
            "admin", "root", "support", "system", "service", "api",
            "test", "demo", "sample", "example", "default", "guest"
        ];
        suspicious_keywords.iter().any(|keyword| username.contains(keyword))
    }
    fn is_bot_pattern(&self, username: &str) -> bool {
        let bot_keywords = vec![
            "bot", "automated", "auto", "script", "crawler", "scraper",
            "monitor", "alert", "notification", "service"
        ];
        bot_keywords.iter().any(|keyword| username.contains(keyword))
    }
    fn is_personal_pattern(&self, username: &str) -> bool {
        let common_names = vec![
            "john", "jane", "mike", "sarah", "david", "lisa", "chris", "emma",
            "alex", "sam", "joe", "amy", "tom", "kate", "dan", "anna"
        ];
        common_names.iter().any(|name| username.contains(name))
    }
    fn is_anonymous_pattern(&self, username: &str) -> bool {
        let anonymous_keywords = vec![
            "anonymous", "anon", "unknown", "hidden", "secret", "private",
            "incognito", "ghost", "shadow", "mystery"
        ];
        anonymous_keywords.iter().any(|keyword| username.contains(keyword))
    }
    async fn calculate_risk_score(&self, username: &str, pattern_type: &UsernamePattern) -> f64 {
        let mut risk_score = 0.0;
        match pattern_type {
            UsernamePattern::Suspicious => risk_score += 0.8,
            UsernamePattern::Bot => risk_score += 0.7,
            UsernamePattern::Cryptic => risk_score += 0.6,
            UsernamePattern::Anonymous => risk_score += 0.5,
            UsernamePattern::Random => risk_score += 0.4,
            UsernamePattern::Sequential => risk_score += 0.3,
            UsernamePattern::Branded => risk_score += 0.2,
            UsernamePattern::Professional => risk_score += 0.1,
            UsernamePattern::Personal => risk_score += 0.1,
            UsernamePattern::Creative => risk_score += 0.2,
        }
        if username.len() < 3 {
            risk_score += 0.3;
        }
        if username.len() > 20 {
            risk_score += 0.2;
        }
        if username.chars().all(|c| c.is_ascii_digit()) {
            risk_score += 0.4;
        }
        if username.chars().all(|c| c.is_ascii_uppercase()) {
            risk_score += 0.3;
        }
        risk_score.min(1.0)
    }
    async fn find_similar_usernames(&self, username: &str) -> IntelligenceResult<Vec<SimilarityMatch>> {
        let mut matches = Vec::new();
        if let Some(cached_matches) = self.similarity_cache.read().await.get(username) {
            return Ok(cached_matches.clone());
        }
        let database = self.username_database.read().await;
        for (other_username, _) in database.iter() {
            if other_username == username {
                continue;
            }
            let similarity_score = self.calculate_similarity(username, other_username);
            if similarity_score > 0.7 {
                let match_type = self.determine_match_type(username, other_username);
                matches.push(SimilarityMatch {
                    username: other_username.clone(),
                    similarity_score,
                    platform: "unknown".to_string(),
                    match_type,
                    confidence: similarity_score,
                });
            }
        }
        matches.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        {
            let mut cache = self.similarity_cache.write().await;
            cache.insert(username.to_string(), matches.clone());
        }
        Ok(matches)
    }
    fn calculate_similarity(&self, username1: &str, username2: &str) -> f64 {
        let distance = self.levenshtein_distance(username1, username2);
        let max_len = username1.len().max(username2.len()) as f64;
        if max_len == 0.0 {
            return 1.0;
        }
        1.0 - (distance as f64 / max_len)
    }
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        let mut matrix = vec![vec![0; s2_chars.len() + 1]; s1_chars.len() + 1];
        for i in 0..=s1_chars.len() {
            matrix[i][0] = i;
        }
        for j in 0..=s2_chars.len() {
            matrix[0][j] = j;
        }
        for i in 1..=s1_chars.len() {
            for j in 1..=s2_chars.len() {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }
        matrix[s1_chars.len()][s2_chars.len()]
    }
    fn determine_match_type(&self, username1: &str, username2: &str) -> MatchType {
        if username1 == username2 {
            return MatchType::Exact;
        }
        let similarity = self.calculate_similarity(username1, username2);
        if similarity > 0.95 {
            MatchType::Typo
        } else if similarity > 0.8 {
            MatchType::Levenshtein
        } else if similarity > 0.6 {
            MatchType::Pattern
        } else {
            MatchType::Semantic
        }
    }
    async fn find_cross_platform_matches(&self, username: &str) -> IntelligenceResult<Vec<CrossPlatformMatch>> {
        let mut matches = Vec::new();
        if let Some(platforms) = self.cross_platform_index.read().await.get(username) {
            if platforms.len() > 1 {
                matches.push(CrossPlatformMatch {
                    username: username.to_string(),
                    platforms: platforms.clone(),
                    confidence: 0.9,
                    first_seen: Utc::now(),
                    last_seen: Utc::now(),
                    activity_correlation: 0.8,
                });
            }
        }
        Ok(matches)
    }
    async fn analyze_behavioral_indicators(&self, username: &str, platform: &str) -> IntelligenceResult<Vec<BehavioralIndicator>> {
        let mut indicators = Vec::new();
        if self.is_suspicious_pattern(&username.to_lowercase()) {
            indicators.push(BehavioralIndicator {
                indicator_type: BehavioralType::SuspiciousActivity,
                description: "Username contains suspicious keywords".to_string(),
                confidence: 0.8,
                risk_level: RiskLevel::High,
                evidence: vec![username.to_string()],
            });
        }
        if self.is_bot_pattern(&username.to_lowercase()) {
            indicators.push(BehavioralIndicator {
                indicator_type: BehavioralType::BotBehavior,
                description: "Username suggests automated behavior".to_string(),
                confidence: 0.7,
                risk_level: RiskLevel::Medium,
                evidence: vec![username.to_string()],
            });
        }
        if self.is_professional_pattern(&username.to_lowercase()) {
            indicators.push(BehavioralIndicator {
                indicator_type: BehavioralType::ProfessionalUse,
                description: "Username suggests professional use".to_string(),
                confidence: 0.6,
                risk_level: RiskLevel::Low,
                evidence: vec![username.to_string()],
            });
        }
        Ok(indicators)
    }
    async fn update_analysis(&self, mut analysis: UsernameAnalysis, platform: &str) -> IntelligenceResult<UsernameAnalysis> {
        let platform_exists = analysis.platforms.iter().any(|p| p.name == platform);
        if !platform_exists {
            analysis.platforms.push(Platform {
                name: platform.to_string(),
                url: None,
                first_seen: Utc::now(),
                last_seen: Utc::now(),
                activity_level: ActivityLevel::Unknown,
                verification_status: VerificationStatus::Unknown,
            });
        }
        analysis.cross_platform_matches = self.find_cross_platform_matches(&analysis.username).await?;
        let new_indicators = self.analyze_behavioral_indicators(&analysis.username, platform).await?;
        analysis.behavioral_indicators.extend(new_indicators);
        analysis.timestamp = Utc::now();
        Ok(analysis)
    }
    async fn update_cross_platform_index(&self, analysis: &UsernameAnalysis) {
        let mut index = self.cross_platform_index.write().await;
        let platforms: Vec<String> = analysis.platforms.iter().map(|p| p.name.clone()).collect();
        index.insert(analysis.username.clone(), platforms);
    }
    fn initialize_pattern_models(&self) {
    }
    pub async fn get_analysis(&self, username: &str) -> Option<UsernameAnalysis> {
        self.username_database.read().await.get(username).cloned()
    }
    pub async fn get_all_analyses(&self) -> HashMap<String, UsernameAnalysis> {
        self.username_database.read().await.clone()
    }
    pub async fn search_by_pattern(&self, pattern: UsernamePattern) -> Vec<UsernameAnalysis> {
        let database = self.username_database.read().await;
        database.values()
            .filter(|analysis| analysis.pattern_type == pattern)
            .cloned()
            .collect()
    }
    pub async fn get_high_risk_usernames(&self, threshold: f64) -> Vec<UsernameAnalysis> {
        let database = self.username_database.read().await;
        database.values()
            .filter(|analysis| analysis.risk_score >= threshold)
            .cloned()
            .collect()
    }
}
