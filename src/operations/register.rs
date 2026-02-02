use crate::auth::Auth;
use crate::email::EmailContext;
use crate::error::{AuthError, Result};
use crate::strategies::token::TokenType;
use crate::types::User;
use crate::validation;

#[cfg(feature = "email-queue")]
use crate::email_job::EmailJob;

#[derive(Debug, Clone)]
pub struct Register {
  pub email: String,
  pub password: String,
  pub name: Option<String>,
}

pub(crate) async fn execute(auth: &Auth, request: Register) -> Result<User> {
  validation::email::validate(&request.email)?;

  validation::password::validate(&request.password)?;

  // Check if user already exists
  if let Some(_existing) = auth.inner.db.find_user_by_email(&request.email).await? {
    return Err(AuthError::UserAlreadyExists(request.email));
  }

  // Hash the password
  let password_hash = auth
    .inner
    .password_strategy
    .hash_password(&request.password)
    .await?;

  let user_id = crate::security::tokens::generate_id();
  let account_id = crate::security::tokens::generate_id();

  let created_at = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;

  // Create the user
  let user = auth
    .inner
    .db
    .create_user(
      &user_id,
      &request.email,
      request.name.as_deref(),
      created_at,
    )
    .await?;

  // Create the credential account (links user to email/password provider)
  auth
    .inner
    .db
    .create_account(
      &account_id,
      &user_id,
      "credential",   // provider type for email/password
      &request.email, // provider_account_id is the email for credentials
      Some(&password_hash),
      created_at,
    )
    .await?;

  // Check if we should send verification email on registration
  if !auth.inner.send_verification_on_register {
    // User opted out of automatic verification emails
    return Ok(user);
  }

  // Check if email sender is configured
  if auth.inner.email_sender.is_none() {
    // No email sender configured, skip sending verification email
    return Ok(user);
  }

  // Generate verification token
  const TWENTY_FOUR_HOURS: i64 = 24 * 60 * 60;
  let token = auth
    .inner
    .token_strategy
    .generate_token(
      auth.inner.db.as_ref().as_ref(),
      &user_id,
      &request.email,
      TokenType::EmailVerification,
      TWENTY_FOUR_HOURS,
    )
    .await?;

  // Try to send verification email via queue (if email-queue feature enabled)
  #[cfg(feature = "email-queue")]
  {
    if let Some(queue) = &auth.inner.email_queue {
      // Queue for async processing (fast path)
      let job = EmailJob::verification(
        user.email.clone(),
        token.token.clone(),
        token.expires_at,
        user.id.clone(),
      );

      // Try to enqueue - if it fails, fall back to sync send
      match queue.enqueue(job).await {
        Ok(()) => {
          // Successfully queued, return immediately
          return Ok(user);
        }
        Err(e) => {
          log::warn!("Email queue error, sending synchronously: {}", e);
          // Fall through to sync send
        }
      }
    }
  }

  // Synchronous send (fallback or when queue not enabled/configured)
  if let Some(email_sender) = &auth.inner.email_sender {
    let context = EmailContext {
      email: user.email.clone(),
      token: token.token,
      expires_at: token.expires_at,
    };

    email_sender.send_verification_email(context).await?;
  }

  Ok(user)
}
