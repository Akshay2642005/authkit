pub mod models;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

use crate::error::Result;
use crate::types::{DatabaseInner, User};
use async_trait::async_trait;
use models::{DbSession, DbToken, DbUser};

/// Core database trait for AuthKit
///
/// This trait abstracts database operations across different backends (SQLite, Postgres).
/// Methods are organized by feature area for easier maintenance and extension.
#[async_trait]
pub(crate) trait DatabaseTrait: Send + Sync {
	// ==========================================
	// Schema Management
	// ==========================================

	/// Run database migrations to set up or update schema
	async fn migrate(&self) -> Result<()>;

	// ==========================================
	// User Operations
	// ==========================================

	/// Find a user by their email address
	async fn find_user_by_email(&self, email: &str) -> Result<Option<DbUser>>;

	/// Find a user by their unique ID
	async fn find_user_by_id(&self, id: &str) -> Result<Option<User>>;

	/// Create a new user with the given credentials
	async fn create_user(
		&self,
		id: &str,
		email: &str,
		password_hash: &str,
		created_at: i64,
	) -> Result<User>;

	// ==========================================
	// Session Operations
	// ==========================================

	/// Create a new session for a user
	async fn create_session(&self, token: &str, user_id: &str, expires_at: i64) -> Result<()>;

	/// Find a session by its token
	async fn find_session(&self, token: &str) -> Result<Option<DbSession>>;

	/// Delete a specific session
	async fn delete_session(&self, token: &str) -> Result<()>;

	/// Delete all expired sessions (cleanup utility)
	#[allow(dead_code)]
	async fn delete_expired_sessions(&self) -> Result<u64>;

	// ==========================================
	// Token Operations (Email Verification, Password Reset, etc.)
	// ==========================================

	/// Create a new token (email verification, password reset, magic link, etc.)
	#[allow(dead_code)]
	async fn create_token(
		&self,
		id: &str,
		user_id: &str,
		token_hash: &str,
		token_type: &str,
		expires_at: i64,
		created_at: i64,
	) -> Result<()>;

	/// Find a token by its hash and type
	#[allow(dead_code)]
	async fn find_token(&self, token_hash: &str, token_type: &str) -> Result<Option<DbToken>>;

	/// Mark a token as used
	#[allow(dead_code)]
	async fn mark_token_used(&self, token_hash: &str, used_at: i64) -> Result<()>;

	/// Delete a specific token
	/// Delete a specific token by its hash
	#[allow(dead_code)]
	async fn delete_token(&self, token_hash: &str) -> Result<()>;

	/// Delete all expired tokens (cleanup utility)
	#[allow(dead_code)]
	async fn delete_expired_tokens(&self) -> Result<u64>;
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
	"AuthKit requires at least one database backend feature to be enabled.\n\
	 \n\
	 Available backends:\n\
	 - 'sqlite' (enabled by default)\n\
	 - 'postgres'\n\
	 \n\
	 Add one to your Cargo.toml:\n\
	 \n\
	 [dependencies]\n\
	 authkit = { version = \"0.1\", features = [\"sqlite\", \"argon2\"] }\n\
	 \n\
	 Or use the defaults which include sqlite:\n\
	 \n\
	 [dependencies]\n\
	 authkit = \"0.1\""
);
