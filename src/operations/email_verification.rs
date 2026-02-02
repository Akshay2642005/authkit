use crate::auth::Auth;
use crate::email::EmailContext;
use crate::error::{AuthError, Result};
use crate::strategies::token::TokenType;
use crate::types::{User, VerificationToken};

#[cfg(feature = "email-queue")]
use crate::email_job::EmailJob;

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
///
/// **Requires:** email_verification feature columns in the database schema.
/// Run `authkit migrate` with email_verification feature enabled.
pub(crate) async fn send_email_verification(
  auth: &Auth,
  request: SendEmailVerification,
) -> Result<VerificationToken> {
  // Find the user by ID with email verification status
  // Uses _with_verification method that queries email_verified columns
  let user = auth
    .inner
    .db
    .find_user_by_id_with_verification(&request.user_id)
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
      &user.email,
      TokenType::EmailVerification,
      TWENTY_FOUR_HOURS,
    )
    .await?;

  // Send verification email (queue or sync based on configuration)
  #[cfg(feature = "email-queue")]
  {
    if let Some(queue) = &auth.inner.email_queue {
      let job = EmailJob::verification(
        user.email.clone(),
        token.token.clone(),
        token.expires_at,
        user.id.clone(),
      );

      match queue.enqueue(job).await {
        Ok(()) => {
          return Ok(VerificationToken {
            token: token.token,
            identifier: user.email,
            expires_at: token.expires_at,
          });
        }
        Err(e) => {
          log::warn!("Email queue error, sending synchronously: {}", e);
          // Fall through to sync send
        }
      }
    }
  }

  // Synchronous send (fallback or when queue not enabled)
  if let Some(email_sender) = &auth.inner.email_sender {
    let context = EmailContext {
      email: user.email.clone(),
      token: token.token.clone(),
      expires_at: token.expires_at,
    };

    email_sender.send_verification_email(context).await?;
  }

  Ok(VerificationToken {
    token: token.token,
    identifier: user.email,
    expires_at: token.expires_at,
  })
}

/// Execute email verification operation
///
/// This verifies the provided token and marks the user's email as verified
/// if the token is valid, not expired, and not already used.
///
/// **Requires:** email_verification feature columns in the database schema.
/// Run `authkit migrate` with email_verification feature enabled.
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

  // Get the user ID from the token
  let user_id = verified_token
    .user_id
    .as_ref()
    .ok_or(AuthError::InvalidToken(
      "Token does not have an associated user".to_string(),
    ))?;

  // Get the user with email verification status
  let user = auth
    .inner
    .db
    .find_user_by_id_with_verification(user_id)
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

  auth.inner.db.update_email_verified(user_id, now).await?;

  // Return updated user with verification status
  let updated_user = auth
    .inner
    .db
    .find_user_by_id_with_verification(user_id)
    .await?
    .ok_or(AuthError::UserNotFound)?;

  Ok(updated_user)
}

/// Execute resend email verification operation
///
/// This finds the user by email and generates a new verification token
/// if the email is not already verified.
///
/// **Requires:** email_verification feature columns in the database schema.
/// Run `authkit migrate` with email_verification feature enabled.
pub(crate) async fn resend_email_verification(
  auth: &Auth,
  request: ResendEmailVerification,
) -> Result<VerificationToken> {
  // Find the user by email with email verification status
  let db_user = auth
    .inner
    .db
    .find_user_by_email_with_verification(&request.email)
    .await?
    .ok_or(AuthError::UserNotFound)?;

  // Check if email is already verified
  let email_verified = db_user.email_verified.unwrap_or(false);
  if email_verified {
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
      &db_user.email,
      TokenType::EmailVerification,
      TWENTY_FOUR_HOURS,
    )
    .await?;

  // Send verification email (queue or sync based on configuration)
  #[cfg(feature = "email-queue")]
  {
    if let Some(queue) = &auth.inner.email_queue {
      let job = EmailJob::verification(
        db_user.email.clone(),
        token.token.clone(),
        token.expires_at,
        db_user.id.clone(),
      );

      match queue.enqueue(job).await {
        Ok(()) => {
          return Ok(VerificationToken {
            token: token.token,
            identifier: db_user.email,
            expires_at: token.expires_at,
          });
        }
        Err(e) => {
          log::warn!("Email queue error, sending synchronously: {}", e);
          // Fall through to sync send
        }
      }
    }
  }

  // Synchronous send (fallback or when queue not enabled)
  if let Some(email_sender) = &auth.inner.email_sender {
    let context = EmailContext {
      email: db_user.email.clone(),
      token: token.token.clone(),
      expires_at: token.expires_at,
    };

    email_sender.send_verification_email(context).await?;
  }

  Ok(VerificationToken {
    token: token.token,
    identifier: db_user.email,
    expires_at: token.expires_at,
  })
}
