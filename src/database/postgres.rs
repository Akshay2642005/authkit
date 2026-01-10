use crate::database::DatabaseTrait;
#[cfg(feature = "postgres")]
use crate::database::models::{DbSession, DbUser};
use crate::error::Result;
use crate::types::User;
use async_trait::async_trait;
use sqlx::Row;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;

pub struct PostgresDatabase {
	pool: PgPool,
}

impl PostgresDatabase {
	pub async fn new(url: &str) -> Result<Self> {
		let options = PgConnectOptions::from_str(url)?;

		let pool = PgPoolOptions::new()
			.max_connections(5)
			.connect_with(options)
			.await?;

		Ok(Self { pool })
	}
}

#[async_trait]
impl DatabaseTrait for PostgresDatabase {
	async fn migrate(&self) -> Result<()> {
		sqlx::query(
			r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at BIGINT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sessions (
                token TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                expires_at BIGINT NOT NULL,
                created_at BIGINT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
            "#,
		)
		.execute(&self.pool)
		.await?;

		Ok(())
	}

	async fn find_user_by_email(&self, email: &str) -> Result<Option<DbUser>> {
		let user = sqlx::query(
			r#"
            SELECT id, email, password_hash, created_at
            FROM users
            WHERE email = $1
            "#,
		)
		.bind(email)
		.map(|row: sqlx::postgres::PgRow| DbUser {
			id: row.get("id"),
			email: row.get("email"),
			password_hash: row.get("password_hash"),
			created_at: row.get("created_at"),
		})
		.fetch_optional(&self.pool)
		.await?;

		Ok(user)
	}

	async fn find_user_by_id(&self, id: &str) -> Result<Option<User>> {
		let user = sqlx::query(
			r#"
            SELECT id, email, password_hash, created_at
            FROM users
            WHERE id = $1
            "#,
		)
		.bind(id)
		.map(|row: sqlx::postgres::PgRow| DbUser {
			id: row.get("id"),
			email: row.get("email"),
			password_hash: row.get("password_hash"),
			created_at: row.get("created_at"),
		})
		.fetch_optional(&self.pool)
		.await?;

		Ok(user.map(Into::into))
	}

	async fn create_user(
		&self,
		id: &str,
		email: &str,
		password_hash: &str,
		created_at: i64,
	) -> Result<User> {
		sqlx::query(
			r#"
            INSERT INTO users (id, email, password_hash, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
		)
		.bind(id)
		.bind(email)
		.bind(password_hash)
		.bind(created_at)
		.execute(&self.pool)
		.await?;

		Ok(User {
			id: id.to_string(),
			email: email.to_string(),
			created_at,
		})
	}

	async fn create_session(&self, token: &str, user_id: &str, expires_at: i64) -> Result<()> {
		let created_at = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs() as i64;

		sqlx::query(
			r#"
            INSERT INTO sessions (token, user_id, expires_at, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
		)
		.bind(token)
		.bind(user_id)
		.bind(expires_at)
		.bind(created_at)
		.execute(&self.pool)
		.await?;

		Ok(())
	}

	async fn find_session(&self, token: &str) -> Result<Option<DbSession>> {
		let session = sqlx::query(
			r#"
            SELECT token, user_id, expires_at, created_at
            FROM sessions
            WHERE token = $1
            "#,
		)
		.bind(token)
		.map(|row: sqlx::postgres::PgRow| DbSession {
			token: row.get("token"),
			user_id: row.get("user_id"),
			expires_at: row.get("expires_at"),
			created_at: row.get("created_at"),
		})
		.fetch_optional(&self.pool)
		.await?;

		Ok(session)
	}

	async fn delete_session(&self, token: &str) -> Result<()> {
		sqlx::query(
			r#"
            DELETE FROM sessions
            WHERE token = $1
            "#,
		)
		.bind(token)
		.execute(&self.pool)
		.await?;

		Ok(())
	}

	async fn delete_expired_sessions(&self) -> Result<u64> {
		let now = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs() as i64;

		let result = sqlx::query(
			r#"
            DELETE FROM sessions
            WHERE expires_at < $1
            "#,
		)
		.bind(now)
		.execute(&self.pool)
		.await?;

		Ok(result.rows_affected())
	}
}
