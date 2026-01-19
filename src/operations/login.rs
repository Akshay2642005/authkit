use crate::auth::Auth;
use crate::error::{AuthError, Result};
use crate::types::Session;

#[derive(Debug, Clone)]
pub struct Login {
  pub email: String,
  pub password: String,
}

pub(crate) async fn execute(auth: &Auth, request: Login) -> Result<Session> {
  let user = auth
    .inner
    .db
    .find_user_by_email(&request.email)
    .await?
    .ok_or(AuthError::InvalidCredentials)?;

  let is_valid = auth
    .inner
    .password_strategy
    .verify_password(&request.password, &user.password_hash)
    .await?;

  if !is_valid {
    return Err(AuthError::InvalidCredentials);
  }

  // Only check email verification if configured to require it
  if auth.inner.require_email_verification && !user.email_verified {
    return Err(AuthError::EmailNotVerified(user.email.clone()));
  }

  let token = crate::security::tokens::generate_token();

  let expires_at = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64
    + 86400;
  auth
    .inner
    .session_strategy
    .create_session(
      auth.inner.db.as_ref().as_ref(),
      &token,
      &user.id,
      expires_at,
    )
    .await?;

  Ok(Session {
    token,
    user_id: user.id,
    expires_at,
  })
}
