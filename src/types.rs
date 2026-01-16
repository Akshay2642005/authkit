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

#[allow(dead_code)]
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
  /// Create a Database backed by a SQLite database at the given filesystem path.
  ///
  /// `path` is the filesystem path to the SQLite database file to open or create.
  ///
  /// # Returns
  ///
  /// `Ok(Database)` on success, or an error from the SQLite backend on failure.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # async fn run() -> crate::Result<()> {
  /// let db = crate::types::Database::sqlite("data/db.sqlite").await?;
  /// // use `db`...
  /// # Ok(())
  /// # }
  /// ```
  #[cfg(feature = "sqlite")]
  pub async fn sqlite(path: &str) -> crate::Result<Self> {
    let inner = crate::database::sqlite::SqliteDatabase::new(path).await?;
    Ok(Database {
      inner: DatabaseInner::Sqlite(inner),
    })
  }

  /// Creates a `Database` backed by a PostgreSQL instance located at the provided connection URL.
  ///
  /// # Parameters
  ///
  /// - `url`: The PostgreSQL connection string (for example, `postgres://user:pass@host:port/dbname`).
  ///
  /// # Returns
  ///
  /// `Ok(Database)` if the PostgreSQL backend was initialized successfully, `Err` otherwise.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use crate::types::Database;
  /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
  /// let db = Database::postgres("postgres://user:password@localhost:5432/mydb").await?;
  /// # Ok(()) }
  /// ```
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
  /// Create a copy of the `Database` with its inner backend cloned.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// // Obtain a `Database` from the appropriate constructor (e.g., `Database::sqlite` or `Database::postgres`).
  /// let db: Database = /* ... */ todo!();
  /// let copy = db.clone();
  /// ```
  fn clone(&self) -> Self {
    Database {
      inner: self.inner.clone(),
    }
  }
}

impl fmt::Debug for Database {
  /// Formats the `Database` for debug output without exposing its internal fields.
  ///
  /// The debug representation shows the struct name ("Database") while omitting internal state.
  ///
  /// # Examples
  ///
  /// ```
  /// use std::mem::MaybeUninit;
  /// use std::fmt::Debug;
  ///
  /// // Create an uninitialized `Database` value for demonstration only.
  /// // This is unsafe and should not be used in production; the example shows
  /// // how the debug formatter presents the type name without revealing internals.
  /// let db: crate::types::Database = unsafe { MaybeUninit::zeroed().assume_init() };
  /// let s = format!("{:?}", db);
  /// assert!(s.starts_with("Database"));
  /// ```
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Database").finish_non_exhaustive()
  }
}