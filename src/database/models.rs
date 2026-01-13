use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DbUser {
	pub id: String,
	pub email: String,
	pub password_hash: String,
	pub email_verified: bool,
	pub email_verified_at: Option<i64>,
	pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DbSession {
	pub token: String,
	pub user_id: String,
	pub expires_at: i64,
	pub created_at: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DbToken {
	pub id: String,
	pub user_id: String,
	pub token_hash: String,
	pub token_type: String,
	pub expires_at: i64,
	pub created_at: i64,
	pub used_at: Option<i64>,
}

impl From<DbUser> for crate::types::User {
	fn from(db_user: DbUser) -> Self {
		crate::types::User {
			id: db_user.id,
			email: db_user.email,
			email_verified: db_user.email_verified,
			email_verified_at: db_user.email_verified_at,
			created_at: db_user.created_at,
		}
	}
}
