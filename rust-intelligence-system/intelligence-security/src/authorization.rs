use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use intelligence_core::{IntelligenceError, Result as IntelligenceResult};
use crate::authentication::{Role, Permission, User};

/// Role-Based Access Control (RBAC) system

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: Uuid,
    pub name: String,
    pub resource_type: ResourceType,
    pub owner_id: Option<Uuid>,
    pub classification: DataClassification,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    IntelligenceData,
    UserProfile,
    AnalysisResult,
    SystemConfiguration,
    AgentConfiguration,
    AuditLog,
    Report,
    Dashboard,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Secret,
    TopSecret,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rules: Vec<AccessRule>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRule {
    pub id: Uuid,
    pub subject: AccessSubject,
    pub resource: AccessResource,
    pub actions: Vec<Action>,
    pub conditions: Vec<AccessCondition>,
    pub effect: AccessEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessSubject {
    User(Uuid),
    Role(Role),
    Group(String),
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessResource {
    ResourceType(ResourceType),
    SpecificResource(Uuid),
    ResourcePattern(String),
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    Read,
    Write,
    Delete,
    Execute,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessCondition {
    TimeRange {
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    },
    IpAddress {
        allowed_ips: Vec<String>,
    },
    DataClassification {
        max_classification: DataClassification,
    },
    Custom {
        name: String,
        value: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDecision {
    pub decision: AccessEffect,
    pub reason: String,
    pub matched_rules: Vec<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Authorization service
pub struct AuthorizationService {
    policies: HashMap<Uuid, AccessPolicy>,
    role_permissions: HashMap<Role, Vec<Permission>>,
    default_policies: Vec<AccessPolicy>,
}

impl AuthorizationService {
    pub fn new() -> Self {
        let mut service = Self {
            policies: HashMap::new(),
            role_permissions: HashMap::new(),
            default_policies: Vec::new(),
        };
        
        service.initialize_default_permissions();
        service.initialize_default_policies();
        service
    }
    
    /// Check if user has permission to perform action on resource
    pub async fn check_permission(
        &self,
        user: &User,
        action: &Action,
        resource: &Resource,
        context: &AccessContext,
    ) -> IntelligenceResult<AccessDecision> {
        let mut matched_rules = Vec::new();
        let mut decision = AccessEffect::Deny;
        let mut reason = "No matching policy found".to_string();
        
        // Check role-based permissions first
        if self.has_role_permission(user, action, resource) {
            decision = AccessEffect::Allow;
            reason = "Role-based permission granted".to_string();
        }
        
        // Check explicit policies
        for policy in self.policies.values() {
            if !policy.is_active {
                continue;
            }
            
            for rule in &policy.rules {
                if self.rule_matches(rule, user, action, resource, context) {
                    matched_rules.push(rule.id);
                    
                    match rule.effect {
                        AccessEffect::Allow => {
                            decision = AccessEffect::Allow;
                            reason = format!("Policy '{}' allows access", policy.name);
                        }
                        AccessEffect::Deny => {
                            decision = AccessEffect::Deny;
                            reason = format!("Policy '{}' denies access", policy.name);
                            break; // Deny takes precedence
                        }
                    }
                }
            }
        }
        
        // Check default policies
        for policy in &self.default_policies {
            if !policy.is_active {
                continue;
            }
            
            for rule in &policy.rules {
                if self.rule_matches(rule, user, action, resource, context) {
                    matched_rules.push(rule.id);
                    
                    match rule.effect {
                        AccessEffect::Allow => {
                            if decision == AccessEffect::Deny {
                                decision = AccessEffect::Allow;
                                reason = format!("Default policy '{}' allows access", policy.name);
                            }
                        }
                        AccessEffect::Deny => {
                            decision = AccessEffect::Deny;
                            reason = format!("Default policy '{}' denies access", policy.name);
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(AccessDecision {
            decision,
            reason,
            matched_rules,
            timestamp: chrono::Utc::now(),
        })
    }
    
    /// Create a new access policy
    pub async fn create_policy(&mut self, policy: AccessPolicy) -> IntelligenceResult<Uuid> {
        let policy_id = policy.id;
        self.policies.insert(policy_id, policy);
        Ok(policy_id)
    }
    
    /// Update an existing access policy
    pub async fn update_policy(&mut self, policy_id: Uuid, policy: AccessPolicy) -> IntelligenceResult<()> {
        if !self.policies.contains_key(&policy_id) {
            return Err(IntelligenceError::NotFound { resource: "Access policy".to_string() });
        }
        
        self.policies.insert(policy_id, policy);
        Ok(())
    }
    
    /// Delete an access policy
    pub async fn delete_policy(&mut self, policy_id: Uuid) -> IntelligenceResult<()> {
        if !self.policies.contains_key(&policy_id) {
            return Err(IntelligenceError::NotFound { resource: "Access policy".to_string() });
        }
        
        self.policies.remove(&policy_id);
        Ok(())
    }
    
    /// Get all policies
    pub async fn get_policies(&self) -> Vec<&AccessPolicy> {
        self.policies.values().collect()
    }
    
    /// Get policy by ID
    pub async fn get_policy(&self, policy_id: Uuid) -> IntelligenceResult<&AccessPolicy> {
        self.policies.get(&policy_id)
            .ok_or_else(|| IntelligenceError::NotFound { resource: "Access policy".to_string() })
    }
    
    // Private helper methods
    
    fn has_role_permission(&self, user: &User, action: &Action, resource: &Resource) -> bool {
        for role in &user.roles {
            if let Some(permissions) = self.role_permissions.get(role) {
                if self.permission_allows_action(permissions, action, resource) {
                    return true;
                }
            }
        }
        false
    }
    
    fn permission_allows_action(&self, permissions: &[Permission], action: &Action, resource: &Resource) -> bool {
        match action {
            Action::Read => permissions.contains(&Permission::ReadData),
            Action::Write => permissions.contains(&Permission::WriteData),
            Action::Delete => permissions.contains(&Permission::DeleteData),
            Action::Execute => permissions.contains(&Permission::ManageAgents),
            Action::Admin => permissions.contains(&Permission::ManageSystem),
        }
    }
    
    fn rule_matches(
        &self,
        rule: &AccessRule,
        user: &User,
        action: &Action,
        resource: &Resource,
        context: &AccessContext,
    ) -> bool {
        // Check subject
        if !self.subject_matches(&rule.subject, user) {
            return false;
        }
        
        // Check resource
        if !self.resource_matches(&rule.resource, resource) {
            return false;
        }
        
        // Check actions
        if !rule.actions.contains(action) {
            return false;
        }
        
        // Check conditions
        for condition in &rule.conditions {
            if !self.condition_matches(condition, user, resource, context) {
                return false;
            }
        }
        
        true
    }
    
    fn subject_matches(&self, subject: &AccessSubject, user: &User) -> bool {
        match subject {
            AccessSubject::User(user_id) => user.id == *user_id,
            AccessSubject::Role(role) => user.roles.contains(role),
            AccessSubject::Group(_) => false, // Group support would be implemented here
            AccessSubject::All => true,
        }
    }
    
    fn resource_matches(&self, resource_pattern: &AccessResource, resource: &Resource) -> bool {
        match resource_pattern {
            AccessResource::ResourceType(resource_type) => resource.resource_type == *resource_type,
            AccessResource::SpecificResource(resource_id) => resource.id == *resource_id,
            AccessResource::ResourcePattern(_) => false, // Pattern matching would be implemented here
            AccessResource::All => true,
        }
    }
    
    fn condition_matches(
        &self,
        condition: &AccessCondition,
        user: &User,
        resource: &Resource,
        context: &AccessContext,
    ) -> bool {
        match condition {
            AccessCondition::TimeRange { start, end } => {
                let now = chrono::Utc::now();
                now >= *start && now <= *end
            }
            AccessCondition::IpAddress { allowed_ips } => {
                if let Some(ip) = &context.ip_address {
                    allowed_ips.contains(ip)
                } else {
                    false
                }
            }
            AccessCondition::DataClassification { max_classification } => {
                self.classification_allowed(&resource.classification, max_classification)
            }
            AccessCondition::Custom { name, value } => {
                // Custom condition logic would be implemented here
                false
            }
        }
    }
    
    fn classification_allowed(&self, resource_classification: &DataClassification, max_classification: &DataClassification) -> bool {
        let classification_levels = [
            DataClassification::Public,
            DataClassification::Internal,
            DataClassification::Confidential,
            DataClassification::Secret,
            DataClassification::TopSecret,
        ];
        
        let resource_level = classification_levels.iter().position(|c| c == resource_classification).unwrap_or(0);
        let max_level = classification_levels.iter().position(|c| c == max_classification).unwrap_or(0);
        
        resource_level <= max_level
    }
    
    fn initialize_default_permissions(&mut self) {
        self.role_permissions.insert(Role::Admin, vec![
            Permission::ReadData,
            Permission::WriteData,
            Permission::DeleteData,
            Permission::ManageUsers,
            Permission::ManageAgents,
            Permission::ViewAnalytics,
            Permission::ManageSystem,
            Permission::AccessSensitiveData,
            Permission::ExportData,
            Permission::ImportData,
        ]);
        
        self.role_permissions.insert(Role::Analyst, vec![
            Permission::ReadData,
            Permission::WriteData,
            Permission::ViewAnalytics,
            Permission::AccessSensitiveData,
            Permission::ExportData,
        ]);
        
        self.role_permissions.insert(Role::Operator, vec![
            Permission::ReadData,
            Permission::ManageAgents,
            Permission::ViewAnalytics,
        ]);
        
        self.role_permissions.insert(Role::Viewer, vec![
            Permission::ReadData,
            Permission::ViewAnalytics,
        ]);
        
        self.role_permissions.insert(Role::Guest, vec![
            Permission::ReadData,
        ]);
    }
    
    fn initialize_default_policies(&mut self) {
        // Default policy: Deny all access to top secret data except for admins
        let deny_top_secret_policy = AccessPolicy {
            id: Uuid::new_v4(),
            name: "Deny Top Secret Access".to_string(),
            description: "Deny access to top secret data for non-admin users".to_string(),
            rules: vec![AccessRule {
                id: Uuid::new_v4(),
                subject: AccessSubject::Role(Role::Admin),
                resource: AccessResource::All,
                actions: vec![Action::Read, Action::Write, Action::Delete],
                conditions: vec![AccessCondition::DataClassification {
                    max_classification: DataClassification::TopSecret,
                }],
                effect: AccessEffect::Allow,
            }],
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        self.default_policies.push(deny_top_secret_policy);
    }
}

#[derive(Debug, Clone)]
pub struct AccessContext {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub additional_data: HashMap<String, serde_json::Value>,
}

impl AccessContext {
    pub fn new() -> Self {
        Self {
            ip_address: None,
            user_agent: None,
            timestamp: chrono::Utc::now(),
            additional_data: HashMap::new(),
        }
    }
    
    pub fn with_ip_address(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }
    
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }
}
