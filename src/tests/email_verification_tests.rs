#[cfg(test)]
mod tests {
  use crate::prelude::*;

  async fn setup_auth() -> Result<Auth> {
    let auth = Auth::builder()
      .database(Database::sqlite(":memory:").await?)
      .build()?;
    auth.migrate().await?;
    Ok(auth)
  }

  #[tokio::test]
  async fn test_send_email_verification_success() {
    let auth = setup_auth().await.unwrap();

    // Register a user first
    let user = auth
      .register(Register {
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

    assert_eq!(verification.email, "test@example.com");
    assert!(!verification.token.is_empty());
    assert!(verification.expires_at > 0);
  }

  #[tokio::test]
  async fn test_send_email_verification_user_not_found() {
    let auth = setup_auth().await.unwrap();

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
    let auth = setup_auth().await.unwrap();

    // Register a user
    let user = auth
      .register(Register {
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
    let auth = setup_auth().await.unwrap();

    // Register a user
    let user = auth
      .register(Register {
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
    let auth = setup_auth().await.unwrap();

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
    let auth = setup_auth().await.unwrap();

    // Register a user
    let user = auth
      .register(Register {
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
    let auth = setup_auth().await.unwrap();

    // Register a user
    let user = auth
      .register(Register {
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
    let auth = setup_auth().await.unwrap();

    // Register a user
    let user = auth
      .register(Register {
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

    assert_eq!(second_verification.email, "test@example.com");
    assert!(!second_verification.token.is_empty());
    assert_ne!(first_verification.token, second_verification.token);
  }

  #[tokio::test]
  async fn test_resend_email_verification_user_not_found() {
    let auth = setup_auth().await.unwrap();

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
    let auth = setup_auth().await.unwrap();

    // Register and verify a user
    let user = auth
      .register(Register {
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
  async fn test_email_verification_end_to_end() {
    let auth = setup_auth().await.unwrap();

    // 1. Register a new user
    let user = auth
      .register(Register {
        email: "newuser@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    assert!(!user.email_verified);
    assert!(user.email_verified_at.is_none());

    // 2. User can login without verification
    let session = auth
      .login(Login {
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

    // 5. Verify that the session still works
    let session_user = auth
      .verify(Verify {
        token: session.token.clone(),
      })
      .await
      .unwrap();

    assert!(session_user.email_verified);
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
  async fn test_multiple_users_email_verification() {
    let auth = setup_auth().await.unwrap();

    // Register multiple users
    let user1 = auth
      .register(Register {
        email: "user1@example.com".to_string(),
        password: "SecurePass123!".to_string(),
      })
      .await
      .unwrap();

    let user2 = auth
      .register(Register {
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

    // User2 should still be unverified
    let user2_check = auth
      .verify(Verify {
        token: auth
          .login(Login {
            email: "user2@example.com".to_string(),
            password: "SecurePass123!".to_string(),
          })
          .await
          .unwrap()
          .token,
      })
      .await
      .unwrap();

    assert!(!user2_check.email_verified);

    // Now verify user2
    let verified_user2 = auth
      .verify_email(VerifyEmail {
        token: verification2.token,
      })
      .await
      .unwrap();

    assert!(verified_user2.email_verified);
  }
}
