//! Integration tests for the full authentication flow
//!
//! These tests cover the complete lifecycle:
//! - User registration
//! - Login and session creation
//! - Session verification
//! - Logout and session invalidation
//! - Error scenarios

use crate::prelude::*;
use crate::types::Database;

/// Create a test `Auth` instance wired to an in-memory or test database and run migrations.
///
/// Chooses SQLite in-memory when the `sqlite` feature is enabled (unless `postgres` is enabled without `sqlite`),
/// otherwise uses a Postgres test database (configured via `DATABASE_URL` or a localhost default).
///
/// # Examples
///
/// ```
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let auth = crate::tests::setup_test_auth().await?;
/// // use `auth` for test operations...
/// assert!(!auth.clone().to_string().is_empty());
/// # Ok(())
/// # }
/// ```
///
/// # Returns
///
/// `Ok(Auth)` with a fully migrated test `Auth` instance on success, or an error if setup or migrations fail.
pub(crate) async fn setup_test_auth() -> Result<Auth> {
  #[cfg(all(
    feature = "sqlite",
    not(all(feature = "postgres", not(feature = "sqlite")))
  ))]
  {
    // Use SQLite in-memory database
    let db_name = ":memory:".to_string();
    let db = Database::sqlite(&db_name).await?;

    let auth = Auth::builder().database(db).build()?;

    // Run migrations
    auth.migrate().await?;

    Ok(auth)
  }

  #[cfg(all(feature = "postgres", not(feature = "sqlite")))]
  {
    // Use Postgres test database
    // This requires a running Postgres instance with test database
    let db_url = std::env::var("DATABASE_URL")
      .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/authkit_test".to_string());
    let db = Database::postgres(&db_url).await?;

    let auth = Auth::builder().database(db).build()?;

    // Run migrations
    auth.migrate().await?;

    Ok(auth)
  }
}

#[tokio::test]
async fn test_register_user_success() {
  let auth = setup_test_auth().await.unwrap();

  let result = auth
    .register(Register {
      email: "test@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await;

  assert!(result.is_ok());
  let user = result.unwrap();
  assert_eq!(user.email, "test@example.com");
  assert!(!user.id.is_empty());
  assert!(user.created_at > 0);
}

/// Ensures registering a second account with an already-used email returns `AuthError::UserAlreadyExists`.
///
/// # Examples
///
/// ```
/// # async fn run() {
/// let auth = setup_test_auth().await.unwrap();
/// auth.register(Register {
///     email: "duplicate@example.com".into(),
///     password: "SecurePass123".into(),
/// }).await.unwrap();
///
/// let res = auth.register(Register {
///     email: "duplicate@example.com".into(),
///     password: "AnotherPass123".into(),
/// }).await;
///
/// assert!(res.is_err());
/// assert!(matches!(res.unwrap_err(), AuthError::UserAlreadyExists(_)));
/// # }
/// ```
#[tokio::test]
async fn test_register_duplicate_email() {
  let auth = setup_test_auth().await.unwrap();

  // Register first user
  auth
    .register(Register {
      email: "duplicate@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // Try to register with same email
  let result = auth
    .register(Register {
      email: "duplicate@example.com".into(),
      password: "AnotherPass123".into(),
    })
    .await;

  assert!(result.is_err());
  assert!(matches!(
    result.unwrap_err(),
    AuthError::UserAlreadyExists(_)
  ));
}

/// Verifies that registering with an invalid email fails with `AuthError::InvalidEmailFormat`.
///
/// # Examples
///
/// ```
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let auth = setup_test_auth().await.unwrap();
///
/// let result = auth
///     .register(Register {
///         email: "not-an-email".into(),
///         password: "SecurePass123".into(),
///     })
///     .await;
///
/// assert!(result.is_err());
/// assert!(matches!(result.unwrap_err(), AuthError::InvalidEmailFormat));
/// # Ok(()) }
/// ```
#[tokio::test]
async fn test_register_invalid_email() {
  let auth = setup_test_auth().await.unwrap();

  let result = auth
    .register(Register {
      email: "not-an-email".into(),
      password: "SecurePass123".into(),
    })
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::InvalidEmailFormat));
}

/// Verifies that registering with a weak password fails with `AuthError::WeakPassword`.
///
/// # Examples
///
/// ```
/// # async fn run() {
/// let auth = setup_test_auth().await.unwrap();
///
/// let result = auth
///     .register(crate::Register {
///         email: "test@example.com".into(),
///         password: "weak".into(),
///     })
///     .await;
///
/// assert!(result.is_err());
/// assert!(matches!(result.unwrap_err(), crate::AuthError::WeakPassword(_)));
/// # }
/// ```
#[tokio::test]
async fn test_register_weak_password() {
  let auth = setup_test_auth().await.unwrap();

  let result = auth
    .register(Register {
      email: "test@example.com".into(),
      password: "weak".into(),
    })
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::WeakPassword(_)));
}

/// Verifies that a registered user can log in and receives a valid session.
///
/// Registers a user, logs in with the same credentials, and asserts the returned session
/// contains a non-empty token, the expected `user_id`, and a positive `expires_at`.
///
/// # Examples
///
/// ```
/// // Setup test auth, register a user, then login and inspect the session:
/// let auth = setup_test_auth().await.unwrap();
/// let user = auth.register(Register { email: "login@example.com".into(), password: "SecurePass123".into() }).await.unwrap();
/// let session = auth.login(Login { email: "login@example.com".into(), password: "SecurePass123".into() }).await.unwrap();
/// assert!(!session.token.is_empty());
/// assert_eq!(session.user_id, user.id);
/// assert!(session.expires_at > 0);
/// ```
#[tokio::test]
async fn test_login_success() {
  let auth = setup_test_auth().await.unwrap();

  // Register user
  let user = auth
    .register(Register {
      email: "login@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // Login
  let result = auth
    .login(Login {
      email: "login@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await;

  assert!(result.is_ok());
  let session = result.unwrap();
  assert!(!session.token.is_empty());
  assert_eq!(session.user_id, user.id);
  assert!(session.expires_at > 0);
}

#[tokio::test]
async fn test_login_wrong_password() {
  let auth = setup_test_auth().await.unwrap();

  // Register user
  auth
    .register(Register {
      email: "test@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // Try to login with wrong password
  let result = auth
    .login(Login {
      email: "test@example.com".into(),
      password: "WrongPass123".into(),
    })
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::InvalidCredentials));
}

/// Verifies that attempting to log in with an email that is not registered fails.
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// let auth = setup_test_auth().await.unwrap();
///
/// let result = auth
///     .login(Login {
///         email: "nonexistent@example.com".into(),
///         password: "SecurePass123".into(),
///     })
///     .await;
///
/// assert!(result.is_err());
/// assert!(matches!(result.unwrap_err(), AuthError::InvalidCredentials));
/// # });
/// ```
#[tokio::test]
async fn test_login_nonexistent_user() {
  let auth = setup_test_auth().await.unwrap();

  let result = auth
    .login(Login {
      email: "nonexistent@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::InvalidCredentials));
}

#[tokio::test]
async fn test_verify_session_success() {
  let auth = setup_test_auth().await.unwrap();

  // Register and login
  let user = auth
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

  // Verify session
  let result = auth.verify(Verify::new(&session.token)).await;

  assert!(result.is_ok());
  let verified_user = result.unwrap();
  assert_eq!(verified_user.id, user.id);
  assert_eq!(verified_user.email, user.email);
}

#[tokio::test]
async fn test_verify_invalid_token() {
  let auth = setup_test_auth().await.unwrap();

  let result = auth.verify(Verify::new("invalid-token")).await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::InvalidSession));
}

/// Verifies that logging out invalidates an existing session token.
///
/// This integration test registers a user, logs in to create a session, asserts the
/// session is initially valid, calls logout for that session token, and then
/// asserts that subsequent verification of the same token fails with `AuthError::InvalidSession`.
///
/// # Examples
///
/// ```
/// # async fn run_test(auth: &crate::Auth) {
/// // register and login
/// auth.register(crate::Register {
///     email: "logout@example.com".into(),
///     password: "SecurePass123".into(),
/// }).await.unwrap();
///
/// let session = auth.login(crate::Login {
///     email: "logout@example.com".into(),
///     password: "SecurePass123".into(),
/// }).await.unwrap();
///
/// // verify exists
/// assert!(auth.verify(crate::Verify::new(&session.token)).await.is_ok());
///
/// // logout and verify invalidation
/// auth.logout(crate::Logout::new(&session.token)).await.unwrap();
/// assert!(matches!(
///     auth.verify(crate::Verify::new(&session.token)).await.unwrap_err(),
///     crate::AuthError::InvalidSession
/// ));
/// # }
/// ```
#[tokio::test]
async fn test_logout_success() {
  let auth = setup_test_auth().await.unwrap();

  // Register and login
  auth
    .register(Register {
      email: "logout@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  let session = auth
    .login(Login {
      email: "logout@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // Verify session exists
  assert!(auth.verify(Verify::new(&session.token)).await.is_ok());

  // Logout
  let result = auth.logout(Logout::new(&session.token)).await;
  assert!(result.is_ok());

  // Verify session no longer exists
  let verify_result = auth.verify(Verify::new(&session.token)).await;
  assert!(verify_result.is_err());
  assert!(matches!(
    verify_result.unwrap_err(),
    AuthError::InvalidSession
  ));
}

#[tokio::test]
async fn test_logout_invalid_token() {
  let auth = setup_test_auth().await.unwrap();

  // Logout with non-existent token should not error
  let result = auth.logout(Logout::new("invalid-token")).await;
  assert!(result.is_ok());
}

#[tokio::test]
async fn test_full_auth_lifecycle() {
  let auth = setup_test_auth().await.unwrap();

  // 1. Register
  let user = auth
    .register(Register {
      email: "lifecycle@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // 2. Login
  let session = auth
    .login(Login {
      email: "lifecycle@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // 3. Verify
  let verified_user = auth.verify(Verify::new(&session.token)).await.unwrap();
  assert_eq!(verified_user.id, user.id);

  // 4. Logout
  auth.logout(Logout::new(&session.token)).await.unwrap();

  // 5. Verify session is invalid
  assert!(auth.verify(Verify::new(&session.token)).await.is_err());
}

#[tokio::test]
async fn test_multiple_sessions_same_user() {
  let auth = setup_test_auth().await.unwrap();

  // Register user
  auth
    .register(Register {
      email: "multi@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // Create multiple sessions
  let session1 = auth
    .login(Login {
      email: "multi@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  let session2 = auth
    .login(Login {
      email: "multi@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // Both sessions should be valid
  assert!(auth.verify(Verify::new(&session1.token)).await.is_ok());
  assert!(auth.verify(Verify::new(&session2.token)).await.is_ok());

  // Logout one session
  auth.logout(Logout::new(&session1.token)).await.unwrap();

  // First session should be invalid, second still valid
  assert!(auth.verify(Verify::new(&session1.token)).await.is_err());
  assert!(auth.verify(Verify::new(&session2.token)).await.is_ok());
}

#[tokio::test]
async fn test_auth_is_clonable() {
  let auth = setup_test_auth().await.unwrap();

  // Clone auth
  let auth_clone = auth.clone();

  // Register with original
  auth
    .register(Register {
      email: "clone@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // Login with clone
  let session = auth_clone
    .login(Login {
      email: "clone@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // Verify with original
  assert!(auth.verify(Verify::new(&session.token)).await.is_ok());
}

#[tokio::test]
async fn test_register_multiple_users() {
  let auth = setup_test_auth().await.unwrap();

  let users = vec![
    ("user1@example.com", "Password123"),
    ("user2@example.com", "Password456"),
    ("user3@example.com", "Password789"),
  ];

  for (email, password) in users {
    let result = auth
      .register(Register {
        email: email.into(),
        password: password.into(),
      })
      .await;
    assert!(result.is_ok());
  }
}

/// Checks that a session token provided as a string slice can be converted and verified.
///
/// This test ensures the `From<&str>` (or `Into`) implementation for the token type is accepted
/// by `verify`, allowing callers to pass `&str` directly.
///
/// # Examples
///
/// ```
/// # async fn run_example(auth: &crate::Auth) {
/// auth.register(crate::Register {
///     email: "test@example.com".into(),
///     password: "SecurePass123".into(),
/// }).await.unwrap();
///
/// let session = auth.login(crate::Login {
///     email: "test@example.com".into(),
///     password: "SecurePass123".into(),
/// }).await.unwrap();
///
/// // Pass a `&str` token to `verify` via `Into`
/// let result = auth.verify(session.token.as_str().into()).await;
/// assert!(result.is_ok());
/// # }
/// ```
#[tokio::test]
async fn test_verify_from_string() {
  let auth = setup_test_auth().await.unwrap();

  auth
    .register(Register {
      email: "test@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  let session = auth
    .login(Login {
      email: "test@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // Test From<&str> implementation
  let result = auth.verify(session.token.as_str().into()).await;
  assert!(result.is_ok());
}

/// Ensures logout succeeds when called with a session token converted from a `&str`.
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// let auth = setup_test_auth().await.unwrap();
/// auth.register(Register {
///     email: "test@example.com".into(),
///     password: "SecurePass123".into(),
/// }).await.unwrap();
///
/// let session = auth.login(Login {
///     email: "test@example.com".into(),
///     password: "SecurePass123".into(),
/// }).await.unwrap();
///
/// let result = auth.logout(session.token.as_str().into()).await;
/// assert!(result.is_ok());
/// # });
/// ```
#[tokio::test]
async fn test_logout_from_string() {
  let auth = setup_test_auth().await.unwrap();

  auth
    .register(Register {
      email: "test@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  let session = auth
    .login(Login {
      email: "test@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // Test From<&str> implementation
  let result = auth.logout(session.token.as_str().into()).await;
  assert!(result.is_ok());
}

#[tokio::test]
async fn test_password_case_sensitivity() {
  let auth = setup_test_auth().await.unwrap();

  auth
    .register(Register {
      email: "case@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // Try to login with different case
  let result = auth
    .login(Login {
      email: "case@example.com".into(),
      password: "securepass123".into(),
    })
    .await;

  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), AuthError::InvalidCredentials));
}

#[tokio::test]
async fn test_email_case_handling() {
  let auth = setup_test_auth().await.unwrap();

  // Register with lowercase
  auth
    .register(Register {
      email: "test@example.com".into(),
      password: "SecurePass123".into(),
    })
    .await
    .unwrap();

  // Try to login with uppercase
  let result = auth
    .login(Login {
      email: "TEST@EXAMPLE.COM".into(),
      password: "SecurePass123".into(),
    })
    .await;

  // This behavior depends on database collation
  // The test documents current behavior
  assert!(result.is_err());
}