#[cfg(feature = "argon2")]
use crate::error::{AuthError, Result};
use crate::strategies::password::PasswordStrategy;
use argon2::{
  password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
  Argon2,
};
use async_trait::async_trait;

/// Argon2id password hashing strategy
#[derive(Default)]
pub(crate) struct Argon2Strategy {
  argon2: Argon2<'static>,
}

#[async_trait]
impl PasswordStrategy for Argon2Strategy {
  /// Hashes a password using Argon2 with a securely generated salt.
  ///
  /// Returns the Argon2-encoded password hash string.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// // inside an async context
  /// let strategy = Argon2Strategy::default();
  /// let hash = strategy.hash_password("password123").await.unwrap();
  /// assert!(!hash.is_empty());
  /// ```
  async fn hash_password(&self, password: &str) -> Result<String> {
    // Generate salt
    let salt = SaltString::generate(&mut OsRng);

    // Hash password
    let password_hash = self
      .argon2
      .hash_password(password.as_bytes(), &salt)
      .map_err(|e| AuthError::PasswordHashingError(e.to_string()))?;

    Ok(password_hash.to_string())
  }

  /// Verifies whether a plaintext password matches a stored Argon2 hash.
  ///
  /// Returns `true` if `password` matches `hash`, `false` otherwise.
  /// Returns an `AuthError::PasswordHashingError` if the stored `hash` cannot be parsed.
  ///
  /// # Examples
  ///
  /// ```
  /// use futures::executor::block_on;
  /// let strategy = Argon2Strategy::default();
  /// let hash = block_on(strategy.hash_password("s3cret")).unwrap();
  /// let ok = block_on(strategy.verify_password("s3cret", &hash)).unwrap();
  /// assert!(ok);
  /// ```
  async fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
    // Parse stored hash
    let parsed_hash =
      PasswordHash::new(hash).map_err(|e| AuthError::PasswordHashingError(e.to_string()))?;

    // Verify password (timing-safe comparison built-in)
    match self
      .argon2
      .verify_password(password.as_bytes(), &parsed_hash)
    {
      Ok(()) => Ok(true),
      Err(_) => Ok(false),
    }
  }
}