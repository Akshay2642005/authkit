//! Test helpers for setting up database schemas in tests
//!
//! This module provides schema setup functions that are only available in test builds.
//! For production, use the CLI: `authkit migrate --db-url <URL>`
//!
//! The schemas here mirror what the CLI generates, including all features
//! to support comprehensive testing.

use crate::error::Result;
use crate::types::Database;

/// Set up the test database schema for SQLite
///
/// Creates all tables including email_verification columns for comprehensive testing.
#[cfg(feature = "sqlite")]
pub(crate) async fn setup_sqlite_schema(db: &Database) -> Result<()> {
  use sqlx::Executor;

  let pool = match &db.inner {
    crate::types::DatabaseInner::Sqlite(sqlite_db) => &sqlite_db.pool,
    #[cfg(feature = "postgres")]
    _ => panic!("Expected SQLite database"),
  };

  // Users table with email_verification columns
  pool
    .execute(
      r#"
      CREATE TABLE IF NOT EXISTS users (
        id TEXT PRIMARY KEY,
        email TEXT NOT NULL UNIQUE,
        name TEXT,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        email_verified INTEGER NOT NULL DEFAULT 0,
        email_verified_at INTEGER
      )
      "#,
    )
    .await?;

  // Accounts table
  pool
    .execute(
      r#"
      CREATE TABLE IF NOT EXISTS accounts (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        provider TEXT NOT NULL,
        provider_account_id TEXT NOT NULL,
        password_hash TEXT,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        UNIQUE(provider, provider_account_id)
      )
      "#,
    )
    .await?;

  // Sessions table
  pool
    .execute(
      r#"
      CREATE TABLE IF NOT EXISTS sessions (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        token TEXT NOT NULL UNIQUE,
        expires_at INTEGER NOT NULL,
        created_at INTEGER NOT NULL,
        ip_address TEXT,
        user_agent TEXT
      )
      "#,
    )
    .await?;

  // Verification table
  pool
    .execute(
      r#"
      CREATE TABLE IF NOT EXISTS verification (
        id TEXT PRIMARY KEY,
        user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
        identifier TEXT NOT NULL,
        token_hash TEXT NOT NULL UNIQUE,
        token_type TEXT NOT NULL,
        expires_at INTEGER NOT NULL,
        created_at INTEGER NOT NULL,
        used_at INTEGER
      )
      "#,
    )
    .await?;

  Ok(())
}

/// Set up the test database schema for PostgreSQL
///
/// Creates all tables including email_verification columns for comprehensive testing.
#[cfg(feature = "postgres")]
pub(crate) async fn setup_postgres_schema(db: &Database) -> Result<()> {
  use sqlx::Executor;

  let pool = match &db.inner {
    crate::types::DatabaseInner::Postgres(postgres_db) => &postgres_db.pool,
    #[cfg(feature = "sqlite")]
    _ => panic!("Expected PostgreSQL database"),
  };

  // Users table with email_verification columns
  pool
    .execute(
      r#"
      CREATE TABLE IF NOT EXISTS users (
        id TEXT PRIMARY KEY,
        email TEXT NOT NULL UNIQUE,
        name TEXT,
        created_at BIGINT NOT NULL,
        updated_at BIGINT NOT NULL,
        email_verified BOOLEAN NOT NULL DEFAULT FALSE,
        email_verified_at BIGINT
      )
      "#,
    )
    .await?;

  // Accounts table
  pool
    .execute(
      r#"
      CREATE TABLE IF NOT EXISTS accounts (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        provider TEXT NOT NULL,
        provider_account_id TEXT NOT NULL,
        password_hash TEXT,
        created_at BIGINT NOT NULL,
        updated_at BIGINT NOT NULL,
        UNIQUE(provider, provider_account_id)
      )
      "#,
    )
    .await?;

  // Sessions table
  pool
    .execute(
      r#"
      CREATE TABLE IF NOT EXISTS sessions (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
        token TEXT NOT NULL UNIQUE,
        expires_at BIGINT NOT NULL,
        created_at BIGINT NOT NULL,
        ip_address TEXT,
        user_agent TEXT
      )
      "#,
    )
    .await?;

  // Verification table
  pool
    .execute(
      r#"
      CREATE TABLE IF NOT EXISTS verification (
        id TEXT PRIMARY KEY,
        user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
        identifier TEXT NOT NULL,
        token_hash TEXT NOT NULL UNIQUE,
        token_type TEXT NOT NULL,
        expires_at BIGINT NOT NULL,
        created_at BIGINT NOT NULL,
        used_at BIGINT
      )
      "#,
    )
    .await?;

  Ok(())
}

/// Set up the test database schema based on the database type
///
/// This function detects the database type and calls the appropriate setup function.
pub(crate) async fn setup_test_schema(db: &Database) -> Result<()> {
  match &db.inner {
    #[cfg(feature = "sqlite")]
    crate::types::DatabaseInner::Sqlite(_) => setup_sqlite_schema(db).await,
    #[cfg(feature = "postgres")]
    crate::types::DatabaseInner::Postgres(_) => setup_postgres_schema(db).await,
  }
}
