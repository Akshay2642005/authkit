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
  /// Creates a new SqliteDatabase connected to the SQLite database at `path`.
  ///
  /// `path` should be a SQLite connection string or file path supported by `SqliteConnectOptions`.
  /// The function ensures the database file is created if missing and configures a connection pool
  /// with a maximum of 5 connections.
  ///
  /// # Examples
  ///
  /// ```
  /// # use crate::database::sqlite::SqliteDatabase;
  /// # async fn _example() -> anyhow::Result<()> {
  /// let db = SqliteDatabase::new("sqlite://./data/db.sqlite").await?;
  /// // use `db`...
  /// # Ok(())
  /// # }
  /// ```
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
  /// Ensures the required database schema and indexes for users, sessions, and tokens exist.
  ///
  /// Creates the `users`, `sessions`, and unified `tokens` tables if they do not already exist,
  /// and adds indexes used for common lookups and expiry queries.
  ///
  /// # Examples
  ///
  /// ```
  /// #[tokio::main]
  /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
  ///     let db = SqliteDatabase::new(":memory:").await?;
  ///     db.migrate().await?;
  ///     Ok(())
  /// }
  /// ```
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

  /// Fetches a user record matching the given email from the database.
  ///
  /// # Returns
  ///
  /// `Some(DbUser)` with the user's database record if a matching email exists, `None` otherwise.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # async fn example(db: &crate::SqliteDatabase) -> anyhow::Result<()> {
  /// let user = db.find_user_by_email("alice@example.com").await?;
  /// if let Some(u) = user {
  ///     println!("Found user: {}", u.email);
  /// }
  /// # Ok(()) }
  /// ```
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

  /// Retrieves the user with the given id and returns it as a public `User` if found.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use crate::database::SqliteDatabase;
  /// # async fn run() -> anyhow::Result<()> {
  /// let db = SqliteDatabase::new("db.sqlite").await?;
  /// let result = db.find_user_by_id("user-id").await?;
  /// if let Some(user) = result {
  ///     // use `user`
  /// }
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// # Returns
  ///
  /// `Some(User)` if a user with the given id exists, `None` otherwise.
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

  /// Creates a new user record in the database.
  ///
  /// Inserts a user with the provided `id`, `email`, `password_hash`, and `created_at` timestamp.
  /// Returns the created `User` with `email_verified` set to `false` and `email_verified_at` set to `None`.
  ///
  /// # Examples
  ///
  /// ```
  /// # async fn run_example(db: &crate::SqliteDatabase) -> Result<(), Box<dyn std::error::Error>> {
  /// let user = db.create_user("u1", "user@example.com", "s0m3h4sh", 1_700_000_000).await?;
  /// assert_eq!(user.id, "u1");
  /// assert_eq!(user.email, "user@example.com");
  /// assert!(!user.email_verified);
  /// # Ok(()) }
  /// ```
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

  /// Marks the specified user's email as verified and records when verification occurred.
  ///
  /// The `verified_at` value is the UNIX epoch timestamp (seconds) when the email was verified.
  ///
  /// # Arguments
  ///
  /// * `user_id` - The ID of the user whose email verification state will be set to true.
  /// * `verified_at` - UNIX epoch seconds representing when the email was verified.
  ///
  /// # Returns
  ///
  /// `Ok(())` on success, or an error if the update fails.
  ///
  /// # Examples
  ///
  /// ```
  /// # use your_crate::database::SqliteDatabase;
  /// # async fn example(db: &SqliteDatabase) -> Result<(), Box<dyn std::error::Error>> {
  /// db.update_email_verified("user-id-123", 1_700_000_000).await?;
  /// # Ok(())
  /// # }
  /// ```
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

  /// Inserts a new session record for a user with the given token and expiry.
  ///
  /// The session's `created_at` timestamp is set to the current UNIX epoch seconds at call time.
  ///
  /// # Parameters
  ///
  /// - `token`: the session token string to store.
  /// - `user_id`: the id of the user the session belongs to.
  /// - `expires_at`: expiration time as UNIX epoch seconds.
  ///
  /// # Returns
  ///
  /// `Ok(())` on success, or an error if the insert fails.
  ///
  /// # Examples
  ///
  /// ```
  /// # async fn run(db: &crate::SqliteDatabase) -> anyhow::Result<()> {
  /// db.create_session("tok123", "user-id-1", 1_700_000_000).await?;
  /// # Ok(())
  /// # }
  /// ```
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

  /// Fetches a session record matching the given token.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # async fn example(db: &crate::SqliteDatabase) -> crate::Result<()> {
  /// let token = "example-token";
  /// let session = db.find_session(token).await?;
  /// // `session` is `Some(DbSession)` if found, otherwise `None`
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// # Returns
  ///
  /// `Some(DbSession)` if a session with the specified token exists, `None` otherwise.
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

  /// Deletes the session record for the specified token.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # async fn example(db: &crate::SqliteDatabase) -> crate::Result<()> {
  /// db.delete_session("session-token-123").await?;
  /// # Ok(())
  /// # }
  /// ```
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

  /// Deletes sessions whose `expires_at` timestamp is earlier than the current UNIX epoch seconds.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # async fn example(db: &crate::SqliteDatabase) {
  /// let deleted = db.delete_expired_sessions().await.unwrap();
  /// println!("deleted {} expired sessions", deleted);
  /// # }
  /// ```
  ///
  /// # Returns
  ///
  /// `u64` the number of sessions deleted.
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

  /// Inserts a new token record for a user with the provided identifiers, hash, type, and timestamps.
  ///
  /// # Arguments
  ///
  /// * `id` — Token identifier (typically a UUID).
  /// * `user_id` — Identifier of the user who owns the token.
  /// * `token_hash` — Stored hash of the token value used for lookups/verification.
  /// * `token_type` — Semantic token category (for example, `"reset_password"` or `"email_verification"`).
  /// * `expires_at` — Expiration time as Unix epoch seconds.
  /// * `created_at` — Creation time as Unix epoch seconds.
  ///
  /// # Returns
  ///
  /// `()` on success.
  ///
  /// # Examples
  ///
  /// ```
  /// # async fn example(db: &crate::SqliteDatabase) -> anyhow::Result<()> {
  /// db.create_token(
  ///     "token-id",
  ///     "user-id",
  ///     "hashed-value",
  ///     "reset_password",
  ///     1_700_000_000,
  ///     1_690_000_000,
  /// ).await?;
  /// # Ok(())
  /// # }
  /// ```
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

  /// Finds a token record by its hashed value and type.
  ///
  /// Returns `Some(DbToken)` when a token with the given `token_hash` and `token_type` exists, `None` otherwise.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # async fn example(db: &crate::database::SqliteDatabase) -> anyhow::Result<()> {
  /// let maybe_token = db.find_token("some_hash", "password_reset").await?;
  /// if let Some(token) = maybe_token {
  ///     println!("found token id: {}", token.id);
  /// }
  /// # Ok(())
  /// # }
  /// ```
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

  /// Mark a token as used by setting its `used_at` timestamp in the database.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use std::error::Error;
  /// # async fn example(db: &crate::SqliteDatabase) -> Result<(), Box<dyn Error>> {
  /// db.mark_token_used("some_token_hash", 1_702_000_000).await?;
  /// # Ok(())
  /// # }
  /// ```
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

  /// Deletes the token record that matches the given token hash.
  ///
  /// # Examples
  ///
  /// ```
  /// # async fn example(db: &crate::SqliteDatabase) {
  /// db.delete_token("some_hash").await.unwrap();
  /// # }
  /// ```
  ///
  /// Returns `Ok(())` on success.
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

  /// Deletes all tokens whose `expires_at` timestamp is earlier than the current time.
  ///
  /// # Returns
  ///
  /// `u64` number of rows removed from the `tokens` table.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use crate::database::sqlite::SqliteDatabase;
  /// # tokio::runtime::Runtime::new().unwrap().block_on(async {
  /// let db = SqliteDatabase::new("test.db").await.unwrap();
  /// let deleted = db.delete_expired_tokens().await.unwrap();
  /// println!("deleted {}", deleted);
  /// # });
  /// ```
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