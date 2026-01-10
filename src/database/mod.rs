pub mod models;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

use crate::error::Result;
use crate::types::{DatabaseInner, User};
use async_trait::async_trait;
use models::{DbSession, DbUser};

#[async_trait]
pub(crate) trait DatabaseTrait: Send + Sync {
	async fn migrate(&self) -> Result<()>;

	async fn find_user_by_email(&self, email: &str) -> Result<Option<DbUser>>;

	async fn find_user_by_id(&self, id: &str) -> Result<Option<User>>;

	async fn create_user(
		&self,
		id: &str,
		email: &str,
		password_hash: &str,
		created_at: i64,
	) -> Result<User>;

	async fn create_session(&self, token: &str, user_id: &str, expires_at: i64) -> Result<()>;

	async fn find_session(&self, token: &str) -> Result<Option<DbSession>>;

	async fn delete_session(&self, token: &str) -> Result<()>;

	/// Delete expired sessions (cleanup utility for future use)
	#[allow(dead_code)]
	async fn delete_expired_sessions(&self) -> Result<u64>;
}

pub(crate) fn create_database_trait(inner: DatabaseInner) -> Box<dyn DatabaseTrait> {
	match inner {
		#[cfg(feature = "sqlite")]
		DatabaseInner::Sqlite(db) => Box::new(db),
		#[cfg(feature = "postgres")]
		DatabaseInner::Postgres(db) => Box::new(db),
	}
}

// Compile-time check: at least one database backend must be enabled
#[cfg(not(any(feature = "sqlite", feature = "postgres")))]
compile_error!(
	"AuthKit requires at least one database backend. \
	 Enable one of: 'sqlite', 'postgres'. \
	 Example: cargo build --features sqlite"
);
