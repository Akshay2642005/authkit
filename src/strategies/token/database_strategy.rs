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
  /// Compute the SHA-256 hex digest of a token for secure storage.
  ///
  /// # Examples
  ///
  /// ```
  /// let hashed = hash_token("hello");
  /// assert_eq!(
  ///     hashed,
  ///     "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
  /// );
  /// ```
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
  /// Generates a new cryptographically secure token, stores its hashed form in the database, and returns the token record containing the plaintext token and metadata.
  ///
  /// # Parameters
  ///
  /// - `user_id`: Identifier of the user the token is issued for.
  /// - `token_type`: Type/category of the token (e.g., email verification).
  /// - `expires_in_seconds`: Lifetime of the token in seconds from creation; used to compute `expires_at`.
  ///
  /// # Returns
  ///
  /// A `Token` containing the token `id`, `user_id`, `token_hash`, plaintext `token`, `token_type`, `expires_at`, and `created_at`.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// // Example usage (requires a real `DatabaseTrait` implementation and async runtime)
  /// # async fn doc() {
  /// let db: Box<dyn DatabaseTrait> = /* your database implementation */;
  /// let strategy = DatabaseTokenStrategy;
  /// let token = strategy
  ///     .generate_token(&*db, "user-123", TokenType::EmailVerification, 3600)
  ///     .await
  ///     .unwrap();
  /// assert_eq!(token.user_id, "user-123");
  /// assert!(token.expires_at > token.created_at);
  /// # }
  /// ```
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

  /// Verifies a plaintext token against the database and returns its verified metadata.
  ///
  /// Computes the token hash, looks up a matching token record for the given type, ensures the token
  /// has not been used, and ensures it has not expired.
  ///
  /// # Examples
  ///
  /// ```
  /// // Requires an async context and a `db` implementing `DatabaseTrait`.
  /// // let strategy = DatabaseTokenStrategy;
  /// // let verified = strategy.verify_token(&db, "plaintext-token", TokenType::Email).await?;
  /// ```
  ///
  /// # Returns
  /// `VerifiedToken` with `id`, `user_id`, and `token_type` on success.
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

  /// Marks a plaintext token as used by recording the current UNIX timestamp for its hashed value in the database.
  ///
  /// # Examples
  ///
  /// ```
  /// # use crate::strategies::token::database_strategy::DatabaseTokenStrategy;
  /// # use crate::db::DatabaseTrait;
  /// # async fn run() {
  /// let strategy = DatabaseTokenStrategy;
  /// let db: &dyn DatabaseTrait = unimplemented!();
  /// let _ = strategy.mark_token_as_used(db, "plaintext-token").await;
  /// # }
  /// ```
  ///
  /// # Returns
  ///
  /// `Ok(())` on success, or an `Err` containing a database error if the operation fails.
  async fn mark_token_as_used(&self, db: &dyn DatabaseTrait, token: &str) -> Result<()> {
    let token_hash = Self::hash_token(token);
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;

    db.mark_token_used(&token_hash, now).await
  }

  /// Removes all tokens that have passed their expiration time from the database.
  ///
  /// # Returns
  ///
  /// `Ok(())` if expired tokens were successfully removed, or an error from the database layer otherwise.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// # async fn example(db: &dyn DatabaseTrait) -> crate::Result<()> {
  /// let strategy = DatabaseTokenStrategy;
  /// strategy.clean_expired_tokens(db).await?;
  /// # Ok(())
  /// # }
  /// ```
  async fn clean_expired_tokens(&self, db: &dyn DatabaseTrait) -> Result<()> {
    db.delete_expired_tokens().await?;
    Ok(())
  }
}