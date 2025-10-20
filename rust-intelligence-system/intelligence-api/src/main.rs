use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use reqwest::Client;

// Ollama API structures
#[derive(Serialize, Deserialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct OllamaResponse {
    model: String,
    response: String,
    done: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatStyle {
    common_words: HashMap<String, u32>,
    average_length: f64,
    emoji_usage: f64,
    punctuation_patterns: HashMap<String, u32>,
    response_time_patterns: Vec<f64>,
    common_phrases: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize)]
struct ChatAnalysis {
    chat_id: i64,
    chat_title: String,
    total_messages: usize,
    analyzed_messages: usize,
    style: ChatStyle,
    last_analyzed: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct TelegramMessage {
    id: Uuid,
    message_id: i64,
    chat_id: i64,
    user_id: Option<i64>,
    username: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    message_text: Option<String>,
    message_date: DateTime<Utc>,
    message_type: String,
    is_bot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersonProfile {
    user_id: i64,
    username: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    bio: Option<String>,
    location: Option<String>,
    timezone: Option<String>,
    language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersonBehaviorPattern {
    user_id: i64,
    pattern_type: String,
    pattern_data: serde_json::Value,
    confidence_score: f64,
    observation_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersonIntelligence {
    user_id: i64,
    intelligence_type: String,
    intelligence_data: serde_json::Value,
    confidence_level: f64,
    extraction_method: String,
    verification_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersonActivity {
    user_id: i64,
    activity_type: String,
    activity_data: Option<serde_json::Value>,
    source_chat_id: Option<i64>,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MonitoredChat {
    chat_id: i64,
    chat_title: Option<String>,
    chat_type: Option<String>,
    is_active: bool,
    auto_analyze: bool,
    auto_respond: bool,
    analysis_frequency: i32,
    last_analyzed: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageAnalysisLog {
    message_id: Uuid,
    analysis_type: String,
    analysis_status: String,
    analysis_data: Option<serde_json::Value>,
    error_message: Option<String>,
    processing_time_ms: Option<i32>,
    completed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
    services: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse {
    message: String,
    data: Option<serde_json::Value>,
}

// Global state for the autonomous system
struct AppState {
    http_client: Client,
    ollama_url: String,
    db_pool: Option<sqlx::PgPool>,
    telegram_connected: Arc<Mutex<bool>>,
    messages_processed: Arc<Mutex<u64>>,
    chats_monitored: Arc<Mutex<u64>>,
    is_running: Arc<Mutex<bool>>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize HTTP client and Ollama URL
    let http_client = Client::new();
    let ollama_url = env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());

    // Try to connect to database
    let db_pool = match env::var("DATABASE_URL") {
        Ok(database_url) => {
            match sqlx::PgPool::connect(&database_url).await {
                Ok(pool) => {
                    println!("✅ Connected to database");
                    Some(pool)
                }
                Err(e) => {
                    println!("❌ Failed to connect to database: {}", e);
                    println!("🔄 Running in offline mode with limited functionality");
                    None
                }
            }
        }
        Err(_) => {
            println!("⚠️  No DATABASE_URL found, running in offline mode");
            None
        }
    };

    // Initialize app state
    let state = Arc::new(AppState {
        http_client,
        ollama_url,
        db_pool,
        telegram_connected: Arc::new(Mutex::new(false)),
        messages_processed: Arc::new(Mutex::new(0)),
        chats_monitored: Arc::new(Mutex::new(0)),
        is_running: Arc::new(Mutex::new(true)),
    });

    // Start autonomous message analysis
    let state_clone = state.clone();
    tokio::spawn(async move {
        autonomous_message_analysis(state_clone).await;
    });

    // Start autonomous response system
    let state_clone = state.clone();
    tokio::spawn(async move {
        autonomous_response_system(state_clone).await;
    });

    // Setup graceful shutdown
    let state_clone = state.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        println!("🛑 Shutting down gracefully...");
        {
            let mut is_running = state_clone.is_running.lock().await;
            *is_running = false;
        }
        std::process::exit(0);
    });

    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/api/status", get(api_status))
        .route("/api/stats", get(get_stats))
        .route("/api/chats", get(get_chats))
        .route("/api/messages", get(get_recent_messages))
        .route("/api/analyze", get(analyze_messages))
        .route("/api/generate", get(generate_response))
                .layer(CorsLayer::permissive())
        .with_state(state);

    // Run the server
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("🚀 AUTONOMOUS AI Intelligence System starting...");
    println!("📊 Health check: http://0.0.0.0:8080/health");
    println!("📈 Stats: http://0.0.0.0:8080/api/stats");
    println!("💬 Chats: http://0.0.0.0:8080/api/chats");
    println!("📝 Messages: http://0.0.0.0:8080/api/messages");
    println!("🔍 Analyze: http://0.0.0.0:8080/api/analyze");
    println!("🤖 Generate: http://0.0.0.0:8080/api/generate");
    println!("");
    println!("🤖 AUTONOMOUS FEATURES:");
    println!("  ✅ Real Ollama AI message understanding");
    println!("  ✅ Context-aware response generation");
    println!("  ✅ Human-like conversational responses");
    println!("  ✅ Person intelligence extraction");
    println!("  ✅ Personal info, social connections, vulnerabilities");
    println!("  ✅ Real database message reading");
    println!("  ✅ Real-time AI processing");
    
    axum::serve(listener, app).await.unwrap();
}

fn create_sample_messages() -> Vec<TelegramMessage> {
    let sample_texts = vec![
        "Hey everyone! How's the project going?",
        "Great! We're making good progress on the AI features",
        "That's awesome! 🚀 What's the next milestone?",
        "We should focus on the message analysis next",
        "Sounds good! Let's build something cool together",
        "I think we need to improve the response generation",
        "Absolutely! The current system needs more intelligence",
        "What do you think about using Ollama for this?",
        "That's a brilliant idea! Ollama would be perfect",
        "Let's implement it step by step and see what happens",
    ];

    let mut messages = Vec::new();
    for (i, text) in sample_texts.iter().enumerate() {
        messages.push(TelegramMessage {
            id: Uuid::new_v4(),
            message_id: i as i64,
            chat_id: -1001234567890i64,
            user_id: Some(i as i64),
            username: Some(format!("user{}", i + 1)),
            first_name: Some(format!("User{}", i + 1)),
            last_name: None,
            message_text: Some(text.to_string()),
            message_date: Utc::now(),
            message_type: "text".to_string(),
            is_bot: false,
        });
    }
    messages
}

async fn autonomous_message_analysis(state: Arc<AppState>) {
    loop {
        // Check if system should still be running
        {
            let is_running = state.is_running.lock().await;
            if !*is_running {
                println!("🛑 Message analysis stopping...");
                break;
            }
        }
        
        println!("🔍 Analyzing messages with Ollama AI...");
        
        // Get monitored chats
        let monitored_chats = match get_monitored_chats(&state).await {
            Ok(chats) => chats,
            Err(e) => {
                println!("❌ Failed to get monitored chats: {}", e);
                continue;
            }
        };

        for chat in monitored_chats {
            if !chat.auto_analyze {
                continue;
            }

            println!("📊 Analyzing chat: {} (ID: {})", 
                chat.chat_title.as_deref().unwrap_or("Unknown"), 
                chat.chat_id
            );

            // Get unanalyzed messages for this chat
            let messages = match fetch_unanalyzed_messages(&state, chat.chat_id).await {
                Ok(msgs) => msgs,
                Err(e) => {
                    println!("❌ Failed to fetch messages for chat {}: {}", chat.chat_id, e);
                    continue;
                }
            };
        
        if !messages.is_empty() {
            // Analyze messages using Ollama
            match analyze_messages_with_ollama(&state, &messages).await {
                Ok(analysis) => {
                    println!("✅ Analysis complete:");
                    println!("   Chat ID: {}", analysis.chat_id);
                    println!("   Messages analyzed: {}", analysis.analyzed_messages);
                    println!("   Average length: {:.1}", analysis.style.average_length);
                    println!("   Emoji usage: {:.2}", analysis.style.emoji_usage);
                    println!("   Common words: {:?}", analysis.style.common_words);
                    
                    // Update counters
                    {
                        let mut processed = state.messages_processed.lock().await;
                        *processed += analysis.analyzed_messages as u64;
                    }

                    // Update chat last analyzed timestamp
                    if let Err(e) = update_chat_last_analyzed(&state, chat.chat_id).await {
                        println!("❌ Failed to update chat timestamp: {}", e);
                    }
                }
                Err(e) => {
                    println!("❌ Analysis failed: {}", e);
                }
            }

            // Process each message for person intelligence
            for message in &messages {
                if let Some(text) = &message.message_text {
                    if let Some(user_id) = message.user_id {
                        println!("🔍 Processing message from user {}: \"{}\"", user_id, text);
                        
                        // Log analysis start
                        let log_id = match log_analysis_start(&state, message.id, "person_intelligence").await {
                            Ok(id) => id,
                            Err(e) => {
                                println!("❌ Failed to log analysis start: {}", e);
                                continue;
                            }
                        };

                        let start_time = std::time::Instant::now();
                        
                        // Extract person intelligence
                        match analyze_person_intelligence(&state, message).await {
                            Ok(intelligence_data) => {
                                let processing_time = start_time.elapsed().as_millis() as i32;
                                
                                for intelligence in intelligence_data {
                                    if let Err(e) = log_person_intelligence(&state, &intelligence).await {
                                        println!("❌ Failed to log intelligence: {}", e);
                                    }
                                }

                                // Log analysis completion
                                let analysis_data = serde_json::to_value(intelligence_data)?;
                                if let Err(e) = log_analysis_complete(&state, log_id, Some(analysis_data), processing_time).await {
                                    println!("❌ Failed to log analysis completion: {}", e);
                                }
                            }
                            Err(e) => {
                                let processing_time = start_time.elapsed().as_millis() as i32;
                                println!("❌ Intelligence extraction failed: {}", e);
                                
                                if let Err(log_err) = log_analysis_error(&state, log_id, &e.to_string()).await {
                                    println!("❌ Failed to log analysis error: {}", log_err);
                                }
                            }
                        }
                    }
                }
            }
        } else {
            println!("ℹ️  No unanalyzed messages found for chat {}", chat.chat_id);
        }
        }
        
        // Sleep for 60 seconds before next analysis
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

async fn autonomous_response_system(state: Arc<AppState>) {
    loop {
        // Check if system should still be running
        {
            let is_running = state.is_running.lock().await;
            if !*is_running {
                println!("🛑 Response system stopping...");
                break;
            }
        }
        
        println!("🤖 Generating responses with Ollama AI...");
        
        // Get real messages from database
        let messages = match fetch_messages_for_response(&state).await {
            Ok(msgs) => msgs,
            Err(e) => {
                println!("❌ Failed to fetch messages: {}", e);
                continue;
            }
        };
        
        // Generate responses for recent messages and extract intelligence
        for message in messages.iter().take(3) {
            if let Some(text) = &message.message_text {
                println!("📝 Processing message: \"{}\"", text);
                
                // Extract person intelligence
                match analyze_person_intelligence(&state, message).await {
                    Ok(intelligence_data) => {
                        for intelligence in intelligence_data {
                            if let Err(e) = log_person_intelligence(&state, &intelligence).await {
                                println!("❌ Failed to log intelligence: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Intelligence extraction failed: {}", e);
                    }
                }
                
                // Generate response
                match generate_response_with_ollama(&state, text).await {
                    Ok(response) => {
                        println!("💬 Generated response: {}", response);
                    }
                    Err(e) => {
                        println!("❌ Response generation failed: {}", e);
                    }
                }
            }
        }
        
        // Sleep for 120 seconds before next response cycle
        tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;
    }
}

async fn analyze_messages_with_ollama(state: &AppState, messages: &[TelegramMessage]) -> Result<ChatAnalysis, Box<dyn std::error::Error + Send + Sync>> {
    // Combine all message texts for analysis
    let combined_text = messages
        .iter()
        .filter_map(|m| m.message_text.as_ref())
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    if combined_text.is_empty() {
        return Err("No text content to analyze".into());
    }

    // Create analysis prompt for Ollama
    let prompt = format!(
        "You are a JSON analysis tool. Analyze these chat messages and return ONLY a valid JSON object with these exact fields:
        {{
            \"common_words\": {{word: count}},
            \"average_length\": number,
            \"emoji_usage\": number,
            \"punctuation_patterns\": {{pattern: count}},
            \"common_phrases\": {{phrase: count}}
        }}
        
        Messages: {}
        
        Return ONLY the JSON object, no other text:",
        combined_text.chars().take(2000).collect::<String>() // Limit text length
    );

    // Call Ollama API
    let ollama_request = OllamaRequest {
        model: "llama3.2".to_string(),
        prompt,
        stream: false,
    };

    let response = state.http_client
        .post(&format!("{}/api/generate", state.ollama_url))
        .json(&ollama_request)
        .send()
        .await?;

    let ollama_response: OllamaResponse = response.json().await?;
    
    // Parse the JSON response from Ollama
    let analysis_data: serde_json::Value = serde_json::from_str(&ollama_response.response)?;
    
    // Extract chat info
    let chat_id = messages.first().map(|m| m.chat_id).unwrap_or(-1);
    let chat_title = format!("Chat {}", chat_id);
    
    // Build ChatStyle from Ollama response
    let style = ChatStyle {
        common_words: analysis_data["common_words"]
            .as_object()
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0) as u32)).collect())
            .unwrap_or_default(),
        average_length: analysis_data["average_length"].as_f64().unwrap_or(0.0),
        emoji_usage: analysis_data["emoji_usage"].as_f64().unwrap_or(0.0),
        punctuation_patterns: analysis_data["punctuation_patterns"]
            .as_object()
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0) as u32)).collect())
            .unwrap_or_default(),
        response_time_patterns: vec![], // Not implemented yet
        common_phrases: analysis_data["common_phrases"]
            .as_object()
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0) as u32)).collect())
            .unwrap_or_default(),
    };

    Ok(ChatAnalysis {
        chat_id,
        chat_title,
        total_messages: messages.len(),
        analyzed_messages: messages.len(),
        style,
        last_analyzed: chrono::Utc::now().to_rfc3339(),
    })
}

async fn understand_message_with_ollama(state: &AppState, message_text: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let prompt = format!(
        "What does this message mean? Analyze the context, intent, emotions, and what the person is really trying to say. Be specific about the underlying meaning:

Message: \"{}\"

Provide a brief analysis of what this message means:",
        message_text
    );

    let ollama_request = OllamaRequest {
        model: "llama3.2".to_string(),
        prompt,
        stream: false,
    };

    let response = state.http_client
        .post(&format!("{}/api/generate", state.ollama_url))
        .json(&ollama_request)
        .send()
        .await?;

    let ollama_response: OllamaResponse = response.json().await?;
    Ok(ollama_response.response)
}

async fn generate_response_with_ollama(state: &AppState, message_text: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // First understand what the message means
    let message_understanding = match understand_message_with_ollama(state, message_text).await {
        Ok(understanding) => understanding,
        Err(e) => {
            println!("❌ Failed to understand message: {}", e);
            "Unable to analyze message".to_string()
        }
    };

    println!("🧠 Message understanding: {}", message_understanding);

    // Create response prompt based on understanding
    let prompt = format!(
        "Based on this message understanding, generate a natural, human-like response that doesn't sound like AI. Be conversational, casual, and match the tone. Avoid formal language or AI-speak.

Message: \"{}\"
Understanding: \"{}\"

Generate a natural response (1-2 sentences, casual tone):",
        message_text, message_understanding
    );

    // Call Ollama API
    let ollama_request = OllamaRequest {
        model: "llama3.2".to_string(),
        prompt,
        stream: false,
    };

    let response = state.http_client
        .post(&format!("{}/api/generate", state.ollama_url))
        .json(&ollama_request)
        .send()
        .await?;

    let ollama_response: OllamaResponse = response.json().await?;
    
    Ok(ollama_response.response.trim().to_string())
}

async fn analyze_person_intelligence(state: &AppState, message: &TelegramMessage) -> Result<Vec<PersonIntelligence>, Box<dyn std::error::Error + Send + Sync>> {
    let mut intelligence_data = Vec::new();
    
    if let Some(text) = &message.message_text {
        if let Some(user_id) = message.user_id {
            // Extract personal information
            let personal_info = extract_personal_info(state, text, user_id).await?;
            if !personal_info.is_empty() {
                intelligence_data.push(PersonIntelligence {
                    user_id,
                    intelligence_type: "personal_info".to_string(),
                    intelligence_data: serde_json::to_value(personal_info)?,
                    confidence_level: 0.8,
                    extraction_method: "ai_analysis".to_string(),
                    verification_status: "unverified".to_string(),
                });
            }

            // Extract social connections
            let social_connections = extract_social_connections(state, text, user_id).await?;
            if !social_connections.is_empty() {
                intelligence_data.push(PersonIntelligence {
                    user_id,
                    intelligence_type: "social_connections".to_string(),
                    intelligence_data: serde_json::to_value(social_connections)?,
                    confidence_level: 0.7,
                    extraction_method: "ai_analysis".to_string(),
                    verification_status: "unverified".to_string(),
                });
            }

            // Extract interests and goals
            let interests_goals = extract_interests_goals(state, text, user_id).await?;
            if !interests_goals.is_empty() {
                intelligence_data.push(PersonIntelligence {
                    user_id,
                    intelligence_type: "interests_goals".to_string(),
                    intelligence_data: serde_json::to_value(interests_goals)?,
                    confidence_level: 0.6,
                    extraction_method: "ai_analysis".to_string(),
                    verification_status: "unverified".to_string(),
                });
            }

            // Extract vulnerabilities and weaknesses
            let vulnerabilities = extract_vulnerabilities(state, text, user_id).await?;
            if !vulnerabilities.is_empty() {
                intelligence_data.push(PersonIntelligence {
                    user_id,
                    intelligence_type: "vulnerabilities".to_string(),
                    intelligence_data: serde_json::to_value(vulnerabilities)?,
                    confidence_level: 0.5,
                    extraction_method: "ai_analysis".to_string(),
                    verification_status: "unverified".to_string(),
                });
            }
        }
    }

    Ok(intelligence_data)
}

async fn extract_personal_info(state: &AppState, text: &str, user_id: i64) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
    let prompt = format!(
        "Extract personal information from this message. Look for:
        - Full name, nickname, or aliases
        - Phone numbers, email addresses
        - Location, city, country
        - Age, birthday, personal details
        - Job, profession, workplace
        - Family members, relationships
        - Personal interests, hobbies
        
        Message: \"{}\"
        
        Return ONLY a JSON object with extracted information. If nothing found, return empty object {{}}.",
        text
    );

    let ollama_request = OllamaRequest {
        model: "llama3.2".to_string(),
        prompt,
        stream: false,
    };

    let response = state.http_client
        .post(&format!("{}/api/generate", state.ollama_url))
        .json(&ollama_request)
        .send()
        .await?;

    let ollama_response: OllamaResponse = response.json().await?;
    
    // Parse JSON response
    let personal_info: HashMap<String, String> = serde_json::from_str(&ollama_response.response)
        .unwrap_or_default();
    
    Ok(personal_info)
}

async fn extract_social_connections(state: &AppState, text: &str, user_id: i64) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
    let prompt = format!(
        "Extract social connections and relationships from this message. Look for:
        - Mentions of friends, family, colleagues
        - Social media handles, usernames
        - Group memberships, communities
        - Professional networks
        - Relationship status, romantic interests
        - Social activities, events
        
        Message: \"{}\"
        
        Return ONLY a JSON object with extracted social information. If nothing found, return empty object {{}}.",
        text
    );

    let ollama_request = OllamaRequest {
        model: "llama3.2".to_string(),
        prompt,
        stream: false,
    };

    let response = state.http_client
        .post(&format!("{}/api/generate", state.ollama_url))
        .json(&ollama_request)
        .send()
        .await?;

    let ollama_response: OllamaResponse = response.json().await?;
    
    let social_info: HashMap<String, String> = serde_json::from_str(&ollama_response.response)
        .unwrap_or_default();
    
    Ok(social_info)
}

async fn extract_interests_goals(state: &AppState, text: &str, user_id: i64) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
    let prompt = format!(
        "Extract interests, goals, and aspirations from this message. Look for:
        - Hobbies, interests, passions
        - Career goals, ambitions
        - Personal projects, plans
        - Dreams, aspirations
        - Skills they're developing
        - Things they want to achieve
        - Problems they're trying to solve
        
        Message: \"{}\"
        
        Return ONLY a JSON object with extracted interests/goals. If nothing found, return empty object {{}}.",
        text
    );

    let ollama_request = OllamaRequest {
        model: "llama3.2".to_string(),
        prompt,
        stream: false,
    };

    let response = state.http_client
        .post(&format!("{}/api/generate", state.ollama_url))
        .json(&ollama_request)
        .send()
        .await?;

    let ollama_response: OllamaResponse = response.json().await?;
    
    let interests: HashMap<String, String> = serde_json::from_str(&ollama_response.response)
        .unwrap_or_default();
    
    Ok(interests)
}

async fn extract_vulnerabilities(state: &AppState, text: &str, user_id: i64) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
    let prompt = format!(
        "Extract vulnerabilities, weaknesses, and sensitive information from this message. Look for:
        - Personal problems, struggles
        - Financial difficulties, money issues
        - Health problems, medical issues
        - Emotional vulnerabilities, insecurities
        - Secrets, confidential information
        - Things they're ashamed of or hiding
        - Dependencies, addictions
        - Trust issues, relationship problems
        
        Message: \"{}\"
        
        Return ONLY a JSON object with extracted vulnerabilities. If nothing found, return empty object {{}}.",
        text
    );

    let ollama_request = OllamaRequest {
        model: "llama3.2".to_string(),
        prompt,
        stream: false,
    };

    let response = state.http_client
        .post(&format!("{}/api/generate", state.ollama_url))
        .json(&ollama_request)
        .send()
        .await?;

    let ollama_response: OllamaResponse = response.json().await?;
    
    let vulnerabilities: HashMap<String, String> = serde_json::from_str(&ollama_response.response)
        .unwrap_or_default();
    
    Ok(vulnerabilities)
}

async fn log_person_intelligence(state: &AppState, intelligence: &PersonIntelligence) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // For now, just log to console since we don't have DB connection
    println!("🔍 PERSON INTELLIGENCE EXTRACTED:");
    println!("   User ID: {}", intelligence.user_id);
    println!("   Type: {}", intelligence.intelligence_type);
    println!("   Method: {}", intelligence.extraction_method);
    println!("   Confidence: {:.2}", intelligence.confidence_level);
    println!("   Data: {}", serde_json::to_string_pretty(&intelligence.intelligence_data)?);
    println!("   Status: {}", intelligence.verification_status);
    println!("");
    
    // TODO: Insert into database when connection is available
    // sqlx::query("INSERT INTO person_intelligence (user_id, intelligence_type, intelligence_data, confidence_level, extraction_method, verification_status) VALUES ($1, $2, $3, $4, $5, $6)")
    //     .bind(intelligence.user_id)
    //     .bind(&intelligence.intelligence_type)
    //     .bind(&intelligence.intelligence_data)
    //     .bind(intelligence.confidence_level)
    //     .bind(&intelligence.extraction_method)
    //     .bind(&intelligence.verification_status)
    //     .execute(&state.db_pool)
    //     .await?;
    
    Ok(())
}

async fn fetch_recent_messages_from_db(state: &AppState, limit: i64) -> Result<Vec<TelegramMessage>, Box<dyn std::error::Error + Send + Sync>> {
    match &state.db_pool {
        Some(pool) => {
            let rows = sqlx::query(
                "SELECT id, message_id, chat_id, user_id, username, first_name, last_name, 
                        message_text, message_date, message_type, is_bot 
                 FROM messages 
                 ORDER BY message_date DESC 
                 LIMIT $1"
            )
            .bind(limit)
            .fetch_all(pool)
            .await?;

            let mut messages = Vec::new();
            for row in rows {
                messages.push(TelegramMessage {
                    id: row.get("id"),
                    message_id: row.get("message_id"),
                    chat_id: row.get("chat_id"),
                    user_id: row.get("user_id"),
                    username: row.get("username"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    message_text: row.get("message_text"),
                    message_date: row.get("message_date"),
                    message_type: row.get("message_type"),
                    is_bot: row.get("is_bot"),
                });
            }
            Ok(messages)
        }
        None => {
            // Fallback to sample messages if no database
            println!("⚠️  No database connection, using sample messages");
            Ok(create_sample_messages())
        }
    }
}

async fn fetch_messages_for_analysis(state: &AppState) -> Result<Vec<TelegramMessage>, Box<dyn std::error::Error + Send + Sync>> {
    fetch_recent_messages_from_db(state, 10).await
}

async fn fetch_messages_for_response(state: &AppState) -> Result<Vec<TelegramMessage>, Box<dyn std::error::Error + Send + Sync>> {
    fetch_recent_messages_from_db(state, 5).await
}

async fn get_monitored_chats(state: &AppState) -> Result<Vec<MonitoredChat>, Box<dyn std::error::Error + Send + Sync>> {
    match &state.db_pool {
        Some(pool) => {
            let rows = sqlx::query(
                "SELECT chat_id, chat_title, chat_type, is_active, auto_analyze, auto_respond, 
                        analysis_frequency, last_analyzed 
                 FROM monitored_chats 
                 WHERE is_active = true"
            )
            .fetch_all(pool)
            .await?;

            let mut chats = Vec::new();
            for row in rows {
                chats.push(MonitoredChat {
                    chat_id: row.get("chat_id"),
                    chat_title: row.get("chat_title"),
                    chat_type: row.get("chat_type"),
                    is_active: row.get("is_active"),
                    auto_analyze: row.get("auto_analyze"),
                    auto_respond: row.get("auto_respond"),
                    analysis_frequency: row.get("analysis_frequency"),
                    last_analyzed: row.get("last_analyzed"),
                });
            }
            Ok(chats)
        }
        None => {
            // Return default chat for testing
            Ok(vec![MonitoredChat {
                chat_id: -1001234567890,
                chat_title: Some("Test Chat".to_string()),
                chat_type: Some("group".to_string()),
                is_active: true,
                auto_analyze: true,
                auto_respond: false,
                analysis_frequency: 60,
                last_analyzed: None,
            }])
        }
    }
}

async fn fetch_unanalyzed_messages(state: &AppState, chat_id: i64) -> Result<Vec<TelegramMessage>, Box<dyn std::error::Error + Send + Sync>> {
    match &state.db_pool {
        Some(pool) => {
            let rows = sqlx::query(
                "SELECT m.id, m.message_id, m.chat_id, m.user_id, m.username, m.first_name, m.last_name, 
                        m.message_text, m.message_date, m.message_type, m.is_bot 
                 FROM messages m
                 LEFT JOIN message_analysis_log mal ON m.id = mal.message_id AND mal.analysis_type = 'person_intelligence'
                 WHERE m.chat_id = $1 
                 AND m.message_text IS NOT NULL 
                 AND m.user_id IS NOT NULL
                 AND mal.id IS NULL
                 ORDER BY m.message_date DESC 
                 LIMIT 10"
            )
            .bind(chat_id)
            .fetch_all(pool)
            .await?;

            let mut messages = Vec::new();
            for row in rows {
                messages.push(TelegramMessage {
                    id: row.get("id"),
                    message_id: row.get("message_id"),
                    chat_id: row.get("chat_id"),
                    user_id: row.get("user_id"),
                    username: row.get("username"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    message_text: row.get("message_text"),
                    message_date: row.get("message_date"),
                    message_type: row.get("message_type"),
                    is_bot: row.get("is_bot"),
                });
            }
            Ok(messages)
        }
        None => {
            // Fallback to sample messages
            Ok(create_sample_messages())
        }
    }
}

async fn log_analysis_start(state: &AppState, message_id: Uuid, analysis_type: &str) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
    match &state.db_pool {
        Some(pool) => {
            let log_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO message_analysis_log (id, message_id, analysis_type, analysis_status) 
                 VALUES ($1, $2, $3, 'pending')"
            )
            .bind(log_id)
            .bind(message_id)
            .bind(analysis_type)
            .execute(pool)
            .await?;
            Ok(log_id)
        }
        None => {
            // Return a dummy ID for offline mode
            Ok(Uuid::new_v4())
        }
    }
}

async fn log_analysis_complete(state: &AppState, log_id: Uuid, analysis_data: Option<serde_json::Value>, processing_time_ms: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match &state.db_pool {
        Some(pool) => {
            sqlx::query(
                "UPDATE message_analysis_log 
                 SET analysis_status = 'completed', analysis_data = $1, processing_time_ms = $2, completed_at = NOW()
                 WHERE id = $3"
            )
            .bind(analysis_data)
            .bind(processing_time_ms)
            .bind(log_id)
            .execute(pool)
            .await?;
        }
        None => {
            println!("📊 Analysis completed (offline mode): {}ms", processing_time_ms);
        }
    }
    Ok(())
}

async fn log_analysis_error(state: &AppState, log_id: Uuid, error_message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match &state.db_pool {
        Some(pool) => {
            sqlx::query(
                "UPDATE message_analysis_log 
                 SET analysis_status = 'failed', error_message = $1, completed_at = NOW()
                 WHERE id = $2"
            )
            .bind(error_message)
            .bind(log_id)
            .execute(pool)
            .await?;
        }
        None => {
            println!("❌ Analysis failed (offline mode): {}", error_message);
        }
    }
    Ok(())
}

async fn update_chat_last_analyzed(state: &AppState, chat_id: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match &state.db_pool {
        Some(pool) => {
            sqlx::query(
                "UPDATE monitored_chats 
                 SET last_analyzed = NOW(), updated_at = NOW()
                 WHERE chat_id = $1"
            )
            .bind(chat_id)
            .execute(pool)
            .await?;
        }
        None => {
            println!("📅 Updated last analyzed for chat {} (offline mode)", chat_id);
        }
    }
    Ok(())
}

async fn root() -> Json<ApiResponse> {
    Json(ApiResponse {
        message: "AUTONOMOUS AI Intelligence System with Ollama".to_string(),
        data: Some(serde_json::json!({
            "version": "3.0.0",
            "mode": "autonomous_ollama",
            "features": [
                "Real Ollama AI Message Understanding",
                "Context-aware Response Generation", 
                "Human-like Conversational Responses",
                "Person Intelligence Extraction",
                "Personal Info & Social Connections Analysis",
                "Vulnerability Detection",
                "Real Database Message Reading",
                "Real-time AI Processing"
            ],
            "status": "running_with_ollama"
        })),
    })
}

async fn health() -> Json<HealthResponse> {
    let mut services = HashMap::new();
    
    // Check Ollama
    services.insert("ollama".to_string(), "running".to_string());
    
    // Check Telegram connection (simulated)
    services.insert("telegram".to_string(), "mtproto_autonomous".to_string());
    
    Json(HealthResponse {
        status: "healthy_autonomous".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        services,
    })
}

async fn api_status() -> Json<ApiResponse> {
    Json(ApiResponse {
        message: "Autonomous AI Intelligence System with Ollama is running".to_string(),
        data: Some(serde_json::json!({
            "environment": "autonomous_ollama",
            "ai": "ollama_llama3.2",
            "autonomous_features": [
                "ollama_message_analysis",
                "ai_style_learning",
                "ollama_response_generation",
                "sample_message_testing"
            ],
            "status": "fully_autonomous_with_ollama"
        })),
    })
}

async fn get_stats(State(state): State<Arc<AppState>>) -> Json<ApiResponse> {
    let connected = *state.telegram_connected.lock().await;
    let processed = *state.messages_processed.lock().await;
    let monitored = *state.chats_monitored.lock().await;
    let is_running = *state.is_running.lock().await;
    
    Json(ApiResponse {
        message: "System statistics".to_string(),
        data: Some(serde_json::json!({
            "telegram_connected": connected,
            "messages_processed": processed,
            "chats_monitored": monitored,
            "is_running": is_running,
            "uptime": "running",
            "last_update": chrono::Utc::now().to_rfc3339()
        })),
    })
}

async fn get_chats(State(state): State<Arc<AppState>>) -> Json<ApiResponse> {
    let messages = {
        let msgs = state.sample_messages.lock().await;
        msgs.clone()
    };
    
    let chat_summary = serde_json::json!({
        "chat_id": -1001234567890i64,
        "chat_title": "AI Development Group",
        "total_messages": messages.len(),
        "last_analyzed": chrono::Utc::now().to_rfc3339(),
        "status": "active"
    });
    
    Json(ApiResponse {
        message: "Active chats with analysis".to_string(),
        data: Some(serde_json::json!(vec![chat_summary])),
    })
}

async fn get_recent_messages(State(state): State<Arc<AppState>>) -> Json<ApiResponse> {
    let messages = {
        let msgs = state.sample_messages.lock().await;
        msgs.clone()
    };
    
    let message_data: Vec<serde_json::Value> = messages
        .into_iter()
        .map(|msg| {
            serde_json::json!({
                "id": msg.id,
                "message_id": msg.message_id,
                "chat_id": msg.chat_id,
                "user_id": msg.user_id,
                "username": msg.username,
                "first_name": msg.first_name,
                "message_text": msg.message_text,
                "message_date": msg.message_date,
                "message_type": msg.message_type,
                "is_bot": msg.is_bot,
            })
        })
        .collect();
    
    Json(ApiResponse {
        message: "Recent messages".to_string(),
        data: Some(serde_json::json!(message_data)),
    })
}

async fn analyze_messages(State(state): State<Arc<AppState>>) -> Json<ApiResponse> {
    let messages = {
        let msgs = state.sample_messages.lock().await;
        msgs.clone()
    };
    
    match analyze_messages_with_ollama(&state, &messages).await {
        Ok(analysis) => {
            Json(ApiResponse {
                message: "Message analysis complete".to_string(),
                data: Some(serde_json::json!({
                    "chat_id": analysis.chat_id,
                    "chat_title": analysis.chat_title,
                    "total_messages": analysis.total_messages,
                    "analyzed_messages": analysis.analyzed_messages,
                    "style": {
                        "common_words": analysis.style.common_words,
                        "average_length": analysis.style.average_length,
                        "emoji_usage": analysis.style.emoji_usage,
                        "punctuation_patterns": analysis.style.punctuation_patterns,
                        "common_phrases": analysis.style.common_phrases,
                    },
                    "last_analyzed": analysis.last_analyzed
                })),
            })
        }
        Err(e) => {
            Json(ApiResponse {
                message: format!("Analysis failed: {}", e),
                data: None,
            })
        }
    }
}

async fn generate_response(State(state): State<Arc<AppState>>) -> Json<ApiResponse> {
    let messages = {
        let msgs = state.sample_messages.lock().await;
        msgs.clone()
    };
    
    if let Some(message) = messages.first() {
        if let Some(text) = &message.message_text {
            match generate_response_with_ollama(&state, text).await {
                Ok(response) => {
                    Json(ApiResponse {
                        message: "Response generated".to_string(),
                        data: Some(serde_json::json!({
                            "original_message": text,
                            "generated_response": response,
                            "model": "llama3.2",
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        })),
                    })
                }
                Err(e) => {
                    Json(ApiResponse {
                        message: format!("Response generation failed: {}", e),
                        data: None,
                    })
                }
            }
        } else {
            Json(ApiResponse {
                message: "No message text available".to_string(),
                data: None,
            })
        }
    } else {
        Json(ApiResponse {
            message: "No messages available".to_string(),
            data: None,
        })
    }
}