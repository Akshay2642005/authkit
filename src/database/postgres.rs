#[cfg(feature = "postgres")]
use crate::database::models::{DbSession, DbToken, DbUser};
use crate::database::DatabaseTrait;
use crate::error::Result;
use crate::types::User;
use async_trait::async_trait;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::Row;
use std::str::FromStr;

#[derive(Clone)]
pub struct PostgresDatabase {
  pool: PgPool,
}

impl PostgresDatabase {
  /// Create a PostgresDatabase by connecting to the provided PostgreSQL URL.
  ///
  /// The function parses the given connection URL into Postgres connection options and
  /// establishes a connection pool configured with a maximum of 5 connections.
  ///
  /// # Parameters
  ///
  /// * `url` - A PostgreSQL connection string (e.g. `postgres://user:pass@host/db`).
  ///
  /// # Returns
  ///
  /// A `Result` containing the newly created `PostgresDatabase` on success.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # async fn example() -> anyhow::Result<()> {
  /// let db = crate::database::postgres::PostgresDatabase::new("postgres://user:pass@localhost/db").await?;
  /// // use `db`...
  /// # Ok(())
  /// # }
  /// ```
  pub async fn new(url: &str) -> Result<Self> {
    let options = PgConnectOptions::from_str(url)?;

    let pool = PgPoolOptions::new()
      .max_connections(5)
      .connect_with(options)
      .await?;

    Ok(Self { pool })
  }
}

#[async_trait]
impl DatabaseTrait for PostgresDatabase {
  /// Ensures the required PostgreSQL schema exists for users, sessions, and tokens.
  ///
  /// Creates the `users`, `sessions`, and unified `tokens` tables if they do not exist,
  /// and creates indexes used for query performance. Returns an error if any SQL or
  /// connection operation fails.
  ///
  /// # Returns
  ///
  /// `Ok(())` on success, or an error if table/index creation fails.
  ///
  /// # Examples
  ///
  /// ```
  /// # use crate::database::postgres::PostgresDatabase;
  /// # tokio_test::block_on(async {
  /// let db = PostgresDatabase::new("postgres://user:pass@localhost/db").await.unwrap();
  /// db.migrate().await.unwrap();
  /// # });
  /// ```
  async fn migrate(&self) -> Result<()> {
    // Users table
    sqlx::query(
      r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at BIGINT NOT NULL,
                email_verified BOOLEAN NOT NULL DEFAULT FALSE,
                email_verified_at BIGINT
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
                expires_at BIGINT NOT NULL,
                created_at BIGINT NOT NULL,
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
                expires_at BIGINT NOT NULL,
                created_at BIGINT NOT NULL,
                used_at BIGINT,
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

  /// Fetches a user record that matches the given email.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
  /// let db = /* obtain PostgresDatabase instance */;
  /// let user = db.find_user_by_email("alice@example.com").await?;
  /// // `user` is `Some(DbUser)` if a row exists for that email, otherwise `None`.
  /// # Ok(()) }
  /// ```
  ///
  /// # Returns
  ///
  /// `Some(DbUser)` with the matching user if found, `None` otherwise.
  async fn find_user_by_email(&self, email: &str) -> Result<Option<DbUser>> {
    let user = sqlx::query(
      r#"
            SELECT id, email, password_hash, created_at, email_verified, email_verified_at
            FROM users
            WHERE email = $1
            "#,
    )
    .bind(email)
    .map(|row: sqlx::postgres::PgRow| DbUser {
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

  /// Fetches a user by their id from the database.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// // Async context required (e.g. #[tokio::test])
  /// let db = PostgresDatabase::new("postgres://user:pass@localhost/db").await.unwrap();
  /// let user_opt = db.find_user_by_id("user-id-123").await.unwrap();
  /// if let Some(user) = user_opt {
  ///     assert_eq!(user.id, "user-id-123");
  /// }
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
            WHERE id = $1
            "#,
    )
    .bind(id)
    .map(|row: sqlx::postgres::PgRow| DbUser {
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

  /// Creates a new user record and returns the corresponding `User`.
  ///
  /// Inserts a row into the `users` table with the provided `id`, `email`,
  /// `password_hash`, and `created_at`. The returned `User` has `email_verified`
  /// set to `false` and `email_verified_at` set to `None`.
  ///
  /// # Examples
  ///
  /// ```
  /// # async fn run_example(db: &crate::database::postgres::PostgresDatabase) -> crate::Result<()> {
  /// let user = db.create_user("user-id", "user@example.com", "hashed_pw", 1_700_000_000).await?;
  /// assert_eq!(user.id, "user-id");
  /// assert_eq!(user.email, "user@example.com");
  /// assert!(!user.email_verified);
  /// assert!(user.email_verified_at.is_none());
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// # Returns
  ///
  /// The created `User` with the provided `id`, `email`, and `created_at`; `email_verified` is `false` and `email_verified_at` is `None`.
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
            VALUES ($1, $2, $3, $4)
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

  /// Marks a user's email as verified at the given UNIX timestamp.
  ///
  /// Updates the user's record to set `email_verified` to true and `email_verified_at` to `verified_at`.
  ///
  /// # Parameters
  ///
  /// - `user_id`: Identifier of the user whose email verification status will be updated.
  /// - `verified_at`: UNIX epoch seconds when the email was verified.
  ///
  /// # Returns
  ///
  /// `Ok(())` on success, or an error if the database operation fails.
  ///
  /// # Examples
  ///
  /// ```
  /// #[tokio::test]
  /// async fn example_update_email_verified() {
  ///     // `db` should be an initialized PostgresDatabase
  ///     let db = /* obtain PostgresDatabase instance */;
  ///     db.update_email_verified("user-id", 1_640_000_000).await.unwrap();
  /// }
  /// ```
  async fn update_email_verified(&self, user_id: &str, verified_at: i64) -> Result<()> {
    sqlx::query(
      r#"
            UPDATE users
            SET email_verified = TRUE, email_verified_at = $1
            WHERE id = $2
            "#,
    )
    .bind(verified_at)
    .bind(user_id)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  /// Inserts a new session for the specified user with the given expiration time.
  ///
  /// The session's `created_at` is set to the current Unix epoch seconds at call time.
  ///
  /// # Examples
  ///
  /// ```
  /// # use crate::database::PostgresDatabase;
  /// # async fn docs_example(db: &PostgresDatabase) {
  /// db.create_session("session-token", "user-id", 1_700_000_000).await.unwrap();
  /// # }
  /// ```
  ///
  /// # Returns
  ///
  /// `Ok(())` on success, or an error if inserting the session fails.
  async fn create_session(&self, token: &str, user_id: &str, expires_at: i64) -> Result<()> {
    let created_at = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;

    sqlx::query(
      r#"
            INSERT INTO sessions (token, user_id, expires_at, created_at)
            VALUES ($1, $2, $3, $4)
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

  /// Fetches a session record matching the provided token.
  ///
  /// # Examples
  ///
  /// ```
  /// # async fn run_example(db: &crate::database::PostgresDatabase) -> anyhow::Result<()> {
  /// let token = "example-token";
  /// if let Some(session) = db.find_session(token).await? {
  ///     println!("Found session for user {}", session.user_id);
  /// }
  /// # Ok(()) }
  /// ```
  ///
  /// # Returns
  ///
  /// `Some(DbSession)` if a session with the given token exists, `None` otherwise.
  async fn find_session(&self, token: &str) -> Result<Option<DbSession>> {
    let session = sqlx::query(
      r#"
            SELECT token, user_id, expires_at, created_at
            FROM sessions
            WHERE token = $1
            "#,
    )
    .bind(token)
    .map(|row: sqlx::postgres::PgRow| DbSession {
      token: row.get("token"),
      user_id: row.get("user_id"),
      expires_at: row.get("expires_at"),
      created_at: row.get("created_at"),
    })
    .fetch_optional(&self.pool)
    .await?;

    Ok(session)
  }

  /// Removes the session row identified by `token` from the `sessions` table.
  ///
  /// # Examples
  ///
  /// ```
  /// # async fn run(db: &PostgresDatabase) -> crate::Result<()> {
  /// db.delete_session("session-token").await?;
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// # Returns
  /// `()` on success.
  async fn delete_session(&self, token: &str) -> Result<()> {
    sqlx::query(
      r#"
            DELETE FROM sessions
            WHERE token = $1
            "#,
    )
    .bind(token)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  /// Removes all sessions whose `expires_at` timestamp is earlier than the current time.
  ///
  /// Returns the number of session rows that were deleted.
  ///
  /// # Examples
  ///
  /// ```
  /// // Given a `db: PostgresDatabase` connected to a test database:
  /// // let removed = db.delete_expired_sessions().await?;
  /// // assert!(removed >= 0);
  /// ```
  async fn delete_expired_sessions(&self) -> Result<u64> {
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;

    let result = sqlx::query(
      r#"
            DELETE FROM sessions
            WHERE expires_at < $1
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

  /// Inserts a new token record for a user into the tokens table.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use crate::database::PostgresDatabase;
  /// # async fn example(db: &PostgresDatabase) {
  /// db.create_token(
  ///     "token-id",
  ///     "user-id",
  ///     "hashed-token",
  ///     "reset_password",
  ///     1_700_000_000, // expires_at (unix seconds)
  ///     1_700_000_000, // created_at (unix seconds)
  /// ).await.unwrap();
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
            VALUES ($1, $2, $3, $4, $5, $6)
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

  /// Finds a token record matching the given hash and type.
  ///
  /// Returns `Some(DbToken)` if a token with the specified `token_hash` and `token_type` exists, `None` otherwise.
  ///
  /// # Examples
  ///
  /// ```
  /// // async context required
  /// # async fn example(db: &crate::database::postgres::PostgresDatabase) -> Result<(), Box<dyn std::error::Error>> {
  /// let result = db.find_token("some_hash", "reset_password").await?;
  /// if let Some(token) = result {
  ///     println!("Found token id {}", token.id);
  /// }
  /// # Ok(()) }
  /// ```
  async fn find_token(&self, token_hash: &str, token_type: &str) -> Result<Option<DbToken>> {
    let token = sqlx::query(
      r#"
            SELECT id, user_id, token_hash, token_type, expires_at, created_at, used_at
            FROM tokens
            WHERE token_hash = $1 AND token_type = $2
            "#,
    )
    .bind(token_hash)
    .bind(token_type)
    .map(|row: sqlx::postgres::PgRow| DbToken {
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

  /// Marks a token as used by setting its `used_at` timestamp.
  ///
  /// # Arguments
  ///
  /// * `token_hash` - The hash that identifies the token to mark as used.
  /// * `used_at` - The time the token was used, expressed as UNIX epoch seconds.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// #[tokio::test]
  /// async fn mark_token_used_example() {
  ///     // assume `db` is an initialized PostgresDatabase
  ///     let db: crate::database::PostgresDatabase = unimplemented!();
  ///     db.mark_token_used("some_token_hash", 1_700_000_000).await.unwrap();
  /// }
  /// ```
  async fn mark_token_used(&self, token_hash: &str, used_at: i64) -> Result<()> {
    sqlx::query(
      r#"
            UPDATE tokens
            SET used_at = $1
            WHERE token_hash = $2
            "#,
    )
    .bind(used_at)
    .bind(token_hash)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  /// Deletes the token record that matches the provided token hash.
  ///
  /// Removes any row in the `tokens` table whose `token_hash` equals `token_hash`.
  ///
  /// # Errors
  ///
  /// Returns an error if the database operation fails.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// // `db` is a PostgresDatabase instance
  /// async fn example(db: &PostgresDatabase) -> anyhow::Result<()> {
  ///     db.delete_token("some_token_hash").await?;
  ///     Ok(())
  /// }
  /// ```
  async fn delete_token(&self, token_hash: &str) -> Result<()> {
    sqlx::query(
      r#"
            DELETE FROM tokens
            WHERE token_hash = $1
            "#,
    )
    .bind(token_hash)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  /// Removes all tokens whose `expires_at` timestamp is earlier than the current UNIX epoch seconds.
  ///
  /// # Returns
  ///
  /// The number of token rows that were deleted.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// # use crate::database::postgres::PostgresDatabase;
  /// # async fn example() -> anyhow::Result<()> {
  /// // let db = PostgresDatabase::new("postgres://user:pass@localhost/db").await?;
  /// // let deleted = db.delete_expired_tokens().await?;
  /// // assert!(deleted >= 0);
  /// # Ok(())
  /// # }
  /// ```
  async fn delete_expired_tokens(&self) -> Result<u64> {
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;

    let result = sqlx::query(
      r#"
            DELETE FROM tokens
            WHERE expires_at < $1
            "#,
    )
    .bind(now)
    .execute(&self.pool)
    .await?;

    Ok(result.rows_affected())
  }
}