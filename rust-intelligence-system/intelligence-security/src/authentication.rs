use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use jwt_simple::prelude::*;
use anyhow::Result;
use thiserror::Error;
use intelligence_core::{IntelligenceError, Result as IntelligenceResult};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub roles: Vec<Role>,
    pub permissions: Vec<Permission>,
    pub is_active: bool,
    pub is_verified: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub failed_login_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    Admin,
    Analyst,
    Operator,
    Viewer,
    Guest,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    ReadData,
    WriteData,
    DeleteData,
    ManageUsers,
    ManageAgents,
    ViewAnalytics,
    ManageSystem,
    AccessSensitiveData,
    ExportData,
    ImportData,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: bool,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub user: UserInfo,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<Role>,
    pub permissions: Vec<Permission>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub user_id: Uuid,
    pub username: String,
    pub roles: Vec<Role>,
    pub permissions: Vec<Permission>,
    pub session_id: Uuid,
    pub issued_at: u64,
    pub expires_at: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub access_token_hash: String,
    pub refresh_token_hash: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub is_active: bool,
}
#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Account locked: {reason}")]
    AccountLocked { reason: String },
    #[error("Account not verified")]
    AccountNotVerified,
    #[error("Account disabled")]
    AccountDisabled,
    #[error("Too many failed login attempts")]
    TooManyFailedAttempts,
    #[error("Invalid token: {reason}")]
    InvalidToken { reason: String },
    #[error("Token expired")]
    TokenExpired,
    #[error("Session not found")]
    SessionNotFound,
    #[error("Password too weak: {reason}")]
    PasswordTooWeak { reason: String },
    #[error("User not found")]
    UserNotFound,
    #[error("User already exists")]
    UserAlreadyExists,
}
impl From<AuthenticationError> for IntelligenceError {
    fn from(err: AuthenticationError) -> Self {
        IntelligenceError::Authentication { message: err.to_string() }
    }
}
pub struct AuthenticationService {
    jwt_secret: String,
    jwt_expiry: Duration,
    refresh_token_expiry: Duration,
    max_failed_attempts: u32,
    lockout_duration: Duration,
    sessions: HashMap<Uuid, Session>,
    users: HashMap<Uuid, User>,
    username_index: HashMap<String, Uuid>,
    email_index: HashMap<String, Uuid>,
}
impl AuthenticationService {
    pub fn new(jwt_secret: String, jwt_expiry: Duration) -> Self {
        Self {
            jwt_secret,
            jwt_expiry,
            refresh_token_expiry: Duration::from_secs(7 * 24 * 3600),
            max_failed_attempts: 5,
            lockout_duration: Duration::from_secs(15 * 60),
            sessions: HashMap::new(),
            users: HashMap::new(),
            username_index: HashMap::new(),
            email_index: HashMap::new(),
        }
    }
    pub async fn register_user(
        &mut self,
        username: String,
        email: String,
        password: String,
        roles: Vec<Role>,
    ) -> IntelligenceResult<User> {
        self.validate_username(&username)?;
        self.validate_email(&email)?;
        self.validate_password(&password)?;
        if self.username_index.contains_key(&username) {
            return Err(AuthenticationError::UserAlreadyExists.into());
        }
        if self.email_index.contains_key(&email) {
            return Err(AuthenticationError::UserAlreadyExists.into());
        }
        let password_hash = self.hash_password(&password)?;
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        let user = User {
            id: user_id,
            username: username.clone(),
            email: email.clone(),
            password_hash,
            roles,
            permissions: Vec::new(),
            is_active: true,
            is_verified: false,
            last_login: None,
            failed_login_attempts: 0,
            locked_until: None,
            created_at: now,
            updated_at: now,
        };
        self.users.insert(user_id, user.clone());
        self.username_index.insert(username, user_id);
        self.email_index.insert(email, user_id);
        Ok(user)
    }
    pub async fn login(&mut self, request: LoginRequest) -> IntelligenceResult<LoginResponse> {
        let user_id = self.username_index.get(&request.username)
            .ok_or(AuthenticationError::InvalidCredentials)?;
        let user = self.users.get_mut(user_id)
            .ok_or(AuthenticationError::UserNotFound)?;
        if let Some(locked_until) = user.locked_until {
            if Utc::now() < locked_until {
                return Err(AuthenticationError::AccountLocked {
                    reason: "Too many failed login attempts".to_string()
                }.into());
            } else {
                user.locked_until = None;
                user.failed_login_attempts = 0;
            }
        }
        if !user.is_active {
            return Err(AuthenticationError::AccountDisabled.into());
        }
        if !self.verify_password(&request.password, &user.password_hash)? {
            user.failed_login_attempts += 1;
            if user.failed_login_attempts >= self.max_failed_attempts {
                user.locked_until = Some(Utc::now() + chrono::Duration::seconds(self.lockout_duration.as_secs() as i64));
            }
            return Err(AuthenticationError::InvalidCredentials.into());
        }
        user.failed_login_attempts = 0;
        user.locked_until = None;
        user.last_login = Some(Utc::now());
        let session = self.create_session(user.id, &request.ip_address, &request.user_agent)?;
        let access_token = self.generate_access_token(&user, &session)?;
        let refresh_token = self.generate_refresh_token(&session)?;
        let user_info = UserInfo {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            roles: user.roles.clone(),
            permissions: user.permissions.clone(),
        };
        Ok(LoginResponse {
            access_token,
            refresh_token,
            expires_in: self.jwt_expiry.as_secs(),
            user: user_info,
        })
    }
    pub async fn validate_token(&self, token: &str) -> IntelligenceResult<JwtClaims> {
        let key = HS256Key::from_bytes(self.jwt_secret.as_bytes());
        let claims = key.verify_token::<JwtClaims>(token, None)
            .map_err(|_| AuthenticationError::InvalidToken { reason: "Invalid signature".to_string() })?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        if claims.expires_at < now {
            return Err(AuthenticationError::TokenExpired.into());
        }
        if let Some(session) = self.sessions.get(&claims.session_id) {
            if !session.is_active || session.expires_at < Utc::now() {
                return Err(AuthenticationError::InvalidToken { reason: "Session expired".to_string() }.into());
            }
        } else {
            return Err(AuthenticationError::SessionNotFound.into());
        }
        Ok(claims)
    }
    pub async fn refresh_token(&mut self, refresh_token: &str) -> IntelligenceResult<LoginResponse> {
        let session_id = self.find_session_by_refresh_token(refresh_token)?;
        let session = self.sessions.get(&session_id)
            .ok_or(AuthenticationError::SessionNotFound)?;
        if !session.is_active || session.expires_at < Utc::now() {
            return Err(AuthenticationError::SessionNotFound.into());
        }
        let user = self.users.get(&session.user_id)
            .ok_or(AuthenticationError::UserNotFound)?;
        if !user.is_active {
            return Err(AuthenticationError::AccountDisabled.into());
        }
        let access_token = self.generate_access_token(user, session)?;
        let user_info = UserInfo {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            roles: user.roles.clone(),
            permissions: user.permissions.clone(),
        };
        Ok(LoginResponse {
            access_token,
            refresh_token: refresh_token.to_string(),
            expires_in: self.jwt_expiry.as_secs(),
            user: user_info,
        })
    }
    pub async fn logout(&mut self, session_id: Uuid) -> IntelligenceResult<()> {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.is_active = false;
        }
        Ok(())
    }
    pub async fn change_password(
        &mut self,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> IntelligenceResult<()> {
        let user = self.users.get_mut(&user_id)
            .ok_or(AuthenticationError::UserNotFound)?;
        if !self.verify_password(current_password, &user.password_hash)? {
            return Err(AuthenticationError::InvalidCredentials.into());
        }
        self.validate_password(new_password)?;
        let new_password_hash = self.hash_password(new_password)?;
        user.password_hash = new_password_hash;
        user.updated_at = Utc::now();
        Ok(())
    }
    fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        Ok(password_hash.to_string())
    }
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)?;
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
    fn validate_username(&self, username: &str) -> IntelligenceResult<()> {
        if username.len() < 3 || username.len() > 50 {
            return Err(AuthenticationError::PasswordTooWeak {
                reason: "Username must be between 3 and 50 characters".to_string()
            }.into());
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(AuthenticationError::PasswordTooWeak {
                reason: "Username can only contain alphanumeric characters, underscores, and hyphens".to_string()
            }.into());
        }
        Ok(())
    }
    fn validate_email(&self, email: &str) -> IntelligenceResult<()> {
        if !email.contains('@') || email.len() > 254 {
            return Err(AuthenticationError::PasswordTooWeak {
                reason: "Invalid email format".to_string()
            }.into());
        }
        Ok(())
    }
    fn validate_password(&self, password: &str) -> IntelligenceResult<()> {
        if password.len() < 12 {
            return Err(AuthenticationError::PasswordTooWeak {
                reason: "Password must be at least 12 characters long".to_string()
            }.into());
        }
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
        if !has_uppercase || !has_lowercase || !has_digit || !has_special {
            return Err(AuthenticationError::PasswordTooWeak {
                reason: "Password must contain uppercase, lowercase, digit, and special character".to_string()
            }.into());
        }
        Ok(())
    }
    fn create_session(
        &mut self,
        user_id: Uuid,
        ip_address: &Option<String>,
        user_agent: &Option<String>,
    ) -> IntelligenceResult<Session> {
        let session_id = Uuid::new_v4();
        let now = Utc::now();
        let session = Session {
            id: session_id,
            user_id,
            access_token_hash: String::new(),
            refresh_token_hash: String::new(),
            ip_address: ip_address.clone(),
            user_agent: user_agent.clone(),
            created_at: now,
            expires_at: now + chrono::Duration::seconds(self.refresh_token_expiry.as_secs() as i64),
            last_activity: now,
            is_active: true,
        };
        self.sessions.insert(session_id, session.clone());
        Ok(session)
    }
    fn generate_access_token(&self, user: &User, session: &Session) -> IntelligenceResult<String> {
        let key = HS256Key::from_bytes(self.jwt_secret.as_bytes());
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let claims = JwtClaims {
            user_id: user.id,
            username: user.username.clone(),
            roles: user.roles.clone(),
            permissions: user.permissions.clone(),
            session_id: session.id,
            issued_at: now,
            expires_at: now + self.jwt_expiry.as_secs(),
        };
        let token = key.authenticate(claims)
            .map_err(|_| AuthenticationError::InvalidToken { reason: "Token generation failed".to_string() })?;
        Ok(token)
    }
    fn generate_refresh_token(&self, session: &Session) -> IntelligenceResult<String> {
        let token = Uuid::new_v4().to_string();
        Ok(token)
    }
    fn find_session_by_refresh_token(&self, refresh_token: &str) -> IntelligenceResult<Uuid> {
        Uuid::parse_str(refresh_token)
            .map_err(|_| AuthenticationError::InvalidToken { reason: "Invalid refresh token".to_string() }.into())
    }
}
