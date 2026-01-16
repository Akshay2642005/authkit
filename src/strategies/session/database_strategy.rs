use crate::database::models::DbSession;
use crate::database::DatabaseTrait;
use crate::error::Result;
use crate::strategies::session::SessionStrategy;
use async_trait::async_trait;

/// Database-backed session strategy
pub(crate) struct DatabaseSessionStrategy;

#[async_trait]
impl SessionStrategy for DatabaseSessionStrategy {
  /// Creates a session record in the backing database for the given token and user.
  ///
  /// `expires_at` is a Unix timestamp (seconds since epoch) when the session should expire.
  /// Returns `Ok(())` on success or an error propagated from the database on failure.
  ///
  /// # Examples
  ///
  /// ```
  /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
  /// // let db: impl DatabaseTrait = /* obtain database implementation */;
  /// let strategy = DatabaseSessionStrategy;
  /// strategy.create_session(&db, "session-token", "user-id", 1_700_000_000).await?;
  /// # Ok(())
  /// # }
  /// ```
  async fn create_session(
    &self,
    db: &dyn DatabaseTrait,
    token: &str,
    user_id: &str,
    expires_at: i64,
  ) -> Result<()> {
    db.create_session(token, user_id, expires_at).await
  }

  /// Looks up a session in the database using the given session token.
  ///
  /// Returns `Ok(Some(DbSession))` if a session with the token exists, `Ok(None)` if no session is found, or an `Err` if the database operation fails.
  ///
  /// # Examples
  ///
  /// ```
  /// # use crate::{DatabaseTrait, DatabaseSessionStrategy, DbSession};
  /// # async fn example(db: &dyn DatabaseTrait) {
  /// let strategy = DatabaseSessionStrategy;
  /// let token = "session-token-abc";
  /// let result = strategy.find_session(db, token).await.unwrap();
  /// // `result` is `Some(DbSession)` when found, or `None` when not.
  /// # }
  /// ```
  async fn find_session(&self, db: &dyn DatabaseTrait, token: &str) -> Result<Option<DbSession>> {
    db.find_session(token).await
  }

  /// Deletes the session identified by `token` from the provided database.
  ///
  /// Returns `Ok(())` if the session was deleted successfully, or an error returned by the database implementation otherwise.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use crate::{DatabaseSessionStrategy, DatabaseTrait};
  /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
  /// let strategy = DatabaseSessionStrategy;
  /// let db: &dyn DatabaseTrait = /* obtain database instance */ unimplemented!();
  /// strategy.delete_session(db, "session-token-123").await?;
  /// # Ok(()) }
  /// ```
  async fn delete_session(&self, db: &dyn DatabaseTrait, token: &str) -> Result<()> {
    db.delete_session(token).await
  }
}