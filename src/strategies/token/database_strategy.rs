use super::{Token, TokenStrategy, TokenType, VerifiedToken};
use crate::database::DatabaseTrait;
use crate::error::{AuthError, Result};
use crate::security::tokens;
use async_trait::async_trait;

/// Database-backed token strategy
///
/// This strategy stores tokens in the database and handles:
/// - Email verification tokens
/// - Password reset tokens (future)
/// - Magic link tokens (future)
pub(crate) struct DatabaseTokenStrategy;

impl DatabaseTokenStrategy {
  /// Hash a token using SHA-256 for secure storage
  #[allow(dead_code)]
  fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
  }
}

#[async_trait]
impl TokenStrategy for DatabaseTokenStrategy {
  async fn generate_token(
    &self,
    db: &dyn DatabaseTrait,
    user_id: &str,
    token_type: TokenType,
    expires_in_seconds: i64,
  ) -> Result<Token> {
    // Generate cryptographically secure random token
    let token = tokens::generate_token();
    let token_hash = Self::hash_token(&token);
    let id = tokens::generate_id();

    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;

    let expires_at = now + expires_in_seconds;

    // Store token in database
    db.create_token(
      &id,
      user_id,
      &token_hash,
      token_type.as_str(),
      expires_at,
      now,
    )
    .await?;

    Ok(Token {
      id,
      user_id: user_id.to_string(),
      token_hash,
      token,
      token_type,
      expires_at,
      created_at: now,
    })
  }

  async fn verify_token(
    &self,
    db: &dyn DatabaseTrait,
    token: &str,
    token_type: TokenType,
  ) -> Result<VerifiedToken> {
    let token_hash = Self::hash_token(token);

    // Find token in database
    let db_token = db
      .find_token(&token_hash, token_type.as_str())
      .await?
      .ok_or_else(|| AuthError::InvalidToken("Token not found or invalid".to_string()))?;

    // Check if token has already been used
    if db_token.used_at.is_some() {
      return Err(AuthError::TokenAlreadyUsed(
        "This token has already been used".to_string(),
      ));
    }

    // Check if token has expired
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;

    if db_token.expires_at < now {
      return Err(AuthError::TokenExpired("Token has expired".to_string()));
    }

    Ok(VerifiedToken {
      id: db_token.id,
      user_id: db_token.user_id,
      token_type,
    })
  }

  async fn mark_token_as_used(&self, db: &dyn DatabaseTrait, token: &str) -> Result<()> {
    let token_hash = Self::hash_token(token);
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;

    db.mark_token_used(&token_hash, now).await
  }

  async fn clean_expired_tokens(&self, db: &dyn DatabaseTrait) -> Result<()> {
    db.delete_expired_tokens().await?;
    Ok(())
  }
}
