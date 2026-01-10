use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DbUser {
	pub id: String,
	pub email: String,
	pub password_hash: String,
	pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DbSession {
	pub token: String,
	pub user_id: String,
	pub expires_at: i64,
	pub created_at: i64,
}

impl From<DbUser> for crate::types::User {
	fn from(db_user: DbUser) -> Self {
		crate::types::User {
			id: db_user.id,
			email: db_user.email,
			created_at: db_user.created_at,
		}
	}
}
