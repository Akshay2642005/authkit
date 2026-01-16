use crate::error::{AuthError, Result};

const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 128;

/// Validates that a password meets the project's strength requirements.
///
/// The password must be between 8 and 128 characters (inclusive), contain at least one
/// uppercase letter, at least one lowercase letter, and at least one digit. On failure
/// returns `AuthError::WeakPassword` with a message describing the first unmet requirement.
///
/// # Examples
///
/// ```
/// use crate::validation::password::validate;
///
/// assert!(validate("StrongPass1").is_ok());
/// assert!(validate("weak").is_err());
/// ```
pub fn validate(password: &str) -> Result<()> {
  if password.len() < MIN_PASSWORD_LENGTH {
    return Err(AuthError::WeakPassword(format!(
      "Password must be at least {} characters",
      MIN_PASSWORD_LENGTH
    )));
  }

  if password.len() > MAX_PASSWORD_LENGTH {
    return Err(AuthError::WeakPassword(format!(
      "Password must be at most {} characters",
      MAX_PASSWORD_LENGTH
    )));
  }

  let has_uppercase = password.chars().any(|c| c.is_uppercase());
  let has_lowercase = password.chars().any(|c| c.is_lowercase());
  let has_digit = password.chars().any(|c| c.is_ascii_digit());

  if !has_uppercase {
    return Err(AuthError::WeakPassword(
      "Password must contain at least one uppercase letter".into(),
    ));
  }

  if !has_lowercase {
    return Err(AuthError::WeakPassword(
      "Password must contain at least one lowercase letter".into(),
    ));
  }

  if !has_digit {
    return Err(AuthError::WeakPassword(
      "Password must contain at least one digit".into(),
    ));
  }

  Ok(())
}