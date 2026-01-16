use crate::error::{AuthError, Result};
use regex::Regex;
use std::sync::OnceLock;

static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();

/// Provides access to a compiled, lazily initialized regular expression for validating email addresses.
///
/// The regex is stored in a global `OnceLock` and initialized on first use.
///
/// # Examples
///
/// ```
/// let re = email_regex();
/// assert!(re.is_match("user@example.com"));
/// assert!(!re.is_match("invalid-email"));
/// ```
fn email_regex() -> &'static Regex {
  EMAIL_REGEX
    .get_or_init(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap())
}

/// Validates that a string is a well-formed email address.
///
/// Returns `Ok(())` if the input matches the expected email pattern, `Err(AuthError::InvalidEmailFormat)` otherwise.
///
/// # Examples
///
/// ```
/// use crate::auth::validate;
/// use crate::error::AuthError;
///
/// assert!(validate("user@example.com").is_ok());
/// assert_eq!(validate("not-an-email"), Err(AuthError::InvalidEmailFormat));
/// ```
pub fn validate(email: &str) -> Result<()> {
  if email_regex().is_match(email) {
    Ok(())
  } else {
    Err(AuthError::InvalidEmailFormat)
  }
}