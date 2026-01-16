use crate::auth::Auth;
use crate::error::{AuthError, Result};
use crate::types::Session;

#[derive(Debug, Clone)]
pub struct Login {
  pub email: String,
  pub password: String,
}

/// Authenticate the provided credentials and create a session token for the user.
///
/// On success returns a `Session` containing the newly generated token, the user's id,
/// and the expiration timestamp (current UNIX time plus 86400 seconds).
///
/// If the email is not found or the password verification fails, returns `Err(AuthError::InvalidCredentials)`.
/// Other underlying errors from the database, password strategy, or session creation are propagated.
///
/// # Examples
///
/// ```no_run
/// use crate::{Auth, Login};
/// # async fn example(auth: &Auth) -> anyhow::Result<()> {
/// let login = Login { email: "user@example.com".into(), password: "secret".into() };
/// // `execute` is async; run it on a runtime (example uses tokio)
/// let session = tokio::runtime::Runtime::new()?.block_on(async {
///     crate::auth::execute(auth, login).await
/// })?;
/// assert!(session.expires_at > 0);
/// # Ok(())
/// # }
/// ```
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