use super::{Token, TokenStrategy, TokenType, VerifiedToken};
use crate::database::DatabaseTrait;
use crate::error::{AuthError, Result};
use crate::security::tokens;
use async_trait::async_trait;
use std::sync::Arc;

pub struct DatabaseTokenStrategy {
	db: Arc<Box<dyn DatabaseTrait>>,
}

impl DatabaseTokenStrategy {
	pub fn new(db: Arc<Box<dyn DatabaseTrait>>) -> Self {
		Self { db }
	}

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
		user_id: &str,
		token_type: TokenType,
		expires_at: i64,
	) -> Result<Token> {
		todo!("Not implemented yet")
	}
	async fn verify_token(&self, token: &str, token_type: TokenType) -> Result<VerifiedToken> {
		todo!("Not implemented yet")
	}
	async fn mark_token_as_used(&self, token: &str) -> Result<()> {
		todo!("Not implemented yet")
	}
	async fn clean_expired_tokens(&self) -> Result<()> {
		todo!("Not implemented yet")
	}
}
