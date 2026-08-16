//! Local users + Bearer API tokens (optional by default).

use std::collections::HashMap;
use std::sync::Arc;

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use axum::extract::{Request, State};
use axum::http::{header, Method};
use axum::middleware::Next;
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

const MIN_PASSWORD_LEN: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user: AuthUser,
    pub token_id: Uuid,
}

#[derive(Clone)]
pub struct AuthStore {
    memory: Arc<Mutex<MemoryAuth>>,
    postgres: Option<PgPool>,
}

#[derive(Default)]
struct MemoryAuth {
    users_by_email: HashMap<String, StoredUser>,
    users_by_id: HashMap<Uuid, StoredUser>,
    tokens_by_hash: HashMap<String, StoredToken>,
}

#[derive(Clone)]
struct StoredUser {
    id: Uuid,
    email: String,
    password_hash: String,
    created_at: DateTime<Utc>,
}

#[derive(Clone)]
struct StoredToken {
    id: Uuid,
    user_id: Uuid,
    token_hash: String,
    label: String,
    created_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

impl AuthStore {
    pub fn new(postgres: Option<PgPool>) -> Self {
        Self {
            memory: Arc::new(Mutex::new(MemoryAuth::default())),
            postgres,
        }
    }

    pub async fn register(&self, email: &str, password: &str) -> ApiResult<(AuthUser, String)> {
        let email = normalize_email(email)?;
        validate_password(password)?;
        if self.find_stored_by_email(&email).await?.is_some() {
            return Err(ApiError::conflict("email already registered"));
        }
        let password_hash = hash_password(password)?;
        let user = StoredUser {
            id: Uuid::new_v4(),
            email: email.clone(),
            password_hash,
            created_at: Utc::now(),
        };
        self.insert_user(&user).await?;
        let token = self.issue_token(user.id, "session").await?;
        Ok((user.into_public(), token))
    }

    pub async fn login(&self, email: &str, password: &str) -> ApiResult<(AuthUser, String)> {
        let email = normalize_email(email)?;
        let user = self
            .find_stored_by_email(&email)
            .await?
            .ok_or_else(|| ApiError::unauthorized("invalid email or password"))?;
        verify_password(password, &user.password_hash)?;
        let token = self.issue_token(user.id, "session").await?;
        Ok((user.into_public(), token))
    }

    pub async fn logout(&self, token_id: Uuid) -> ApiResult<()> {
        if let Some(pool) = &self.postgres {
            sqlx::query(
                "UPDATE api_tokens SET revoked_at = NOW() WHERE id = $1 AND revoked_at IS NULL",
            )
            .bind(token_id)
            .execute(pool)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
        }
        let mut mem = self.memory.lock().await;
        if let Some(token) = mem.tokens_by_hash.values_mut().find(|t| t.id == token_id) {
            token.revoked_at = Some(Utc::now());
        }
        Ok(())
    }

    pub async fn resolve_bearer(&self, raw_token: &str) -> ApiResult<Option<AuthContext>> {
        let hash = hash_token(raw_token);
        let stored = if let Some(pool) = &self.postgres {
            sqlx::query_as::<_, TokenRow>(
                r#"
                SELECT id, user_id, token_hash, label, created_at, revoked_at
                FROM api_tokens
                WHERE token_hash = $1
                "#,
            )
            .bind(&hash)
            .fetch_optional(pool)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?
            .map(StoredToken::from)
        } else {
            let mem = self.memory.lock().await;
            mem.tokens_by_hash.get(&hash).cloned()
        };

        let Some(token) = stored else {
            return Ok(None);
        };
        if token.revoked_at.is_some() {
            return Ok(None);
        }
        let Some(user) = self.get_user(token.user_id).await? else {
            return Ok(None);
        };
        Ok(Some(AuthContext {
            user,
            token_id: token.id,
        }))
    }

    pub async fn get_user(&self, id: Uuid) -> ApiResult<Option<AuthUser>> {
        if let Some(pool) = &self.postgres {
            let row = sqlx::query_as::<_, UserRow>(
                "SELECT id, email, password_hash, created_at FROM users WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
            return Ok(row.map(|r| AuthUser {
                id: r.id,
                email: r.email,
                created_at: r.created_at,
            }));
        }
        let mem = self.memory.lock().await;
        Ok(mem.users_by_id.get(&id).map(StoredUser::into_public))
    }

    async fn find_stored_by_email(&self, email: &str) -> ApiResult<Option<StoredUser>> {
        if let Some(pool) = &self.postgres {
            let row = sqlx::query_as::<_, UserRow>(
                "SELECT id, email, password_hash, created_at FROM users WHERE email = $1",
            )
            .bind(email)
            .fetch_optional(pool)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
            return Ok(row.map(StoredUser::from));
        }
        let mem = self.memory.lock().await;
        Ok(mem.users_by_email.get(email).cloned())
    }

    pub async fn find_by_email(&self, email: &str) -> ApiResult<Option<AuthUser>> {
        Ok(self
            .find_stored_by_email(email)
            .await?
            .map(|u| u.into_public()))
    }

    async fn insert_user(&self, user: &StoredUser) -> ApiResult<()> {
        if let Some(pool) = &self.postgres {
            sqlx::query(
                "INSERT INTO users (id, email, password_hash, created_at) VALUES ($1, $2, $3, $4)",
            )
            .bind(user.id)
            .bind(&user.email)
            .bind(&user.password_hash)
            .bind(user.created_at)
            .execute(pool)
            .await
            .map_err(|err| {
                if err.to_string().contains("duplicate") || err.to_string().contains("unique") {
                    ApiError::conflict("email already registered")
                } else {
                    ApiError::internal(err.to_string())
                }
            })?;
            return Ok(());
        }
        let mut mem = self.memory.lock().await;
        if mem.users_by_email.contains_key(&user.email) {
            return Err(ApiError::conflict("email already registered"));
        }
        mem.users_by_email.insert(user.email.clone(), user.clone());
        mem.users_by_id.insert(user.id, user.clone());
        Ok(())
    }

    async fn issue_token(&self, user_id: Uuid, label: &str) -> ApiResult<String> {
        let raw = generate_token();
        let token = StoredToken {
            id: Uuid::new_v4(),
            user_id,
            token_hash: hash_token(&raw),
            label: label.to_string(),
            created_at: Utc::now(),
            revoked_at: None,
        };
        if let Some(pool) = &self.postgres {
            sqlx::query(
                r#"
                INSERT INTO api_tokens (id, user_id, token_hash, label, created_at, revoked_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(token.id)
            .bind(token.user_id)
            .bind(&token.token_hash)
            .bind(&token.label)
            .bind(token.created_at)
            .bind(token.revoked_at)
            .execute(pool)
            .await
            .map_err(|err| ApiError::internal(err.to_string()))?;
        } else {
            let mut mem = self.memory.lock().await;
            mem.tokens_by_hash
                .insert(token.token_hash.clone(), token);
        }
        Ok(raw)
    }
}

impl StoredUser {
    fn into_public(&self) -> AuthUser {
        AuthUser {
            id: self.id,
            email: self.email.clone(),
            created_at: self.created_at,
        }
    }
}

impl From<UserRow> for StoredUser {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            password_hash: row.password_hash,
            created_at: row.created_at,
        }
    }
}

impl From<TokenRow> for StoredToken {
    fn from(row: TokenRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            token_hash: row.token_hash,
            label: row.label,
            created_at: row.created_at,
            revoked_at: row.revoked_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: String,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct TokenRow {
    id: Uuid,
    user_id: Uuid,
    token_hash: String,
    label: String,
    created_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

fn normalize_email(email: &str) -> ApiResult<String> {
    let email = email.trim().to_ascii_lowercase();
    if email.is_empty() || !email.contains('@') || email.len() > 320 {
        return Err(ApiError::bad_request("invalid email"));
    }
    Ok(email)
}

fn validate_password(password: &str) -> ApiResult<()> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(ApiError::bad_request(format!(
            "password must be at least {MIN_PASSWORD_LEN} characters"
        )));
    }
    if password.len() > 256 {
        return Err(ApiError::bad_request("password too long"));
    }
    Ok(())
}

fn hash_password(password: &str) -> ApiResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| ApiError::internal(format!("password hash: {err}")))
}

fn verify_password(password: &str, hash: &str) -> ApiResult<()> {
    let parsed = PasswordHash::new(hash)
        .map_err(|_| ApiError::unauthorized("invalid email or password"))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| ApiError::unauthorized("invalid email or password"))
}

fn generate_token() -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn hash_token(raw: &str) -> String {
    let digest = Sha256::digest(raw.as_bytes());
    hex::encode(digest)
}

pub fn is_public_route(method: &Method, path: &str) -> bool {
    matches!(
        (method.as_str(), path),
        ("GET", "/api/v1/health")
            | ("GET", "/api/v1/demos")
            | ("POST", "/api/v1/auth/register")
            | ("POST", "/api/v1/auth/login")
    )
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> impl IntoResponse {
    if let Some(raw) = bearer_token(request.headers()) {
        match state.auth.resolve_bearer(&raw).await {
            Ok(Some(ctx)) => {
                request.extensions_mut().insert(ctx);
            }
            Ok(None) => {}
            Err(err) => return err.into_response(),
        }
    }

    if state.config.auth_required
        && !is_public_route(request.method(), request.uri().path())
        && request.extensions().get::<AuthContext>().is_none()
    {
        return ApiError::unauthorized("authentication required").into_response();
    }

    // Session endpoints always require a valid Bearer token.
    let path = request.uri().path();
    if matches!(path, "/api/v1/auth/me" | "/api/v1/auth/logout")
        && request.extensions().get::<AuthContext>().is_none()
    {
        return ApiError::unauthorized("authentication required").into_response();
    }

    next.run(request).await
}

fn bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))?;
    let token = token.trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}
