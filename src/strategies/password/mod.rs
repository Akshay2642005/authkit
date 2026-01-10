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
#[cfg_attr(feature = "argon2", derive(Default))]
pub enum PasswordStrategyType {
	#[cfg(feature = "argon2")]
	#[cfg_attr(feature = "argon2", default)]
	Argon2,
}

impl PasswordStrategyType {
	pub(crate) fn create_strategy(self) -> Result<Box<dyn PasswordStrategy>> {
		match self {
			#[cfg(feature = "argon2")]
			Self::Argon2 => Ok(Box::new(argon2_strategy::Argon2Strategy::default())),
		}
	}
}

// Compile-time check: at least one password strategy must be enabled
#[cfg(not(any(feature = "argon2", feature = "bcrypt")))]
compile_error!(
	"AuthKit requires at least one password strategy. \
	 Enable one of: 'argon2' (recommended), 'bcrypt'. \
	 Example: cargo build --features argon2"
);
