use crate::database::DatabaseTrait;
use crate::database::models::DbSession;
use crate::error::Result;
use crate::strategies::session::SessionStrategy;
use async_trait::async_trait;

/// Database-backed session strategy
pub(crate) struct DatabaseSessionStrategy;

#[async_trait]
impl SessionStrategy for DatabaseSessionStrategy {
	async fn create_session(
		&self,
		db: &dyn DatabaseTrait,
		token: &str,
		user_id: &str,
		expires_at: i64,
	) -> Result<()> {
		db.create_session(token, user_id, expires_at).await
	}

	async fn find_session(&self, db: &dyn DatabaseTrait, token: &str) -> Result<Option<DbSession>> {
		db.find_session(token).await
	}

	async fn delete_session(&self, db: &dyn DatabaseTrait, token: &str) -> Result<()> {
		db.delete_session(token).await
	}
}
