//! Session management strategies

pub mod database_strategy;

use crate::database::models::DbSession;
use crate::database::DatabaseTrait;
use crate::error::Result;
use async_trait::async_trait;

/// Session management strategy trait (internal)
#[async_trait]
pub(crate) trait SessionStrategy: Send + Sync {
  /// Create a new session
  async fn create_session(
    &self,
    db: &dyn DatabaseTrait,
    token: &str,
    user_id: &str,
    expires_at: i64,
  ) -> Result<()>;

  /// Find a session by token
  async fn find_session(&self, db: &dyn DatabaseTrait, token: &str) -> Result<Option<DbSession>>;

  /// Delete a session
  async fn delete_session(&self, db: &dyn DatabaseTrait, token: &str) -> Result<()>;
}

/// Public enum for selecting session strategy
#[derive(Debug, Clone, Copy, Default)]
pub enum SessionStrategyType {
  #[default]
  Database,
  // Future: JWT, Redis, etc.
}

impl SessionStrategyType {
  /// Creates a concrete session management strategy corresponding to this variant.
  ///
  /// The returned value is a boxed implementation of `SessionStrategy` for the selected strategy type.
  ///
  /// # Examples
  ///
  /// ```
  /// let strategy = SessionStrategyType::Database.create_strategy();
  /// // `strategy` is a Box<dyn SessionStrategy> ready to be used.
  /// ```
  pub(crate) fn create_strategy(self) -> Box<dyn SessionStrategy> {
    match self {
      Self::Database => Box::new(database_strategy::DatabaseSessionStrategy),
    }
  }
}