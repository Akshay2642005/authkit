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
  /// Converts a database `DbUser` record into the public `crate::types::User` representation.
  ///
  /// # Examples
  ///
  /// ```
  /// let db_user = DbUser {
  ///     id: "user-1".into(),
  ///     email: "user@example.com".into(),
  ///     password_hash: "hash".into(),
  ///     email_verified: true,
  ///     email_verified_at: Some(1_640_995_200),
  ///     created_at: 1_640_995_200,
  /// };
  /// let user: crate::types::User = db_user.into();
  /// assert_eq!(user.id, "user-1");
  /// assert_eq!(user.email, "user@example.com");
  /// assert!(user.email_verified);
  /// ```
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