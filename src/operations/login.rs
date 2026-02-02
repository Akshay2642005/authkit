use crate::auth::Auth;
use crate::error::{AuthError, Result};
use crate::types::Session;

#[derive(Debug, Clone)]
pub struct Login {
  pub email: String,
  pub password: String,
  /// Optional IP address for session tracking
  pub ip_address: Option<String>,
  /// Optional user agent for session tracking
  pub user_agent: Option<String>,
}

pub(crate) async fn execute(auth: &Auth, request: Login) -> Result<Session> {
  // Find user with their credential account (email/password)
  // Use the verification-aware query if email verification is required
  let user_with_account = if auth.inner.require_email_verification {
    // Query includes email_verified columns - requires email_verification feature migration
    auth
      .inner
      .db
      .find_user_with_credential_account_with_verification(&request.email)
      .await?
      .ok_or(AuthError::InvalidCredentials)?
  } else {
    // Query base columns only - no email_verification feature required
    auth
      .inner
      .db
      .find_user_with_credential_account(&request.email)
      .await?
      .ok_or(AuthError::InvalidCredentials)?
  };

  // Get password hash from the account
  let password_hash = user_with_account
    .password_hash()
    .ok_or(AuthError::InvalidCredentials)?;

  // Verify password
  let is_valid = auth
    .inner
    .password_strategy
    .verify_password(&request.password, password_hash)
    .await?;

  if !is_valid {
    return Err(AuthError::InvalidCredentials);
  }

  let user = user_with_account.user;

  // Only check email verification if configured to require it
  if auth.inner.require_email_verification {
    let email_verified = user.email_verified.unwrap_or(false);
    if !email_verified {
      return Err(AuthError::EmailNotVerified(user.email.clone()));
    }
  }

  // Generate session ID and token
  let session_id = crate::security::tokens::generate_id();
  let token = crate::security::tokens::generate_token();

  let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;

  // Session expires in 24 hours by default
  let expires_at = now + 86400;

  // Create the session
  auth
    .inner
    .session_strategy
    .create_session(
      auth.inner.db.as_ref().as_ref(),
      &session_id,
      &token,
      &user.id,
      expires_at,
      request.ip_address.as_deref(),
      request.user_agent.as_deref(),
    )
    .await?;

  Ok(Session {
    id: session_id,
    token,
    user_id: user.id,
    expires_at,
    created_at: now,
    ip_address: request.ip_address,
    user_agent: request.user_agent,
  })
}
