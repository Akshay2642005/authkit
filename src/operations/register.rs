use crate::auth::Auth;
use crate::error::{AuthError, Result};
use crate::types::User;
use crate::validation;

#[derive(Debug, Clone)]
pub struct Register {
  pub email: String,
  pub password: String,
}

/// Registers a new user with the provided email and password.
///
/// Validates the email and password, ensures no existing user has the same email,
/// hashes the password, generates a user ID and creation timestamp, and persists the user.
///
/// # Parameters
///
/// - `auth`: Authentication context containing database and password strategy.
/// - `request`: Registration payload with `email` and `password`.
///
/// # Returns
///
/// The newly created `User`.
///
/// # Errors
///
/// Returns validation errors for email or password, or `AuthError::UserAlreadyExists(email)`
/// if an account with the given email already exists. Other underlying I/O or hashing errors
/// may also be propagated.
///
/// # Examples
///
/// ```
/// # async fn example(auth: &crate::Auth) -> Result<(), crate::AuthError> {
/// let req = crate::register::Register {
///     email: "alice@example.com".to_string(),
///     password: "s3cur3P@ssw0rd".to_string(),
/// };
/// let user = crate::register::execute(auth, req).await?;
/// assert_eq!(user.email, "alice@example.com");
/// # Ok(())
/// # }
/// ```
pub(crate) async fn execute(auth: &Auth, request: Register) -> Result<User> {
  validation::email::validate(&request.email)?;

  validation::password::validate(&request.password)?;

  if let Some(_existing) = auth.inner.db.find_user_by_email(&request.email).await? {
    return Err(AuthError::UserAlreadyExists(request.email));
  }
  let password_hash = auth
    .inner
    .password_strategy
    .hash_password(&request.password)
    .await?;

  let user_id = crate::security::tokens::generate_id();

  let created_at = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64;

  let user = auth
    .inner
    .db
    .create_user(&user_id, &request.email, &password_hash, created_at)
    .await?;

  Ok(user)
}