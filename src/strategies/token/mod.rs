use crate::error::Result;
use async_trait::async_trait;

#[derive(Clone, Debug, PartialEq, Eq, Copy, Default)]
pub enum TokenStrategyType {
	#[default]
	Database,
}

impl TokenStrategyType {
	pub fn create_strategy(
		self,
		db: std::sync::Arc<Box<dyn crate::database::DatabaseTrait>>,
	) -> Box<dyn TokenStrategy> {
		match self {
			TokenStrategyType::Database => Box::new(database_strategy::DatabaseTokenStrategy::new(db)),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum TokenType {
	EmailVerification,
	// PasswordReset,
	// MagicLink,
}

impl TokenType {
	pub fn as_str(&self) -> &'static str {
		match self {
			TokenType::EmailVerification => "email_verification",
			// PasswordReset => "password_reset",
			// MagicLink => "magic_link",
		}
	}
}

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

#[derive(Clone, Debug)]
pub struct VerifiedToken {
	pub id: String,
	pub user_id: String,
	pub token_type: TokenType,
}

#[async_trait]
pub trait TokenStrategy: Send + Sync {
	async fn generate_token(
		&self,
		user_id: &str,
		token_type: TokenType,
		expires_at: i64,
	) -> Result<Token>;
	async fn verify_token(&self, token: &str, token_type: TokenType) -> Result<VerifiedToken>;
	async fn mark_token_as_used(&self, token: &str) -> Result<()>;
	async fn clean_expired_tokens(&self) -> Result<()>;
}
pub mod database_strategy;
