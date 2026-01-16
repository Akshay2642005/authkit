//! Error handling and edge case tests
//!
//! These tests verify that AuthKit properly handles:
//! - Error conditions
//! - Edge cases
//! - Boundary conditions
//! - Security concerns

use crate::error::AuthError;
use crate::prelude::*;
use crate::tests::integration_tests::setup_test_auth;

#[tokio::test]
async fn test_builder_missing_database() {
  let result = Auth::builder().build();

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::MissingDatabase));
}

#[tokio::test]
async fn test_empty_email() {
  let auth = setup_test_auth().await.unwrap();

  let result = auth
    .register(Register {
      email: "".into(),
      password: "SecurePass123".into(),
    })
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::InvalidEmailFormat));
}

#[tokio::test]
async fn test_empty_password() {
  let auth = setup_test_auth().await.unwrap();

  let result = auth
    .register(Register {
      email: "test@example.com".into(),
      password: "".into(),
    })
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::WeakPassword(_)));
}

#[tokio::test]
async fn test_whitespace_only_email() {
  let auth = setup_test_auth().await.unwrap();

  let result = auth
    .register(Register {
      email: "   ".into(),
      password: "SecurePass123".into(),
    })
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::InvalidEmailFormat));
}

/// Verifies that a password containing spaces is accepted when it meets strength requirements.
///
/// # Examples
///
/// ```
/// # use crate::tests::integration_tests::setup_test_auth;
/// # use authkit::prelude::Register;
/// #[tokio::test]
/// async fn whitespace_in_password_example() {
///     let auth = setup_test_auth().await.unwrap();
///     let res = auth.register(Register {
///         email: "test@example.com".into(),
///         password: "Secure Pass 123".into(),
///     }).await;
///     assert!(res.is_ok());
/// }
/// ```
#[tokio::test]
async fn test_whitespace_in_password() {
  let auth = setup_test_auth().await.unwrap();

  // Password with spaces should still work if it meets requirements
  let result = auth
    .register(Register {
      email: "test@example.com".into(),
      password: "Secure Pass 123".into(),
    })
    .await;

  assert!(result.is_ok());
}

/// Checks registration behavior with an extremely long email local part.
///
/// The test attempts to register using an email whose local part is 200 characters long.
/// The outcome depends on database or storage constraints; the test only performs the call
/// and documents current behavior without asserting a specific result.
///
/// # Examples
///
/// ```
/// # async {
/// let auth = setup_test_auth().await.unwrap();
/// let long_local = "a".repeat(200);
/// let email = format!("{}@example.com", long_local);
/// let result = auth
///     .register(Register { email, password: "SecurePass123".into() })
///     .await;
/// // result may be Ok or an error depending on backend constraints
/// # };
/// ```
#[tokio::test]
async fn test_very_long_email() {
  let auth = setup_test_auth().await.unwrap();

  // Create an extremely long but valid email
  let long_local = "a".repeat(200);
  let email = format!("{}@example.com", long_local);

  let result = auth
    .register(Register {
      email,
      password: "SecurePass123".into(),
    })
    .await;

  // Should succeed or fail gracefully depending on database constraints
  // This documents current behavior
  let _ = result;
}

#[tokio::test]
async fn test_special_characters_in_email() {
  let auth = setup_test_auth().await.unwrap();

  // These should be valid per RFC 5322
  let valid_emails = vec![
    "user+tag@example.com",
    "user.name@example.com",
    "user_name@example.com",
    "user-name@example.com",
  ];

  for email in valid_emails {
    let result = auth
      .register(Register {
        email: email.into(),
        password: "SecurePass123".into(),
      })
      .await;
    assert!(result.is_ok(), "Failed for email: {}", email);
  }
}

#[tokio::test]
async fn test_sql_injection_in_email() {
  let auth = setup_test_auth().await.unwrap();

  // Attempt SQL injection in email field
  let malicious_emails = vec![
    "'; DROP TABLE users; --@example.com",
    "admin'--@example.com",
    "' OR '1'='1@example.com",
  ];

  for email in malicious_emails {
    let result = auth
      .register(Register {
        email: email.into(),
        password: "SecurePass123".into(),
      })
      .await;

    // Should fail validation or be safely escaped
    // Either way, it should not cause SQL injection
    let _ = result;
  }

  // Verify the auth system still works
  let result = auth
    .register(Register {
      email: "safe@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await;
  assert!(result.is_ok());
}

/// Verifies that passwords containing SQL injection payloads are treated as data and do not execute.
///
/// This test ensures the system accepts and authenticates a password that looks like a SQL injection
/// payload (the password is hashed/stored as data, not executed).
///
/// # Examples
///
/// ```
/// # use crate::tests::integration_tests::setup_test_auth;
/// # use authkit::prelude::{Register, Login};
/// # tokio_test::block_on(async {
/// let auth = setup_test_auth().await.unwrap();
///
/// let pwd = "Password123'; DROP TABLE users; --";
/// let res = auth.register(Register { email: "test@example.com".into(), password: pwd.into() }).await;
/// assert!(res.is_ok());
///
/// let login_res = auth.login(Login { email: "test@example.com".into(), password: pwd.into() }).await;
/// assert!(login_res.is_ok());
/// # });
/// ```
#[tokio::test]
async fn test_sql_injection_in_password() {
  let auth = setup_test_auth().await.unwrap();

  // Register with SQL injection attempt in password
  let result = auth
    .register(Register {
      email: "test@example.com".into(),
      password: "Password123'; DROP TABLE users; --".into(),
    })
    .await;

  // Should succeed (password is hashed, not executed)
  assert!(result.is_ok());

  // Login should work with the same "malicious" password
  let login_result = auth
    .login(Login {
      email: "test@example.com".into(),
      password: "Password123'; DROP TABLE users; --".into(),
    })
    .await;
  assert!(login_result.is_ok());
}

#[tokio::test]
async fn test_empty_token_verify() {
  let auth = setup_test_auth().await.unwrap();

  let result = auth.verify(Verify::new("")).await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::InvalidSession));
}

/// Verifies that logging out with an empty session token completes without error.
///
/// Ensures that calling `logout` with an empty token does not panic and returns `Ok`.
///
/// # Examples
///
/// ```
/// #[tokio::test]
/// async fn example_logout_empty_token() {
///     let auth = setup_test_auth().await.unwrap();
///     let result = auth.logout(Logout::new("")).await;
///     assert!(result.is_ok());
/// }
/// ```
#[tokio::test]
async fn test_empty_token_logout() {
  let auth = setup_test_auth().await.unwrap();

  // Logout with empty token should not panic
  let result = auth.logout(Logout::new("")).await;
  assert!(result.is_ok());
}

/// Verifies that calling logout twice with the same session token does not produce an error (logout is idempotent).
///
/// # Examples
///
/// ```
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let auth = setup_test_auth().await.unwrap();
///
/// auth.register(Register {
///     email: "double@example.com".into(),
///     password: "SecurePass123".into(),
/// }).await.unwrap();
///
/// let session = auth.login(Login {
///     email: "double@example.com".into(),
///     password: "SecurePass123".into(),
/// }).await.unwrap();
///
/// auth.logout(Logout::new(&session.token)).await.unwrap();
///
/// // second logout must not error
/// let res = auth.logout(Logout::new(&session.token)).await;
/// assert!(res.is_ok());
/// # Ok(()) }
/// ```
#[tokio::test]
async fn test_double_logout() {
  let auth = setup_test_auth().await.unwrap();

  // Register and login
  auth
    .register(Register {
      email: "double@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  let session = auth
    .login(Login {
      email: "double@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // First logout
  auth.logout(Logout::new(&session.token)).await.unwrap();

  // Second logout should not error
  let result = auth.logout(Logout::new(&session.token)).await;
  assert!(result.is_ok());
}

/// Ensures a session token is invalidated by logout.
///
/// Registers a user, logs in to obtain a session token, logs out using that token,
/// and asserts that verifying the same token afterwards fails with `AuthError::InvalidSession`.
///
/// # Examples
///
/// ```rust
/// # async fn run(auth: crate::tests::integration_tests::setup_test_auth::TestAuth) {
/// let auth = auth.await.unwrap();
///
/// auth.register(crate::prelude::Register {
///     email: "verify@example.com".into(),
///     password: "SecurePass123".into(),
/// }).await.unwrap();
///
/// let session = auth.login(crate::prelude::Login {
///     email: "verify@example.com".into(),
///     password: "SecurePass123".into(),
/// }).await.unwrap();
///
/// auth.logout(crate::prelude::Logout::new(&session.token)).await.unwrap();
///
/// let result = auth.verify(crate::prelude::Verify::new(&session.token)).await;
/// assert!(matches!(result.unwrap_err(), crate::error::AuthError::InvalidSession));
/// # }
/// ```
#[tokio::test]
async fn test_verify_after_logout() {
  let auth = setup_test_auth().await.unwrap();

  auth
    .register(Register {
      email: "verify@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  let session = auth
    .login(Login {
      email: "verify@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // Logout
  auth.logout(Logout::new(&session.token)).await.unwrap();

  // Verify should fail
  let result = auth.verify(Verify::new(&session.token)).await;
  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::InvalidSession));
}

#[tokio::test]
async fn test_concurrent_operations() {
  let auth = setup_test_auth().await.unwrap();

  // Register user
  auth
    .register(Register {
      email: "concurrent@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // Create multiple concurrent login attempts
  let auth1 = auth.clone();
  let auth2 = auth.clone();
  let auth3 = auth.clone();

  let handle1 = tokio::spawn(async move {
    auth1
      .login(Login {
        email: "concurrent@example.com".into(),
        password: "SecurePass123".into(),
      })
      .await
  });

  let handle2 = tokio::spawn(async move {
    auth2
      .login(Login {
        email: "concurrent@example.com".into(),
        password: "SecurePass123".into(),
      })
      .await
  });

  let handle3 = tokio::spawn(async move {
    auth3
      .login(Login {
        email: "concurrent@example.com".into(),
        password: "SecurePass123".into(),
      })
      .await
  });

  // All should succeed
  let result1 = handle1.await.unwrap();
  let result2 = handle2.await.unwrap();
  let result3 = handle3.await.unwrap();

  assert!(result1.is_ok());
  assert!(result2.is_ok());
  assert!(result3.is_ok());
}

#[tokio::test]
async fn test_password_with_null_bytes() {
  let auth = setup_test_auth().await.unwrap();

  // Password with null byte
  let password = "Pass\0word123";

  let result = auth
    .register(Register {
      email: "null@example.com".into(),
      password: password.into(),
    })
    .await;

  // Should handle gracefully (either accept or reject consistently)
  let _ = result;
}

/// Verifies that registration and subsequent login work with passwords containing Unicode characters.
///
/// Attempts to register accounts using several non-ASCII passwords (e.g., accented letters, symbols, and multi-byte scripts).
/// For each password, if registration succeeds the test asserts that logging in with the same password also succeeds.
#[tokio::test]
async fn test_unicode_in_password() {
  let auth = setup_test_auth().await.unwrap();

  let passwords = [
    "Pässw0rd",       // German umlaut
    "P@ssw0rd™",      // Trademark symbol
    "パスワード123A", // Japanese + ASCII
    "Contraseña1",    // Spanish ñ
  ];

  for (i, password) in passwords.iter().enumerate() {
    let email = format!("unicode{}@example.com", i);
    let result = auth
      .register(Register {
        email: email.clone(),
        password: password.to_string(),
      })
      .await;

    if result.is_ok() {
      // If registration succeeds, login should work
      let login_result = auth
        .login(Login {
          email: email.clone(),
          password: password.to_string(),
        })
        .await;
      assert!(
        login_result.is_ok(),
        "Login failed for password: {}",
        password
      );
    }
  }
}

/// Verifies that registering accounts with email addresses containing subdomains succeeds.
///
/// # Examples
///
/// ```
/// # async fn _run_test() {
/// let auth = setup_test_auth().await.unwrap();
/// let emails = [
///     "user@mail.example.com",
///     "user@sub.mail.example.com",
///     "user@deep.sub.mail.example.com",
/// ];
/// for &email in &emails {
///     let result = auth.register(Register {
///         email: email.into(),
///         password: "SecurePass123".into(),
///     }).await;
///     assert!(result.is_ok(), "Failed for email: {}", email);
/// }
/// # }
/// ```
#[tokio::test]
async fn test_email_with_subdomains() {
  let auth = setup_test_auth().await.unwrap();

  let emails = vec![
    "user@mail.example.com",
    "user@sub.mail.example.com",
    "user@deep.sub.mail.example.com",
  ];

  for email in emails {
    let result = auth
      .register(Register {
        email: email.into(),
        password: "SecurePass123".into(),
      })
      .await;
    assert!(result.is_ok(), "Failed for email: {}", email);
  }
}

#[tokio::test]
async fn test_very_long_token() {
  let auth = setup_test_auth().await.unwrap();

  let long_token = "a".repeat(10000);
  let result = auth.verify(Verify::new(long_token)).await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::InvalidSession));
}

/// Verifies that registering an account with leading or trailing spaces in the email fails.
///
/// The test documents current behavior where email input is not automatically trimmed;
/// an email with surrounding whitespace should produce an error when attempting to register.
///
/// # Examples
///
/// ```
/// # async fn run_example() {
/// let auth = setup_test_auth().await.unwrap();
/// let result = auth.register(Register {
///     email: "  test@example.com  ".into(),
///     password: "SecurePass123".into(),
/// }).await;
/// assert!(result.is_err());
/// # }
/// ```
#[tokio::test]
async fn test_register_login_with_trimmed_spaces() {
  let auth = setup_test_auth().await.unwrap();

  // Register with email that has leading/trailing spaces
  let result = auth
    .register(Register {
      email: "  test@example.com  ".into(),
      password: "SecurePass123".into(),
    })
    .await;

  // Current behavior: spaces are NOT trimmed automatically
  // This documents that behavior
  assert!(result.is_err());
}

#[tokio::test]
async fn test_error_types_are_sendable() {
  // Compile-time check that AuthError implements Send
  /// Asserts that a type implements `Send` at compile time.
///
/// This function enforces the `Send` bound for `T`; if `T` does not implement `Send`,
/// code that references this function with `T` will fail to compile.
///
/// # Examples
///
/// ```
/// #[test]
/// fn compile_time_send_assertion() {
///     // Compiles: i32 implements Send
///     assert_send::<i32>();
/// }
/// ```
fn assert_send<T: Send>() {}
  assert_send::<AuthError>();
}

/// Asserts at compile time that `AuthError` implements `Sync`.
///
/// # Examples
///
/// ```
/// fn assert_sync<T: Sync>() {}
/// assert_sync::<AuthError>();
/// ```
#[tokio::test]
async fn test_error_types_are_sync() {
  // Compile-time check that AuthError implements Sync
  /// Asserts that a type implements `Sync`.
///
/// This function triggers a compile-time error if `T` does not implement the `Sync` trait.
///
/// # Examples
///
/// ```
/// // Compiles: `i32` is `Sync`.
/// assert_sync::<i32>();
/// ```
fn assert_sync<T: Sync>() {}
  assert_sync::<AuthError>();
}

#[tokio::test]
async fn test_auth_is_send() {
  // Compile-time check that Auth implements Send
  /// Asserts that a type implements `Send` at compile time.
///
/// This function enforces the `Send` bound for `T`; if `T` does not implement `Send`,
/// code that references this function with `T` will fail to compile.
///
/// # Examples
///
/// ```
/// #[test]
/// fn compile_time_send_assertion() {
///     // Compiles: i32 implements Send
///     assert_send::<i32>();
/// }
/// ```
fn assert_send<T: Send>() {}
  assert_send::<Auth>();
}

#[tokio::test]
async fn test_auth_is_sync() {
  // Compile-time check that Auth implements Sync
  /// Asserts that a type implements `Sync`.
///
/// This function triggers a compile-time error if `T` does not implement the `Sync` trait.
///
/// # Examples
///
/// ```
/// // Compiles: `i32` is `Sync`.
/// assert_sync::<i32>();
/// ```
fn assert_sync<T: Sync>() {}
  assert_sync::<Auth>();
}