use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
	pub id: String,
	pub email: String,
	pub created_at: i64,
	pub email_verified: bool,
	pub email_verified_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VerificationToken {
	pub token: String,
	pub email: String,
	pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
	pub token: String,
	pub user_id: String,
	pub expires_at: i64,
}

pub struct Database {
	pub(crate) inner: DatabaseInner,
}

impl Database {
	#[cfg(feature = "sqlite")]
	pub async fn sqlite(path: &str) -> crate::Result<Self> {
		let inner = crate::database::sqlite::SqliteDatabase::new(path).await?;
		Ok(Database {
			inner: DatabaseInner::Sqlite(inner),
		})
	}

	#[cfg(feature = "postgres")]
	pub async fn postgres(url: &str) -> crate::Result<Self> {
		let inner = crate::database::postgres::PostgresDatabase::new(url).await?;
		Ok(Database {
			inner: DatabaseInner::Postgres(inner),
		})
	}
}
#[derive(Clone)]
pub(crate) enum DatabaseInner {
	#[cfg(feature = "sqlite")]
	Sqlite(crate::database::sqlite::SqliteDatabase),
	#[cfg(feature = "postgres")]
	Postgres(crate::database::postgres::PostgresDatabase),
}

impl Clone for Database {
	fn clone(&self) -> Self {
		Database {
			inner: self.inner.clone(),
		}
	}
}

impl fmt::Debug for Database {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Database").finish_non_exhaustive()
	}
}
