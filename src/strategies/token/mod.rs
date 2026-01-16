use crate::error::Result;
use async_trait::async_trait;

#[derive(Clone, Debug, PartialEq, Eq, Copy, Default)]
pub enum TokenStrategyType {
  #[default]
  Database,
}

impl TokenStrategyType {
  /// Create a token strategy instance for this strategy type.
  ///
  /// # Returns
  ///
  /// A boxed `TokenStrategy` implementation corresponding to `self`.
  ///
  /// # Examples
  ///
  /// ```
  /// let _ = crate::strategies::token::TokenStrategyType::Database.create_strategy();
  /// ```
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
  // PasswordReset,
  // MagicLink,
}

impl TokenType {
  /// Returns the token type as a static string identifier.
  ///
  /// # Examples
  ///
  /// ```
  /// let s = TokenType::EmailVerification.as_str();
  /// assert_eq!(s, "email_verification");
  /// ```
  #[allow(dead_code)]
  pub fn as_str(&self) -> &'static str {
    match self {
      TokenType::EmailVerification => "email_verification",
      // PasswordReset => "password_reset",
      // MagicLink => "magic_link",
    }
  }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Token {
  pub id: String,
  pub token: String,
  pub token_hash: String,
  pub user_id: String,
  pub token_type: TokenType,
  pub expires_at: i64,
  pub created_at: i64,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct VerifiedToken {
  pub id: String,
  pub user_id: String,
  pub token_type: TokenType,
}

#[async_trait]
#[allow(dead_code)]
pub(crate) trait TokenStrategy: Send + Sync {
  async fn generate_token(
    &self,
    db: &dyn crate::database::DatabaseTrait,
    user_id: &str,
    token_type: TokenType,
    expires_at: i64,
  ) -> Result<Token>;
  async fn verify_token(
    &self,
    db: &dyn crate::database::DatabaseTrait,
    token: &str,
    token_type: TokenType,
  ) -> Result<VerifiedToken>;
  async fn mark_token_as_used(
    &self,
    db: &dyn crate::database::DatabaseTrait,
    token: &str,
  ) -> Result<()>;
  async fn clean_expired_tokens(&self, db: &dyn crate::database::DatabaseTrait) -> Result<()>;
}
pub mod database_strategy;