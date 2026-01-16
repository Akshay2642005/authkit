use crate::error::{AuthError, Result};
use regex::Regex;
use std::sync::OnceLock;

static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();

fn email_regex() -> &'static Regex {
  EMAIL_REGEX
    .get_or_init(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap())
}

/// Validate email format
pub fn validate(email: &str) -> Result<()> {
  if email_regex().is_match(email) {
    Ok(())
  } else {
    Err(AuthError::InvalidEmailFormat)
  }
}
