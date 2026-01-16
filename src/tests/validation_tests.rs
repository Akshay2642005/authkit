//! Validation tests for email and password validation

use crate::error::AuthError;
use crate::validation::{email, password};

#[test]
fn test_valid_email() {
  assert!(email::validate("user@example.com").is_ok());
  assert!(email::validate("test.user@domain.co.uk").is_ok());
  assert!(email::validate("user+tag@example.com").is_ok());
  assert!(email::validate("user_name@example.org").is_ok());
  assert!(email::validate("123@example.com").is_ok());
}

#[test]
fn test_invalid_email() {
  assert!(matches!(
    email::validate("invalid"),
    Err(AuthError::InvalidEmailFormat)
  ));
  assert!(matches!(
    email::validate("@example.com"),
    Err(AuthError::InvalidEmailFormat)
  ));
  assert!(matches!(
    email::validate("user@"),
    Err(AuthError::InvalidEmailFormat)
  ));
  assert!(matches!(
    email::validate("user@domain"),
    Err(AuthError::InvalidEmailFormat)
  ));
  assert!(matches!(
    email::validate("user domain@example.com"),
    Err(AuthError::InvalidEmailFormat)
  ));
  assert!(matches!(
    email::validate(""),
    Err(AuthError::InvalidEmailFormat)
  ));
}

#[test]
fn test_valid_password() {
  assert!(password::validate("Password123").is_ok());
  assert!(password::validate("Abcdefgh1").is_ok());
  assert!(password::validate("MyP@ssw0rd").is_ok());
  assert!(password::validate("Str0ngP@ssw0rd!").is_ok());
}

#[test]
fn test_password_too_short() {
  let result = password::validate("Short1");
  assert!(matches!(result, Err(AuthError::WeakPassword(_))));
  if let Err(AuthError::WeakPassword(msg)) = result {
    assert!(msg.contains("at least 8 characters"));
  }
}

#[test]
fn test_password_too_long() {
  let long_password = "A1".to_string() + &"a".repeat(127);
  let result = password::validate(&long_password);
  assert!(matches!(result, Err(AuthError::WeakPassword(_))));
  if let Err(AuthError::WeakPassword(msg)) = result {
    assert!(msg.contains("at most 128 characters"));
  }
}

#[test]
fn test_password_no_uppercase() {
  let result = password::validate("password123");
  assert!(matches!(result, Err(AuthError::WeakPassword(_))));
  if let Err(AuthError::WeakPassword(msg)) = result {
    assert!(msg.contains("uppercase"));
  }
}

#[test]
fn test_password_no_lowercase() {
  let result = password::validate("PASSWORD123");
  assert!(matches!(result, Err(AuthError::WeakPassword(_))));
  if let Err(AuthError::WeakPassword(msg)) = result {
    assert!(msg.contains("lowercase"));
  }
}

/// Ensures a password with no digits is rejected and the error message references "digit".
///
/// # Examples
///
/// ```
/// let result = password::validate("Password");
/// assert!(matches!(result, Err(AuthError::WeakPassword(_))));
/// if let Err(AuthError::WeakPassword(msg)) = result {
///     assert!(msg.contains("digit"));
/// }
/// ```
#[test]
fn test_password_no_digit() {
  let result = password::validate("Password");
  assert!(matches!(result, Err(AuthError::WeakPassword(_))));
  if let Err(AuthError::WeakPassword(msg)) = result {
    assert!(msg.contains("digit"));
  }
}

/// Verifies that a password with the minimum allowed length is accepted.
///
/// # Examples
///
/// ```
/// assert!(crate::validation::password::validate("Passwor1").is_ok());
/// ```
#[test]
fn test_password_exactly_min_length() {
  assert!(password::validate("Passwor1").is_ok());
}

#[test]
fn test_password_exactly_max_length() {
  let max_password = "A1".to_string() + &"a".repeat(126);
  assert_eq!(max_password.len(), 128);
  assert!(password::validate(&max_password).is_ok());
}

#[test]
fn test_password_with_special_characters() {
  assert!(password::validate("P@ssw0rd!").is_ok());
  assert!(password::validate("My$ecureP@ss1").is_ok());
  assert!(password::validate("Test#Pass123").is_ok());
}

/// Ensures passwords with Unicode characters satisfy the validator's strength rules.
///
/// # Examples
///
/// ```
/// assert!(password::validate("Pässw0rd").is_ok());
/// assert!(password::validate("P4ssw0rd™").is_ok());
/// ```
#[test]
fn test_password_unicode_characters() {
  // Unicode characters should work as long as requirements are met
  assert!(password::validate("Pässw0rd").is_ok());
  assert!(password::validate("P4ssw0rd™").is_ok());
}

#[test]
fn test_empty_password() {
  let result = password::validate("");
  assert!(matches!(result, Err(AuthError::WeakPassword(_))));
}

#[test]
fn test_email_edge_cases() {
  // Multiple dots in local part
  assert!(email::validate("user.name.test@example.com").is_ok());

  // Numbers in domain
  assert!(email::validate("user@example123.com").is_ok());

  // Subdomain
  assert!(email::validate("user@mail.example.com").is_ok());
}