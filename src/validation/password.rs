use crate::error::{AuthError, Result};

const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 128;

/// Validate password strength
///
/// Requirements:
/// - At least 8 characters
/// - At most 128 characters
/// - Contains at least one uppercase letter
/// - Contains at least one lowercase letter
/// - Contains at least one digit
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
