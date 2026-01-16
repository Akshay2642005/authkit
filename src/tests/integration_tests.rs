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

/// Helper function to set up a test Auth instance with in-memory database
/// Uses SQLite by default, or Postgres if only postgres feature is enabled
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
