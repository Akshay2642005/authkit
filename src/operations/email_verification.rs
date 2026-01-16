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

/// Generates an email verification token for the specified user.
///
/// Returns an error if the user does not exist or if the user's email is already verified. The generated token expires 24 hours after creation.
///
/// # Returns
///
/// A `VerificationToken` containing the token string, the user's email, and the token expiry timestamp.
///
/// # Examples
///
/// ```no_run
/// # use crate::operations::email_verification::{send_email_verification, SendEmailVerification};
/// # async fn run(auth: &crate::Auth) {
/// let req = SendEmailVerification { user_id: "user-123".into() };
/// let token = send_email_verification(auth, req).await.unwrap();
/// assert!(!token.token.is_empty());
/// # }
/// ```
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

/// Verify an email verification token and mark the associated user's email as verified.
///
/// Verifies the provided token for email verification, marks the token as used, updates the user's
/// `email_verified` timestamp to the current UNIX epoch seconds, and returns the updated user.
///
/// # Errors
///
/// Returns `AuthError::UserNotFound` if the token's user cannot be found, `AuthError::EmailAlreadyVerified`
/// if the user's email is already verified, or any error returned by the token strategy or database calls.
///
/// # Examples
///
/// ```no_run
/// # async fn run_example() -> Result<(), Box<dyn std::error::Error>> {
/// let auth = /* obtain Auth */ unimplemented!();
/// let req = VerifyEmail { token: "token-string".to_string() };
/// let user = verify_email(&auth, req).await?;
/// println!("Verified user id: {}", user.id);
/// # Ok(())
/// # }
/// ```
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

/// Resend an email verification token to the given email address.
///
/// Finds the user by email, returns `UserNotFound` if missing, returns `EmailAlreadyVerified` if the
/// email is already verified, generates a new `EmailVerification` token valid for 24 hours, and
/// returns a `VerificationToken` containing the token value, the user's email, and the expiry time.
///
/// # Examples
///
/// ```
/// #[tokio::test]
/// async fn example_resend_email_verification() {
///     // Setup `auth` and ensure a user exists with the given email for this example.
///     let auth = /* test Auth setup */;
///     let req = crate::operations::email_verification::ResendEmailVerification {
///         email: "user@example.com".into(),
///     };
///
///     let token = crate::operations::email_verification::resend_email_verification(&auth, req)
///         .await
///         .unwrap();
///
///     assert_eq!(token.email, "user@example.com");
///     // `token.token` is a new verification token and `token.expires_at` is ~24 hours in the future.
/// }
/// ```
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