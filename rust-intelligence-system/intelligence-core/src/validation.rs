use crate::types::*;
use validator::{Validate, ValidationError};
use std::collections::HashMap;
pub fn validate_intelligence_data(data: &IntelligenceData) -> Result<(), ValidationError> {
    data.validate()?;
    if data.confidence < 0.0 || data.confidence > 1.0 {
        return Err(ValidationError::new("confidence_range"));
    }
    if data.quality_score < 0.0 || data.quality_score > 1.0 {
        return Err(ValidationError::new("quality_score_range"));
    }
    if data.content.trim().is_empty() {
        return Err(ValidationError::new("content_empty"));
    }
    Ok(())
}
pub fn validate_user_profile(profile: &UserProfile) -> Result<(), ValidationError> {
    profile.validate()?;
    if profile.risk_score < 0.0 || profile.risk_score > 1.0 {
        return Err(ValidationError::new("risk_score_range"));
    }
    if profile.username.trim().is_empty() {
        return Err(ValidationError::new("username_empty"));
    }
    Ok(())
}
pub fn validate_threat_assessment(assessment: &ThreatAssessment) -> Result<(), ValidationError> {
    if assessment.confidence < 0.0 || assessment.confidence > 1.0 {
        return Err(ValidationError::new("confidence_range"));
    }
    if assessment.indicators.is_empty() {
        return Err(ValidationError::new("indicators_empty"));
    }
    for indicator in &assessment.indicators {
        if indicator.confidence < 0.0 || indicator.confidence > 1.0 {
            return Err(ValidationError::new("indicator_confidence_range"));
        }
        if indicator.value.trim().is_empty() {
            return Err(ValidationError::new("indicator_value_empty"));
        }
    }
    Ok(())
}
pub fn validate_analysis_result(result: &AnalysisResult) -> Result<(), ValidationError> {
    if result.confidence < 0.0 || result.confidence > 1.0 {
        return Err(ValidationError::new("confidence_range"));
    }
    if result.input_data.is_empty() {
        return Err(ValidationError::new("input_data_empty"));
    }
    if result.results.is_empty() {
        return Err(ValidationError::new("results_empty"));
    }
    Ok(())
}
pub fn validate_agent_task(task: &AgentTask) -> Result<(), ValidationError> {
    if task.input_data.is_empty() {
        return Err(ValidationError::new("input_data_empty"));
    }
    if let Some(deadline) = task.deadline {
        if deadline <= task.created_at {
            return Err(ValidationError::new("invalid_deadline"));
        }
    }
    Ok(())
}
pub fn sanitize_string(input: &str, max_length: usize) -> Result<String, ValidationError> {
    let sanitized = input.trim();
    if sanitized.is_empty() {
        return Err(ValidationError::new("empty_string"));
    }
    if sanitized.len() > max_length {
        return Err(ValidationError::new("string_too_long"));
    }
    let cleaned = sanitized
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
        .collect::<String>();
    Ok(cleaned)
}
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if email.is_empty() {
        return Err(ValidationError::new("email_empty"));
    }
    if !email.contains('@') {
        return Err(ValidationError::new("email_invalid_format"));
    }
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(ValidationError::new("email_invalid_format"));
    }
    if parts[0].is_empty() || parts[1].is_empty() {
        return Err(ValidationError::new("email_invalid_format"));
    }
    if !parts[1].contains('.') {
        return Err(ValidationError::new("email_invalid_format"));
    }
    Ok(())
}
pub fn validate_ip_address(ip: &str) -> Result<(), ValidationError> {
    if ip.is_empty() {
        return Err(ValidationError::new("ip_empty"));
    }
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() == 4 {
        for part in parts {
            if let Ok(num) = part.parse::<u8>() {
                if num > 255 {
                    return Err(ValidationError::new("ip_invalid_range"));
                }
            } else {
                return Err(ValidationError::new("ip_invalid_format"));
            }
        }
        return Ok(());
    }
    if ip.contains(':') {
        return Ok(());
    }
    Err(ValidationError::new("ip_invalid_format"))
}
pub fn validate_crypto_address(address: &str, currency: &str) -> Result<(), ValidationError> {
    if address.is_empty() {
        return Err(ValidationError::new("crypto_address_empty"));
    }
    match currency.to_lowercase().as_str() {
        "bitcoin" | "btc" => {
            if !address.starts_with('1') && !address.starts_with('3') && !address.starts_with("bc1") {
                return Err(ValidationError::new("bitcoin_address_invalid"));
            }
        }
        "ethereum" | "eth" => {
            if !address.starts_with("0x") || address.len() != 42 {
                return Err(ValidationError::new("ethereum_address_invalid"));
            }
        }
        _ => {
            if address.len() < 10 || address.len() > 100 {
                return Err(ValidationError::new("crypto_address_invalid_length"));
            }
        }
    }
    Ok(())
}
pub fn validate_metadata(metadata: &HashMap<String, serde_json::Value>) -> Result<(), ValidationError> {
    if metadata.len() > 100 {
        return Err(ValidationError::new("metadata_too_large"));
    }
    for (key, value) in metadata {
        if key.len() > 100 {
            return Err(ValidationError::new("metadata_key_too_long"));
        }
        let serialized = serde_json::to_string(value).unwrap_or_default();
        if serialized.len() > 10000 {
            return Err(ValidationError::new("metadata_value_too_large"));
        }
    }
    Ok(())
}
