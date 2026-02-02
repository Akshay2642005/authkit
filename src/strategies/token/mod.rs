use crate::error::Result;
use async_trait::async_trait;

#[derive(Clone, Debug, PartialEq, Eq, Copy, Default)]
pub enum TokenStrategyType {
  #[default]
  Database,
}

impl TokenStrategyType {
  pub(crate) fn create_strategy(self) -> Box<dyn TokenStrategy> {
    match self {
      TokenStrategyType::Database => Box::new(database_strategy::DatabaseTokenStrategy),
    }
  }
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum TokenType {
  EmailVerification,
  PasswordReset,
  MagicLink,
}

impl TokenType {
  #[allow(dead_code)]
  pub fn as_str(&self) -> &'static str {
    match self {
      TokenType::EmailVerification => "email_verification",
      TokenType::PasswordReset => "password_reset",
      TokenType::MagicLink => "magic_link",
    }
  }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Token {
  pub id: String,
  pub token: String,
  pub token_hash: String,
  pub user_id: Option<String>,
  /// Identifier for the token (usually email)
  pub identifier: String,
  pub token_type: TokenType,
  pub expires_at: i64,
  pub created_at: i64,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct VerifiedToken {
  pub id: String,
  pub user_id: Option<String>,
  pub identifier: String,
  pub token_type: TokenType,
}

#[async_trait]
#[allow(dead_code)]
pub(crate) trait TokenStrategy: Send + Sync {
  /// Generate a new verification token
  async fn generate_token(
    &self,
    db: &dyn crate::database::DatabaseTrait,
    user_id: &str,
    identifier: &str,
    token_type: TokenType,
    expires_in_seconds: i64,
  ) -> Result<Token>;

  /// Verify a token and return the verified token info
  async fn verify_token(
    &self,
    db: &dyn crate::database::DatabaseTrait,
    token: &str,
    token_type: TokenType,
  ) -> Result<VerifiedToken>;

  /// Mark a token as used (so it can't be reused)
  async fn mark_token_as_used(
    &self,
    db: &dyn crate::database::DatabaseTrait,
    token: &str,
  ) -> Result<()>;

  /// Clean up expired tokens from the database
  async fn clean_expired_tokens(&self, db: &dyn crate::database::DatabaseTrait) -> Result<()>;
}

pub mod database_strategy;
