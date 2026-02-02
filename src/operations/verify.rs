use crate::auth::Auth;
use crate::error::{AuthError, Result};
use crate::types::User;

/// Verify a session token and return the associated user
///
/// This operation verifies that a session token is valid and not expired,
/// then returns the user associated with the session.

#[derive(Debug, Clone)]
pub struct Verify {
  pub token: String,
}

impl Verify {
  pub fn new(token: impl Into<String>) -> Self {
    Self {
      token: token.into(),
    }
  }
}

impl From<&str> for Verify {
  fn from(token: &str) -> Self {
    Self::new(token)
  }
}

pub(crate) async fn execute(auth: &Auth, request: Verify) -> Result<User> {
  let session = auth
    .inner
    .session_strategy
    .find_session(auth.inner.db.as_ref().as_ref(), &request.token)
    .await?
    .ok_or(AuthError::InvalidSession)?;

  let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;

  if session.expires_at < now {
    return Err(AuthError::InvalidSession);
  }

  // Use the verification-aware query if email verification feature is used
  // This ensures we return accurate email_verified status
  if auth.inner.require_email_verification {
    auth
      .inner
      .db
      .find_user_by_id_with_verification(&session.user_id)
      .await?
      .ok_or(AuthError::UserNotFound)
  } else {
    auth
      .inner
      .db
      .find_user_by_id(&session.user_id)
      .await?
      .ok_or(AuthError::UserNotFound)
  }
}
