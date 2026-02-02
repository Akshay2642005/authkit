use serde::{Deserialize, Serialize};

/// Database model for users table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DbUser {
  pub id: String,
  pub email: String,
  pub name: Option<String>,
  pub created_at: i64,
  pub updated_at: i64,
  /// Email verification status - only present if email_verification feature is enabled
  pub email_verified: Option<bool>,
  pub email_verified_at: Option<i64>,
}

/// Database model for accounts table
/// Links authentication providers to users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DbAccount {
  pub id: String,
  pub user_id: String,
  /// Provider type: "credential", "google", "github", etc.
  pub provider: String,
  /// Provider-specific account ID (email for credential, OAuth ID for social)
  pub provider_account_id: String,
  /// Password hash - only set for "credential" provider
  pub password_hash: Option<String>,
  pub created_at: i64,
  pub updated_at: i64,
}

/// Database model for sessions table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DbSession {
  pub id: String,
  pub user_id: String,
  pub token: String,
  pub expires_at: i64,
  pub created_at: i64,
  pub ip_address: Option<String>,
  pub user_agent: Option<String>,
}

/// Database model for verification table (tokens for password reset, magic links, etc.)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DbVerification {
  pub id: String,
  pub user_id: Option<String>,
  /// Identifier for the verification (usually email)
  pub identifier: String,
  pub token_hash: String,
  /// Token type: "email_verification", "password_reset", "magic_link", etc.
  pub token_type: String,
  pub expires_at: i64,
  pub created_at: i64,
  pub used_at: Option<i64>,
}

impl From<DbUser> for crate::types::User {
  fn from(db_user: DbUser) -> Self {
    crate::types::User {
      id: db_user.id,
      email: db_user.email,
      name: db_user.name,
      email_verified: db_user.email_verified.unwrap_or(false),
      email_verified_at: db_user.email_verified_at,
      created_at: db_user.created_at,
      updated_at: db_user.updated_at,
    }
  }
}

/// Helper struct for user with account info (for login operations)
#[derive(Debug, Clone)]
pub(crate) struct DbUserWithAccount {
  pub user: DbUser,
  pub account: DbAccount,
}

impl DbUserWithAccount {
  pub fn password_hash(&self) -> Option<&str> {
    self.account.password_hash.as_deref()
  }
}
