//! Email Verification Example
//!
//! This example demonstrates how to use AuthKit's email verification feature.
//!
//! Run with:
//! ```sh
//! cargo run --example email_verification --features sqlite
//! ```

use authkit::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
  println!("=== AuthKit Email Verification Example ===\n");

  // 1. Initialize AuthKit with SQLite
  println!("1. Initializing AuthKit...");
  let auth = Auth::builder()
    .database(Database::sqlite(":memory:").await?)
    .build()?;

  // Run migrations
  auth.migrate().await?;
  println!("   ✓ Database initialized\n");

  // 2. Register a new user
  println!("2. Registering a new user...");
  let user = auth
    .register(Register {
      email: "alice@example.com".to_string(),
      password: "SecurePassword123!".to_string(),
    })
    .await?;

  println!("   ✓ User registered:");
  println!("     - Email: {}", user.email);
  println!("     - ID: {}", user.id);
  println!("     - Email Verified: {}", user.email_verified);
  println!("     - Created At: {}\n", user.created_at);

  // 3. User can login even without email verification
  println!("3. Logging in (email verification not required)...");
  let session = auth
    .login(Login {
      email: "alice@example.com".to_string(),
      password: "SecurePassword123!".to_string(),
    })
    .await?;
  println!("   ✓ Login successful!");
  println!("     - Session Token: {}...\n", &session.token[..16]);

  // 4. Send email verification token
  println!("4. Generating email verification token...");
  let verification_token = auth
    .send_email_verification(SendEmailVerification {
      user_id: user.id.clone(),
    })
    .await?;

  println!("   ✓ Verification token generated:");
  println!("     - Token: {}...", &verification_token.token[..16]);
  println!("     - For Email: {}", verification_token.email);
  println!("     - Expires At: {}", verification_token.expires_at);
  println!("\n   📧 In a real application, you would send this token via email.");
  println!("      For example:");
  println!(
    "      https://yourapp.com/verify-email?token={}\n",
    verification_token.token
  );

  // 5. Verify the email using the token
  println!("5. Verifying email with token...");
  let verified_user = auth
    .verify_email(VerifyEmail {
      token: verification_token.token.clone(),
    })
    .await?;

  println!("   ✓ Email verified successfully!");
  println!("     - Email Verified: {}", verified_user.email_verified);
  println!(
    "     - Verified At: {:?}\n",
    verified_user.email_verified_at
  );

  // 6. Try to use the same token again (should fail)
  println!("6. Attempting to reuse the same token...");
  match auth
    .verify_email(VerifyEmail {
      token: verification_token.token,
    })
    .await
  {
    Ok(_) => println!("   ✗ Unexpectedly succeeded!"),
    Err(e) => println!("   ✓ Correctly rejected: {}\n", e),
  }

  // 7. Try to send verification for already verified email (should fail)
  println!("7. Attempting to send verification for already verified email...");
  match auth
    .send_email_verification(SendEmailVerification {
      user_id: user.id.clone(),
    })
    .await
  {
    Ok(_) => println!("   ✗ Unexpectedly succeeded!"),
    Err(e) => println!("   ✓ Correctly rejected: {}\n", e),
  }

  // 8. Demonstrate resend verification for unverified user
  println!("8. Demonstrating resend for a new unverified user...");
  let user2 = auth
    .register(Register {
      email: "bob@example.com".to_string(),
      password: "AnotherSecure123!".to_string(),
    })
    .await?;

  println!("   ✓ Registered: {}", user2.email);

  // Send first verification
  let first_token = auth
    .send_email_verification(SendEmailVerification {
      user_id: user2.id.clone(),
    })
    .await?;
  println!(
    "   ✓ First verification token: {}...",
    &first_token.token[..16]
  );

  // Resend verification (simulating user didn't receive first email)
  let resent_token = auth
    .resend_email_verification(ResendEmailVerification {
      email: "bob@example.com".to_string(),
    })
    .await?;

  println!(
    "   ✓ Resent verification token: {}...",
    &resent_token.token[..16]
  );
  println!("   ℹ Note: Each token is unique and can only be used once\n");

  // 9. Verify with the resent token
  println!("9. Verifying with the resent token...");
  let verified_user2 = auth
    .verify_email(VerifyEmail {
      token: resent_token.token,
    })
    .await?;

  println!("   ✓ Email verified: {}", verified_user2.email);
  println!("   ✓ Verified status: {}\n", verified_user2.email_verified);

  // 10. Session remains valid after verification
  println!("10. Verifying that original session is still valid...");
  let session_user = auth
    .verify(Verify {
      token: session.token.clone(),
    })
    .await?;

  println!("   ✓ Session valid!");
  println!(
    "   ✓ User email verified status: {}\n",
    session_user.email_verified
  );

  // 11. Cleanup
  println!("11. Cleaning up...");
  auth
    .logout(Logout {
      token: session.token,
    })
    .await?;
  println!("   ✓ Logged out\n");

  println!("=== Example Complete ===");
  println!("\nKey Takeaways:");
  println!("• Users can register and login without email verification");
  println!("• Email verification is optional but recommended for security");
  println!("• Verification tokens are single-use and time-limited (24 hours)");
  println!("• Tokens can be resent if users don't receive the email");
  println!("• Once verified, users cannot be sent new verification tokens");
  println!("• Application is responsible for sending emails (AuthKit only generates tokens)");

  Ok(())
}
