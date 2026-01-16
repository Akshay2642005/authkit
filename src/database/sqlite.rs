#[cfg(feature = "sqlite")]
use crate::database::models::{DbSession, DbToken, DbUser};
use crate::database::DatabaseTrait;
use crate::error::Result;
use crate::types::User;
use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::str::FromStr;

#[derive(Clone)]
pub struct SqliteDatabase {
  pool: SqlitePool,
}

impl SqliteDatabase {
  pub async fn new(path: &str) -> Result<Self> {
    let options = SqliteConnectOptions::from_str(path)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
      .max_connections(5)
      .connect_with(options)
      .await?;

    Ok(Self { pool })
  }
}

#[async_trait]
impl DatabaseTrait for SqliteDatabase {
  async fn migrate(&self) -> Result<()> {
    // Users table
    sqlx::query(
      r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                email_verified BOOLEAN NOT NULL DEFAULT 0,
                email_verified_at INTEGER
            )
            "#,
    )
    .execute(&self.pool)
    .await?;

    // Sessions table
    sqlx::query(
      r#"
            CREATE TABLE IF NOT EXISTS sessions (
                token TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                expires_at INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
    )
    .execute(&self.pool)
    .await?;

    // Tokens table (unified for email verification, password reset, magic links, etc.)
    sqlx::query(
      r#"
            CREATE TABLE IF NOT EXISTS tokens (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                token_hash TEXT NOT NULL UNIQUE,
                token_type TEXT NOT NULL,
                expires_at INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                used_at INTEGER,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
    )
    .execute(&self.pool)
    .await?;

    // Create indexes for better query performance
    sqlx::query(
      r#"
            CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
            CREATE INDEX IF NOT EXISTS idx_tokens_user_id ON tokens(user_id);
            CREATE INDEX IF NOT EXISTS idx_tokens_token_hash ON tokens(token_hash);
            CREATE INDEX IF NOT EXISTS idx_tokens_expires_at ON tokens(expires_at);
            CREATE INDEX IF NOT EXISTS idx_tokens_type ON tokens(token_type)
            "#,
    )
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn find_user_by_email(&self, email: &str) -> Result<Option<DbUser>> {
    let user = sqlx::query(
      r#"
            SELECT id, email, password_hash, created_at, email_verified, email_verified_at
            FROM users
            WHERE email = ?
            "#,
    )
    .bind(email)
    .map(|row: sqlx::sqlite::SqliteRow| DbUser {
      id: row.get("id"),
      email: row.get("email"),
      password_hash: row.get("password_hash"),
      created_at: row.get("created_at"),
      email_verified: row.get("email_verified"),
      email_verified_at: row.get("email_verified_at"),
    })
    .fetch_optional(&self.pool)
    .await?;

    Ok(user)
  }

  async fn find_user_by_id(&self, id: &str) -> Result<Option<User>> {
    let user = sqlx::query(
      r#"
            SELECT id, email, password_hash, created_at, email_verified, email_verified_at
            FROM users
            WHERE id = ?
            "#,
    )
    .bind(id)
    .map(|row: sqlx::sqlite::SqliteRow| DbUser {
      id: row.get("id"),
      email: row.get("email"),
      password_hash: row.get("password_hash"),
      created_at: row.get("created_at"),
      email_verified: row.get("email_verified"),
      email_verified_at: row.get("email_verified_at"),
    })
    .fetch_optional(&self.pool)
    .await?;

    Ok(user.map(Into::into))
  }

  async fn create_user(
    &self,
    id: &str,
    email: &str,
    password_hash: &str,
    created_at: i64,
  ) -> Result<User> {
    sqlx::query(
      r#"
            INSERT INTO users (id, email, password_hash, created_at)
            VALUES (?, ?, ?, ?)
            "#,
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .bind(created_at)
    .execute(&self.pool)
    .await?;

    Ok(User {
      id: id.to_string(),
      email: email.to_string(),
      email_verified: false,
      email_verified_at: None,
      created_at,
    })
  }

  async fn update_email_verified(&self, user_id: &str, verified_at: i64) -> Result<()> {
    sqlx::query(
      r#"
            UPDATE users
            SET email_verified = 1, email_verified_at = ?
            WHERE id = ?
            "#,
    )
    .bind(verified_at)
    .bind(user_id)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn create_session(&self, token: &str, user_id: &str, expires_at: i64) -> Result<()> {
    let created_at = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;

    sqlx::query(
      r#"
            INSERT INTO sessions (token, user_id, expires_at, created_at)
            VALUES (?, ?, ?, ?)
            "#,
    )
    .bind(token)
    .bind(user_id)
    .bind(expires_at)
    .bind(created_at)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn find_session(&self, token: &str) -> Result<Option<DbSession>> {
    let session = sqlx::query(
      r#"
            SELECT token, user_id, expires_at, created_at
            FROM sessions
            WHERE token = ?
            "#,
    )
    .bind(token)
    .map(|row: sqlx::sqlite::SqliteRow| DbSession {
      token: row.get("token"),
      user_id: row.get("user_id"),
      expires_at: row.get("expires_at"),
      created_at: row.get("created_at"),
    })
    .fetch_optional(&self.pool)
    .await?;

    Ok(session)
  }

  async fn delete_session(&self, token: &str) -> Result<()> {
    sqlx::query(
      r#"
            DELETE FROM sessions
            WHERE token = ?
            "#,
    )
    .bind(token)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn delete_expired_sessions(&self) -> Result<u64> {
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;

    let result = sqlx::query(
      r#"
            DELETE FROM sessions
            WHERE expires_at < ?
            "#,
    )
    .bind(now)
    .execute(&self.pool)
    .await?;

    Ok(result.rows_affected())
  }

  // ==========================================
  // Token Operations
  // ==========================================

  async fn create_token(
    &self,
    id: &str,
    user_id: &str,
    token_hash: &str,
    token_type: &str,
    expires_at: i64,
    created_at: i64,
  ) -> Result<()> {
    sqlx::query(
      r#"
            INSERT INTO tokens (id, user_id, token_hash, token_type, expires_at, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(token_hash)
    .bind(token_type)
    .bind(expires_at)
    .bind(created_at)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn find_token(&self, token_hash: &str, token_type: &str) -> Result<Option<DbToken>> {
    let token = sqlx::query(
      r#"
            SELECT id, user_id, token_hash, token_type, expires_at, created_at, used_at
            FROM tokens
            WHERE token_hash = ? AND token_type = ?
            "#,
    )
    .bind(token_hash)
    .bind(token_type)
    .map(|row: sqlx::sqlite::SqliteRow| DbToken {
      id: row.get("id"),
      user_id: row.get("user_id"),
      token_hash: row.get("token_hash"),
      token_type: row.get("token_type"),
      expires_at: row.get("expires_at"),
      created_at: row.get("created_at"),
      used_at: row.get("used_at"),
    })
    .fetch_optional(&self.pool)
    .await?;

    Ok(token)
  }

  async fn mark_token_used(&self, token_hash: &str, used_at: i64) -> Result<()> {
    sqlx::query(
      r#"
            UPDATE tokens
            SET used_at = ?
            WHERE token_hash = ?
            "#,
    )
    .bind(used_at)
    .bind(token_hash)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn delete_token(&self, token_hash: &str) -> Result<()> {
    sqlx::query(
      r#"
            DELETE FROM tokens
            WHERE token_hash = ?
            "#,
    )
    .bind(token_hash)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn delete_expired_tokens(&self) -> Result<u64> {
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;

    let result = sqlx::query(
      r#"
            DELETE FROM tokens
            WHERE expires_at < ?
            "#,
    )
    .bind(now)
    .execute(&self.pool)
    .await?;

    Ok(result.rows_affected())
  }
}
