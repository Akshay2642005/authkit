#[cfg(feature = "sqlite")]
use crate::database::models::{DbAccount, DbSession, DbUser, DbUserWithAccount, DbVerification};
use crate::database::DatabaseTrait;
use crate::error::Result;
use crate::types::User;
use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::str::FromStr;

#[derive(Clone)]
pub struct SqliteDatabase {
  pub(crate) pool: SqlitePool,
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
  // ==========================================
  // User Operations
  // ==========================================

  async fn find_user_by_email(&self, email: &str) -> Result<Option<DbUser>> {
    // Query base columns only - email_verified columns are optional (added by email_verification feature)
    let user = sqlx::query(
      r#"
      SELECT id, email, name, created_at, updated_at
      FROM users
      WHERE email = ?
      "#,
    )
    .bind(email)
    .map(|row: sqlx::sqlite::SqliteRow| DbUser {
      id: row.get("id"),
      email: row.get("email"),
      name: row.get("name"),
      created_at: row.get("created_at"),
      updated_at: row.get("updated_at"),
      email_verified: None,
      email_verified_at: None,
    })
    .fetch_optional(&self.pool)
    .await?;

    Ok(user)
  }

  async fn find_user_by_id(&self, id: &str) -> Result<Option<User>> {
    // Query base columns only - email_verified columns are optional (added by email_verification feature)
    let user = sqlx::query(
      r#"
      SELECT id, email, name, created_at, updated_at
      FROM users
      WHERE id = ?
      "#,
    )
    .bind(id)
    .map(|row: sqlx::sqlite::SqliteRow| DbUser {
      id: row.get("id"),
      email: row.get("email"),
      name: row.get("name"),
      created_at: row.get("created_at"),
      updated_at: row.get("updated_at"),
      email_verified: None,
      email_verified_at: None,
    })
    .fetch_optional(&self.pool)
    .await?;

    Ok(user.map(Into::into))
  }

  async fn create_user(
    &self,
    id: &str,
    email: &str,
    name: Option<&str>,
    created_at: i64,
  ) -> Result<User> {
    sqlx::query(
      r#"
      INSERT INTO users (id, email, name, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?)
      "#,
    )
    .bind(id)
    .bind(email)
    .bind(name)
    .bind(created_at)
    .bind(created_at)
    .execute(&self.pool)
    .await?;

    Ok(User {
      id: id.to_string(),
      email: email.to_string(),
      name: name.map(|s| s.to_string()),
      email_verified: false,
      email_verified_at: None,
      created_at,
      updated_at: created_at,
    })
  }

  async fn update_email_verified(&self, user_id: &str, verified_at: i64) -> Result<()> {
    sqlx::query(
      r#"
      UPDATE users
      SET email_verified = 1, email_verified_at = ?, updated_at = ?
      WHERE id = ?
      "#,
    )
    .bind(verified_at)
    .bind(verified_at)
    .bind(user_id)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn find_user_by_id_with_verification(&self, id: &str) -> Result<Option<User>> {
    // Queries email_verified columns - requires email_verification feature migration
    let user = sqlx::query(
      r#"
      SELECT id, email, name, created_at, updated_at, email_verified, email_verified_at
      FROM users
      WHERE id = ?
      "#,
    )
    .bind(id)
    .map(|row: sqlx::sqlite::SqliteRow| {
      let email_verified: i32 = row.get("email_verified");
      DbUser {
        id: row.get("id"),
        email: row.get("email"),
        name: row.get("name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        email_verified: Some(email_verified != 0),
        email_verified_at: row.get("email_verified_at"),
      }
    })
    .fetch_optional(&self.pool)
    .await?;

    Ok(user.map(Into::into))
  }

  async fn find_user_by_email_with_verification(&self, email: &str) -> Result<Option<DbUser>> {
    // Queries email_verified columns - requires email_verification feature migration
    let user = sqlx::query(
      r#"
      SELECT id, email, name, created_at, updated_at, email_verified, email_verified_at
      FROM users
      WHERE email = ?
      "#,
    )
    .bind(email)
    .map(|row: sqlx::sqlite::SqliteRow| {
      let email_verified: i32 = row.get("email_verified");
      DbUser {
        id: row.get("id"),
        email: row.get("email"),
        name: row.get("name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        email_verified: Some(email_verified != 0),
        email_verified_at: row.get("email_verified_at"),
      }
    })
    .fetch_optional(&self.pool)
    .await?;

    Ok(user)
  }

  // ==========================================
  // Account Operations
  // ==========================================

  async fn create_account(
    &self,
    id: &str,
    user_id: &str,
    provider: &str,
    provider_account_id: &str,
    password_hash: Option<&str>,
    created_at: i64,
  ) -> Result<()> {
    sqlx::query(
      r#"
      INSERT INTO accounts (id, user_id, provider, provider_account_id, password_hash, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(provider)
    .bind(provider_account_id)
    .bind(password_hash)
    .bind(created_at)
    .bind(created_at)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn find_account_by_provider(
    &self,
    provider: &str,
    provider_account_id: &str,
  ) -> Result<Option<DbAccount>> {
    let account = sqlx::query(
      r#"
      SELECT id, user_id, provider, provider_account_id, password_hash, created_at, updated_at
      FROM accounts
      WHERE provider = ? AND provider_account_id = ?
      "#,
    )
    .bind(provider)
    .bind(provider_account_id)
    .map(|row: sqlx::sqlite::SqliteRow| DbAccount {
      id: row.get("id"),
      user_id: row.get("user_id"),
      provider: row.get("provider"),
      provider_account_id: row.get("provider_account_id"),
      password_hash: row.get("password_hash"),
      created_at: row.get("created_at"),
      updated_at: row.get("updated_at"),
    })
    .fetch_optional(&self.pool)
    .await?;

    Ok(account)
  }

  async fn find_user_with_credential_account(
    &self,
    email: &str,
  ) -> Result<Option<DbUserWithAccount>> {
    // Query base columns only - email_verified columns are optional (added by email_verification feature)
    let result = sqlx::query(
      r#"
      SELECT
        u.id as user_id, u.email, u.name, u.created_at as user_created_at,
        u.updated_at as user_updated_at,
        a.id as account_id, a.provider, a.provider_account_id, a.password_hash,
        a.created_at as account_created_at, a.updated_at as account_updated_at
      FROM users u
      INNER JOIN accounts a ON u.id = a.user_id
      WHERE u.email = ? AND a.provider = 'credential'
      "#,
    )
    .bind(email)
    .map(|row: sqlx::sqlite::SqliteRow| {
      let user = DbUser {
        id: row.get("user_id"),
        email: row.get("email"),
        name: row.get("name"),
        created_at: row.get("user_created_at"),
        updated_at: row.get("user_updated_at"),
        email_verified: None,
        email_verified_at: None,
      };
      let account = DbAccount {
        id: row.get("account_id"),
        user_id: row.get("user_id"),
        provider: row.get("provider"),
        provider_account_id: row.get("provider_account_id"),
        password_hash: row.get("password_hash"),
        created_at: row.get("account_created_at"),
        updated_at: row.get("account_updated_at"),
      };
      DbUserWithAccount { user, account }
    })
    .fetch_optional(&self.pool)
    .await?;

    Ok(result)
  }

  async fn find_user_with_credential_account_with_verification(
    &self,
    email: &str,
  ) -> Result<Option<DbUserWithAccount>> {
    // Queries email_verified columns - requires email_verification feature migration
    let result = sqlx::query(
      r#"
      SELECT
        u.id as user_id, u.email, u.name, u.created_at as user_created_at,
        u.updated_at as user_updated_at, u.email_verified, u.email_verified_at,
        a.id as account_id, a.provider, a.provider_account_id, a.password_hash,
        a.created_at as account_created_at, a.updated_at as account_updated_at
      FROM users u
      INNER JOIN accounts a ON u.id = a.user_id
      WHERE u.email = ? AND a.provider = 'credential'
      "#,
    )
    .bind(email)
    .map(|row: sqlx::sqlite::SqliteRow| {
      let email_verified: i32 = row.get("email_verified");
      let user = DbUser {
        id: row.get("user_id"),
        email: row.get("email"),
        name: row.get("name"),
        created_at: row.get("user_created_at"),
        updated_at: row.get("user_updated_at"),
        email_verified: Some(email_verified != 0),
        email_verified_at: row.get("email_verified_at"),
      };
      let account = DbAccount {
        id: row.get("account_id"),
        user_id: row.get("user_id"),
        provider: row.get("provider"),
        provider_account_id: row.get("provider_account_id"),
        password_hash: row.get("password_hash"),
        created_at: row.get("account_created_at"),
        updated_at: row.get("account_updated_at"),
      };
      DbUserWithAccount { user, account }
    })
    .fetch_optional(&self.pool)
    .await?;

    Ok(result)
  }

  // ==========================================
  // Session Operations
  // ==========================================

  async fn create_session(
    &self,
    id: &str,
    token: &str,
    user_id: &str,
    expires_at: i64,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
  ) -> Result<()> {
    let created_at = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;

    sqlx::query(
      r#"
      INSERT INTO sessions (id, token, user_id, expires_at, created_at, ip_address, user_agent)
      VALUES (?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(id)
    .bind(token)
    .bind(user_id)
    .bind(expires_at)
    .bind(created_at)
    .bind(ip_address)
    .bind(user_agent)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn find_session(&self, token: &str) -> Result<Option<DbSession>> {
    let session = sqlx::query(
      r#"
      SELECT id, token, user_id, expires_at, created_at, ip_address, user_agent
      FROM sessions
      WHERE token = ?
      "#,
    )
    .bind(token)
    .map(|row: sqlx::sqlite::SqliteRow| DbSession {
      id: row.get("id"),
      token: row.get("token"),
      user_id: row.get("user_id"),
      expires_at: row.get("expires_at"),
      created_at: row.get("created_at"),
      ip_address: row.get("ip_address"),
      user_agent: row.get("user_agent"),
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
  // Verification Token Operations
  // ==========================================

  async fn create_verification(
    &self,
    id: &str,
    user_id: Option<&str>,
    identifier: &str,
    token_hash: &str,
    token_type: &str,
    expires_at: i64,
    created_at: i64,
  ) -> Result<()> {
    sqlx::query(
      r#"
      INSERT INTO verification (id, user_id, identifier, token_hash, token_type, expires_at, created_at)
      VALUES (?, ?, ?, ?, ?, ?, ?)
      "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(identifier)
    .bind(token_hash)
    .bind(token_type)
    .bind(expires_at)
    .bind(created_at)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn find_verification(
    &self,
    token_hash: &str,
    token_type: &str,
  ) -> Result<Option<DbVerification>> {
    let token = sqlx::query(
      r#"
      SELECT id, user_id, identifier, token_hash, token_type, expires_at, created_at, used_at
      FROM verification
      WHERE token_hash = ? AND token_type = ?
      "#,
    )
    .bind(token_hash)
    .bind(token_type)
    .map(|row: sqlx::sqlite::SqliteRow| DbVerification {
      id: row.get("id"),
      user_id: row.get("user_id"),
      identifier: row.get("identifier"),
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

  async fn mark_verification_used(&self, token_hash: &str, used_at: i64) -> Result<()> {
    sqlx::query(
      r#"
      UPDATE verification
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

  async fn delete_verification(&self, token_hash: &str) -> Result<()> {
    sqlx::query(
      r#"
      DELETE FROM verification
      WHERE token_hash = ?
      "#,
    )
    .bind(token_hash)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  async fn delete_expired_verifications(&self) -> Result<u64> {
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;

    let result = sqlx::query(
      r#"
      DELETE FROM verification
      WHERE expires_at < ?
      "#,
    )
    .bind(now)
    .execute(&self.pool)
    .await?;

    Ok(result.rows_affected())
  }
}
