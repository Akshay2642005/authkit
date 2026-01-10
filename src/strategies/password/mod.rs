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
#[derive(Debug, Clone, Copy)]
pub enum PasswordStrategyType {
	#[cfg(feature = "argon2")]
	Argon2,
}

impl Default for PasswordStrategyType {
	fn default() -> Self {
		#[cfg(feature = "argon2")]
		{
			Self::Argon2
		}
		#[cfg(not(feature = "argon2"))]
		{
			compile_error!("At least one password hashing strategy must be enabled")
		}
	}
}

impl PasswordStrategyType {
	pub(crate) fn create_strategy(self) -> Result<Box<dyn PasswordStrategy>> {
		match self {
			#[cfg(feature = "argon2")]
			Self::Argon2 => Ok(Box::new(argon2_strategy::Argon2Strategy::default())),
		}
	}
}
