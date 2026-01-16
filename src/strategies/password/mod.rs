//! Password hashing strategies

#[cfg(feature = "argon2")]
pub mod argon2_strategy;

use crate::error::Result;
use async_trait::async_trait;

/// Password hashing strategy trait (internal)
#[async_trait]
pub(crate) trait PasswordStrategy: Send + Sync {
  /// Hash a password
  async fn hash_password(&self, password: &str) -> Result<String>;

  /// Verify a password against a hash (timing-safe)
  async fn verify_password(&self, password: &str, hash: &str) -> Result<bool>;
}

/// Public enum for selecting password strategy
///
/// **At least one password strategy feature must be enabled.**
///
/// Available strategies:
/// - `argon2` (recommended, enabled by default) - Argon2id password hashing
/// - `bcrypt` (not yet implemented) - bcrypt password hashing
///
/// # Examples
///
/// ```ignore
/// use authkit::Auth;
/// use authkit::strategies::password::PasswordStrategyType;
///
/// // Using default (argon2)
/// let auth = Auth::builder()
///     .database(db)
///     .build()?;
///
/// // Explicitly selecting argon2
/// let auth = Auth::builder()
///     .database(db)
///     .password_strategy(PasswordStrategyType::Argon2)
///     .build()?;
/// ```
#[derive(Debug, Clone, Copy)]
pub enum PasswordStrategyType {
  #[cfg(feature = "argon2")]
  Argon2,
  #[cfg(feature = "bcrypt")]
  Bcrypt,
}

// Compile-time check: at least one password strategy must be enabled
// This will fail compilation if neither argon2 nor bcrypt features are enabled
#[cfg(not(any(feature = "argon2", feature = "bcrypt")))]
compile_error!(
  "AuthKit requires at least one password hashing strategy feature to be enabled.\n\
	 \n\
	 Available strategies:\n\
	 - 'argon2' (recommended, secure default)\n\
	 - 'bcrypt' (not yet implemented)\n\
	 \n\
	 Add one to your Cargo.toml:\n\
	 \n\
	 [dependencies]\n\
	 authkit = { version = \"0.1\", features = [\"argon2\", \"sqlite\"] }\n\
	 \n\
	 Or use the defaults which include argon2:\n\
	 \n\
	 [dependencies]\n\
	 authkit = \"0.1\""
);

impl Default for PasswordStrategyType {
  /// Selects the default password hashing strategy based on enabled features.
  ///
  /// Returns the default `PasswordStrategyType`: `Argon2` when the `argon2` feature is enabled,
  /// otherwise `Bcrypt` (when `bcrypt` is enabled).
  ///
  /// # Examples
  ///
  /// ```
  /// // When compiled with the "argon2" feature
  /// # #[cfg(feature = "argon2")]
  /// # {
  /// use crate::PasswordStrategyType;
  /// let def = PasswordStrategyType::default();
  /// assert!(matches!(def, PasswordStrategyType::Argon2));
  /// # }
  /// ```
  ///
  /// ```
  /// // When compiled with the "bcrypt" feature but without "argon2"
  /// # #[cfg(all(not(feature = "argon2"), feature = "bcrypt"))]
  /// # {
  /// use crate::PasswordStrategyType;
  /// let def = PasswordStrategyType::default();
  /// assert!(matches!(def, PasswordStrategyType::Bcrypt));
  /// # }
  /// ```
  fn default() -> Self {
    // Prioritize argon2 (recommended)
    #[cfg(feature = "argon2")]
    return Self::Argon2;

    // Fall back to bcrypt if argon2 not available
    #[cfg(all(not(feature = "argon2"), feature = "bcrypt"))]
    return Self::Bcrypt;
  }
}

impl PasswordStrategyType {
  /// Create a concrete boxed password hashing strategy for this variant.
  ///
  /// Returns a boxed implementation of `PasswordStrategy` for the selected variant when available.
  /// If the chosen variant is not implemented (currently bcrypt), an `AuthError::InternalError` is returned.
  ///
  /// # Examples
  ///
  /// ```
  /// let strategy = crate::password::PasswordStrategyType::default().create_strategy().unwrap();
  /// // `strategy` is a `Box<dyn crate::password::PasswordStrategy>` and can be used to hash/verify passwords.
  /// ```
  pub(crate) fn create_strategy(self) -> Result<Box<dyn PasswordStrategy>> {
    match self {
      #[cfg(feature = "argon2")]
      Self::Argon2 => Ok(Box::new(argon2_strategy::Argon2Strategy::default())),
      #[cfg(feature = "bcrypt")]
      Self::Bcrypt => {
        // bcrypt strategy not yet implemented
        Err(crate::error::AuthError::InternalError(
          "bcrypt password strategy is not yet implemented".to_string(),
        ))
      }
    }
  }
}