#[cfg(feature = "argon2")]
use crate::error::{AuthError, Result};
use crate::strategies::password::PasswordStrategy;
use argon2::{
	Argon2,
	password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use async_trait::async_trait;

/// Argon2id password hashing strategy
pub(crate) struct Argon2Strategy {
	argon2: Argon2<'static>,
}

impl Default for Argon2Strategy {
	fn default() -> Self {
		// Use recommended defaults
		Self {
			argon2: Argon2::default(),
		}
	}
}

#[async_trait]
impl PasswordStrategy for Argon2Strategy {
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
