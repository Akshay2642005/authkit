use crate::auth::Auth;
use crate::error::{AuthError, Result};
use crate::strategies::token::TokenType;
use crate::types::{User, VerificationToken};

/// Request to send an email verification token
///
/// This generates a verification token for the specified user and returns it.
/// The application is responsible for sending the email with the token.
#[derive(Debug, Clone)]
pub struct SendEmailVerification {
  pub user_id: String,
}

/// Request to verify an email using a token
#[derive(Debug, Clone)]
pub struct VerifyEmail {
  pub token: String,
}

/// Request to resend email verification
#[derive(Debug, Clone)]
pub struct ResendEmailVerification {
  pub email: String,
}

/// Execute email verification send operation
///
/// This generates a verification token for the user. The token is returned
/// in a `VerificationToken` struct, which the application should use to
/// send an email to the user.
///
/// The token expires in 24 hours by default.
pub(crate) async fn send_email_verification(
  auth: &Auth,
  request: SendEmailVerification,
) -> Result<VerificationToken> {
  // Find the user by ID
  let user = auth
    .inner
    .db
    .find_user_by_id(&request.user_id)
    .await?
    .ok_or(AuthError::UserNotFound)?;

  // Check if email is already verified
  if user.email_verified {
    return Err(AuthError::EmailAlreadyVerified(
      "Email is already verified".to_string(),
    ));
  }

  // Generate token (24 hours expiry)
  const TWENTY_FOUR_HOURS: i64 = 24 * 60 * 60;
  let token = auth
    .inner
    .token_strategy
    .generate_token(
      auth.inner.db.as_ref().as_ref(),
      &request.user_id,
      TokenType::EmailVerification,
      TWENTY_FOUR_HOURS,
    )
    .await?;

  Ok(VerificationToken {
    token: token.token,
    email: user.email,
    expires_at: token.expires_at,
  })
}

/// Execute email verification operation
///
/// This verifies the provided token and marks the user's email as verified
/// if the token is valid, not expired, and not already used.
pub(crate) async fn verify_email(auth: &Auth, request: VerifyEmail) -> Result<User> {
  // Verify the token
  let verified_token = auth
    .inner
    .token_strategy
    .verify_token(
      auth.inner.db.as_ref().as_ref(),
      &request.token,
      TokenType::EmailVerification,
    )
    .await?;

  // Get the user
  let user = auth
    .inner
    .db
    .find_user_by_id(&verified_token.user_id)
    .await?
    .ok_or(AuthError::UserNotFound)?;

  // Check if already verified
  if user.email_verified {
    return Err(AuthError::EmailAlreadyVerified(
      "Email is already verified".to_string(),
    ));
  }

  // Mark token as used
  auth
    .inner
    .token_strategy
    .mark_token_as_used(auth.inner.db.as_ref().as_ref(), &request.token)
    .await?;

  // Update user's email_verified status
  let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;

  auth
    .inner
    .db
    .update_email_verified(&verified_token.user_id, now)
    .await?;

  // Return updated user
  let updated_user = auth
    .inner
    .db
    .find_user_by_id(&verified_token.user_id)
    .await?
    .ok_or(AuthError::UserNotFound)?;

  Ok(updated_user)
}

/// Execute resend email verification operation
///
/// This finds the user by email and generates a new verification token
/// if the email is not already verified.
pub(crate) async fn resend_email_verification(
  auth: &Auth,
  request: ResendEmailVerification,
) -> Result<VerificationToken> {
  // Find the user by email
  let db_user = auth
    .inner
    .db
    .find_user_by_email(&request.email)
    .await?
    .ok_or(AuthError::UserNotFound)?;

  // Check if email is already verified
  if db_user.email_verified {
    return Err(AuthError::EmailAlreadyVerified(
      "Email is already verified".to_string(),
    ));
  }

  // Generate new token (24 hours expiry)
  const TWENTY_FOUR_HOURS: i64 = 24 * 60 * 60;
  let token = auth
    .inner
    .token_strategy
    .generate_token(
      auth.inner.db.as_ref().as_ref(),
      &db_user.id,
      TokenType::EmailVerification,
      TWENTY_FOUR_HOURS,
    )
    .await?;

  Ok(VerificationToken {
    token: token.token,
    email: db_user.email,
    expires_at: token.expires_at,
  })
}
