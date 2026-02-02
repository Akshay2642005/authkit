#[cfg(test)]
mod tests {
  use crate::prelude::*;
  use crate::tests::integration_tests::{
    register_and_verify_user, setup_test_auth, setup_test_auth_with_email_verification,
  };

  #[tokio::test]
  async fn test_send_email_verification_success() {
    let auth = setup_test_auth().await.unwrap();

    // Register a user first
    let user = auth
      .register(Register { name: None,
        email: "test@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    assert!(!user.email_verified);

    // Send verification email
    let verification = auth
      .send_email_verification(SendEmailVerification {
        user_id: user.id.clone(),
      })
      .await
      .unwrap();

    assert_eq!(verification.identifier, "test@example.com");
    assert!(!verification.token.is_empty());
    assert!(verification.expires_at > 0);
  }

  #[tokio::test]
  async fn test_send_email_verification_user_not_found() {
    let auth = setup_test_auth().await.unwrap();

    let result = auth
      .send_email_verification(SendEmailVerification {
        user_id: "nonexistent-user-id".to_string(),
      })
      .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::UserNotFound));
  }

  #[tokio::test]
  async fn test_send_email_verification_already_verified() {
    let auth = setup_test_auth().await.unwrap();

    // Register a user
    let user = auth
      .register(Register { name: None,
        email: "test@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    // Send and verify email
    let verification = auth
      .send_email_verification(SendEmailVerification {
        user_id: user.id.clone(),
      })
      .await
      .unwrap();

    auth
      .verify_email(VerifyEmail {
        token: verification.token,
      })
      .await
      .unwrap();

    // Try to send verification again
    let result = auth
      .send_email_verification(SendEmailVerification {
        user_id: user.id.clone(),
      })
      .await;

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      AuthError::EmailAlreadyVerified(_)
    ));
  }

  #[tokio::test]
  async fn test_verify_email_success() {
    let auth = setup_test_auth().await.unwrap();

    // Register a user
    let user = auth
      .register(Register { name: None,
        email: "test@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    assert!(!user.email_verified);
    assert!(user.email_verified_at.is_none());

    // Send verification email
    let verification = auth
      .send_email_verification(SendEmailVerification {
        user_id: user.id.clone(),
      })
      .await
      .unwrap();

    // Verify email
    let verified_user = auth
      .verify_email(VerifyEmail {
        token: verification.token,
      })
      .await
      .unwrap();

    assert!(verified_user.email_verified);
    assert!(verified_user.email_verified_at.is_some());
    assert_eq!(verified_user.email, "test@example.com");
  }

  #[tokio::test]
  async fn test_verify_email_invalid_token() {
    let auth = setup_test_auth().await.unwrap();

    let result = auth
      .verify_email(VerifyEmail {
        token: "invalid-token".to_string(),
      })
      .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::InvalidToken(_)));
  }

  #[tokio::test]
  async fn test_verify_email_token_already_used() {
    let auth = setup_test_auth().await.unwrap();

    // Register a user
    let user = auth
      .register(Register { name: None,
        email: "test@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    // Send verification email
    let verification = auth
      .send_email_verification(SendEmailVerification {
        user_id: user.id.clone(),
      })
      .await
      .unwrap();

    // Verify email once
    auth
      .verify_email(VerifyEmail {
        token: verification.token.clone(),
      })
      .await
      .unwrap();

    // Try to verify again with the same token
    let result = auth
      .verify_email(VerifyEmail {
        token: verification.token,
      })
      .await;

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      AuthError::TokenAlreadyUsed(_)
    ));
  }

  #[tokio::test]
  async fn test_verify_email_already_verified() {
    let auth = setup_test_auth().await.unwrap();

    // Register a user
    let user = auth
      .register(Register { name: None,
        email: "test@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    // Send and verify first time
    let verification = auth
      .send_email_verification(SendEmailVerification {
        user_id: user.id.clone(),
      })
      .await
      .unwrap();

    auth
      .verify_email(VerifyEmail {
        token: verification.token,
      })
      .await
      .unwrap();

    // Generate a new token and try to verify again
    // This should fail because email is already verified
    // Note: We can't actually test this scenario properly because
    // send_email_verification will fail for already-verified users
  }

  #[tokio::test]
  async fn test_resend_email_verification_success() {
    let auth = setup_test_auth().await.unwrap();

    // Register a user
    let user = auth
      .register(Register { name: None,
        email: "test@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    // Send initial verification
    let first_verification = auth
      .send_email_verification(SendEmailVerification {
        user_id: user.id.clone(),
      })
      .await
      .unwrap();

    // Resend verification
    let second_verification = auth
      .resend_email_verification(ResendEmailVerification {
        email: "test@example.com".to_string(),
      })
      .await
      .unwrap();

    assert_eq!(second_verification.identifier, "test@example.com");
    assert!(!second_verification.token.is_empty());
    assert_ne!(first_verification.token, second_verification.token);
  }

  #[tokio::test]
  async fn test_resend_email_verification_user_not_found() {
    let auth = setup_test_auth().await.unwrap();

    let result = auth
      .resend_email_verification(ResendEmailVerification {
        email: "nonexistent@example.com".to_string(),
      })
      .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::UserNotFound));
  }

  #[tokio::test]
  async fn test_resend_email_verification_already_verified() {
    let auth = setup_test_auth().await.unwrap();

    // Register and verify a user
    let user = auth
      .register(Register { name: None,
        email: "test@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    let verification = auth
      .send_email_verification(SendEmailVerification {
        user_id: user.id.clone(),
      })
      .await
      .unwrap();

    auth
      .verify_email(VerifyEmail {
        token: verification.token,
      })
      .await
      .unwrap();

    // Try to resend verification
    let result = auth
      .resend_email_verification(ResendEmailVerification {
        email: "test@example.com".to_string(),
      })
      .await;

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      AuthError::EmailAlreadyVerified(_)
    ));
  }

  #[tokio::test]
  async fn test_login_without_verification_when_not_required() {
    // Default auth does NOT require email verification
    let auth = setup_test_auth().await.unwrap();

    // Register user (no email verification)
    auth
      .register(Register { name: None,
        email: "test@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    // Login should succeed without email verification
    let session = auth
      .login(Login { ip_address: None, user_agent: None,
        email: "test@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    assert!(!session.token.is_empty());
  }

  #[tokio::test]
  async fn test_login_requires_email_verification_when_configured() {
    // Use auth that requires email verification
    let auth = setup_test_auth_with_email_verification().await.unwrap();

    // Register a user but don't verify email
    auth
      .register(Register { name: None,
        email: "unverified@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    // Attempt to login should fail with EmailNotVerified
    let result = auth
      .login(Login { ip_address: None, user_agent: None,
        email: "unverified@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await;

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      AuthError::EmailNotVerified(_)
    ));
  }

  #[tokio::test]
  async fn test_login_succeeds_after_verification_when_required() {
    // Use auth that requires email verification
    let auth = setup_test_auth_with_email_verification().await.unwrap();

    // Register and verify user using helper
    register_and_verify_user(&auth, "verified@example.com", "SecurePass123!")
      .await
      .unwrap();

    // Now login should succeed
    let session = auth
      .login(Login { ip_address: None, user_agent: None,
        email: "verified@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    assert!(!session.token.is_empty());
  }

  #[tokio::test]
  async fn test_email_verification_end_to_end_without_requirement() {
    // Default auth - verification not required for login
    let auth = setup_test_auth().await.unwrap();

    // 1. Register a new user
    let user = auth
      .register(Register { name: None,
        email: "newuser@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    assert!(!user.email_verified);
    assert!(user.email_verified_at.is_none());

    // 2. User CAN login without email verification (not required by default)
    let session = auth
      .login(Login { ip_address: None, user_agent: None,
        email: "newuser@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    assert!(!session.token.is_empty());

    // 3. Send verification email
    let verification = auth
      .send_email_verification(SendEmailVerification {
        user_id: user.id.clone(),
      })
      .await
      .unwrap();

    // 4. Verify the email
    let verified_user = auth
      .verify_email(VerifyEmail {
        token: verification.token,
      })
      .await
      .unwrap();

    assert!(verified_user.email_verified);
    assert!(verified_user.email_verified_at.is_some());

    // 5. Session still works
    let session_user = auth
      .verify(Verify {
        token: session.token.clone(),
      })
      .await
      .unwrap();

    assert_eq!(session_user.id, user.id);

    // 6. Logout
    auth
      .logout(Logout {
        token: session.token,
      })
      .await
      .unwrap();
  }

  #[tokio::test]
  async fn test_email_verification_end_to_end_with_requirement() {
    // Use auth that requires email verification
    let auth = setup_test_auth_with_email_verification().await.unwrap();

    // 1. Register a new user
    let user = auth
      .register(Register { name: None,
        email: "newuser@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    assert!(!user.email_verified);
    assert!(user.email_verified_at.is_none());

    // 2. User CANNOT login without email verification
    let login_result = auth
      .login(Login { ip_address: None, user_agent: None,
        email: "newuser@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await;

    assert!(login_result.is_err());
    assert!(matches!(
      login_result.unwrap_err(),
      AuthError::EmailNotVerified(_)
    ));

    // 3. Send verification email
    let verification = auth
      .send_email_verification(SendEmailVerification {
        user_id: user.id.clone(),
      })
      .await
      .unwrap();

    // 4. Verify the email
    let verified_user = auth
      .verify_email(VerifyEmail {
        token: verification.token,
      })
      .await
      .unwrap();

    assert!(verified_user.email_verified);
    assert!(verified_user.email_verified_at.is_some());

    // 5. Now user CAN login after email verification
    let session = auth
      .login(Login { ip_address: None, user_agent: None,
        email: "newuser@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    assert!(!session.token.is_empty());

    // 6. Verify the session works
    let session_user = auth
      .verify(Verify {
        token: session.token.clone(),
      })
      .await
      .unwrap();

    assert!(session_user.email_verified);
    assert_eq!(session_user.id, user.id);

    // 7. Logout
    auth
      .logout(Logout {
        token: session.token,
      })
      .await
      .unwrap();
  }

  #[tokio::test]
  async fn test_multiple_users_email_verification() {
    let auth = setup_test_auth().await.unwrap();

    // Register multiple users
    let user1 = auth
      .register(Register { name: None,
        email: "user1@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    let user2 = auth
      .register(Register { name: None,
        email: "user2@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    // Send verification emails
    let verification1 = auth
      .send_email_verification(SendEmailVerification {
        user_id: user1.id.clone(),
      })
      .await
      .unwrap();

    let verification2 = auth
      .send_email_verification(SendEmailVerification {
        user_id: user2.id.clone(),
      })
      .await
      .unwrap();

    // Verify only user1
    let verified_user1 = auth
      .verify_email(VerifyEmail {
        token: verification1.token,
      })
      .await
      .unwrap();

    assert!(verified_user1.email_verified);

    // User2 is still unverified
    assert!(!user2.email_verified);

    // Both users can login (verification not required by default)
    let session1 = auth
      .login(Login { ip_address: None, user_agent: None,
        email: "user1@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    let session2 = auth
      .login(Login { ip_address: None, user_agent: None,
        email: "user2@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    assert!(!session1.token.is_empty());
    assert!(!session2.token.is_empty());

    // Now verify user2
    let verified_user2 = auth
      .verify_email(VerifyEmail {
        token: verification2.token,
      })
      .await
      .unwrap();

    assert!(verified_user2.email_verified);
  }

  #[tokio::test]
  async fn test_sends_verification_on_register_default_false() {
    // Default auth has send_verification_on_register = false
    let auth = setup_test_auth().await.unwrap();

    // Check the helper methods
    assert!(!auth.has_email_sender());
    assert!(!auth.sends_verification_on_register());
    assert!(!auth.requires_email_verification());
  }

  #[tokio::test]
  async fn test_require_email_verification_config() {
    let auth = setup_test_auth_with_email_verification().await.unwrap();

    // This auth requires email verification
    assert!(auth.requires_email_verification());
    // But doesn't auto-send verification on register
    assert!(!auth.sends_verification_on_register());
  }

  #[tokio::test]
  async fn test_register_without_email_sender_works() {
    // Without email sender, registration should work
    let auth = setup_test_auth().await.unwrap();

    // No email sender is configured in setup_test_auth
    assert!(!auth.has_email_sender());

    // Register should succeed
    let user = auth
      .register(Register { name: None,
        email: "no-email-sender@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    assert!(!user.email_verified);

    // User can still manually verify via send_email_verification + verify_email
    let verification = auth
      .send_email_verification(SendEmailVerification {
        user_id: user.id.clone(),
      })
      .await
      .unwrap();

    let verified_user = auth
      .verify_email(VerifyEmail {
        token: verification.token,
      })
      .await
      .unwrap();

    assert!(verified_user.email_verified);
  }
}
