pub mod models;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

use crate::error::Result;
use crate::types::{DatabaseInner, User};
use async_trait::async_trait;
use models::{DbAccount, DbSession, DbUser, DbUserWithAccount, DbVerification};

/// Core database trait for AuthKit
///
/// This trait abstracts database operations across different backends (SQLite, Postgres).
/// The schema follows the feature-based approach:
/// - Base (email_password): users, accounts, sessions, verification tables
/// - Email verification: adds email_verified columns to users
#[async_trait]
pub(crate) trait DatabaseTrait: Send + Sync {
  // ==========================================
  // User Operations
  // ==========================================

  /// Find a user by their email address
  async fn find_user_by_email(&self, email: &str) -> Result<Option<DbUser>>;

  /// Find a user by their unique ID
  async fn find_user_by_id(&self, id: &str) -> Result<Option<User>>;

  /// Create a new user
  async fn create_user(
    &self,
    id: &str,
    email: &str,
    name: Option<&str>,
    created_at: i64,
  ) -> Result<User>;

  // ==========================================
  // Email Verification Operations
  // (Requires email_verification feature migration)
  // ==========================================

  /// Update a user's email verification status
  /// Requires: email_verification feature columns (email_verified, email_verified_at)
  async fn update_email_verified(&self, user_id: &str, verified_at: i64) -> Result<()>;

  /// Find a user by ID with email verification status
  /// Requires: email_verification feature columns (email_verified, email_verified_at)
  async fn find_user_by_id_with_verification(&self, id: &str) -> Result<Option<User>>;

  /// Find a user by email with email verification status
  /// Requires: email_verification feature columns (email_verified, email_verified_at)
  async fn find_user_by_email_with_verification(&self, email: &str) -> Result<Option<DbUser>>;

  // ==========================================
  // Account Operations
  // ==========================================

  /// Create an account (links a provider to a user)
  #[allow(dead_code)]
  async fn create_account(
    &self,
    id: &str,
    user_id: &str,
    provider: &str,
    provider_account_id: &str,
    password_hash: Option<&str>,
    created_at: i64,
  ) -> Result<()>;

  /// Find an account by provider and provider account ID
  #[allow(dead_code)]
  async fn find_account_by_provider(
    &self,
    provider: &str,
    provider_account_id: &str,
  ) -> Result<Option<DbAccount>>;

  /// Find user with their credential account (for email/password login)
  async fn find_user_with_credential_account(
    &self,
    email: &str,
  ) -> Result<Option<DbUserWithAccount>>;

  /// Find user with their credential account including email verification status
  /// Requires: email_verification feature columns (email_verified, email_verified_at)
  async fn find_user_with_credential_account_with_verification(
    &self,
    email: &str,
  ) -> Result<Option<DbUserWithAccount>>;

  // ==========================================
  // Session Operations
  // ==========================================

  /// Create a new session for a user
  async fn create_session(
    &self,
    id: &str,
    token: &str,
    user_id: &str,
    expires_at: i64,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
  ) -> Result<()>;

  /// Find a session by its token
  async fn find_session(&self, token: &str) -> Result<Option<DbSession>>;

  /// Delete a specific session
  async fn delete_session(&self, token: &str) -> Result<()>;

  /// Delete all expired sessions (cleanup utility)
  #[allow(dead_code)]
  async fn delete_expired_sessions(&self) -> Result<u64>;

  // ==========================================
  // Verification Token Operations
  // ==========================================

  /// Create a new verification token
  #[allow(dead_code)]
  async fn create_verification(
    &self,
    id: &str,
    user_id: Option<&str>,
    identifier: &str,
    token_hash: &str,
    token_type: &str,
    expires_at: i64,
    created_at: i64,
  ) -> Result<()>;

  /// Find a verification token by its hash and type
  #[allow(dead_code)]
  async fn find_verification(
    &self,
    token_hash: &str,
    token_type: &str,
  ) -> Result<Option<DbVerification>>;

  /// Mark a verification token as used
  #[allow(dead_code)]
  async fn mark_verification_used(&self, token_hash: &str, used_at: i64) -> Result<()>;

  /// Delete a specific verification token by its hash
  #[allow(dead_code)]
  async fn delete_verification(&self, token_hash: &str) -> Result<()>;

  /// Delete all expired verification tokens (cleanup utility)
  #[allow(dead_code)]
  async fn delete_expired_verifications(&self) -> Result<u64>;
}

pub(crate) fn create_database_trait(inner: DatabaseInner) -> Box<dyn DatabaseTrait> {
  match inner {
    #[cfg(feature = "sqlite")]
    DatabaseInner::Sqlite(db) => Box::new(db),
    #[cfg(feature = "postgres")]
    DatabaseInner::Postgres(db) => Box::new(db),
  }
}

// Compile-time check: at least one database backend must be enabled
#[cfg(not(any(feature = "sqlite", feature = "postgres")))]
compile_error!(
  "AuthKit requires at least one database backend feature to be enabled.\n\
	 \n\
	 Available backends:\n\
	 - 'sqlite' (enabled by default)\n\
	 - 'postgres'\n\
	 \n\
	 Add one to your Cargo.toml:\n\
	 \n\
	 [dependencies]\n\
	 authkit = { version = \"0.1\", features = [\"sqlite\", \"argon2\"] }\n\
	 \n\
	 Or use the defaults which include sqlite:\n\
	 \n\
	 [dependencies]\n\
	 authkit = \"0.1\""
);
