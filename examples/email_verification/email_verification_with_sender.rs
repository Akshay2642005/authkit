//! Example: Email Verification with Custom Email Sender
//!
//! This example demonstrates how to use AuthKit with a custom email sender
//! for automatic email verification flow.
//!
//! Run with: cargo run --example email_verification_with_sender --features sqlite

use async_trait::async_trait;
use authkit::email::{EmailContext, EmailSender};
use authkit::prelude::*;

/// A simple email sender that prints emails to the console
/// Perfect for development and testing
struct ConsoleEmailSender {
  base_url: String,
}

#[async_trait]
impl EmailSender for ConsoleEmailSender {
  async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
    println!("\n📧 ============= EMAIL SENT =============");
    println!("To: {}", context.email);
    println!("Subject: Verify your email address");
    println!("---------------------------------------");
    println!("Click the link below to verify your email:");
    println!();
    println!("  {}/verify?token={}", self.base_url, context.token);
    println!();
    println!("Token expires at: {} (Unix timestamp)", context.expires_at);
    println!("=======================================\n");

    Ok(())
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  println!("🔐 AuthKit - Email Verification with Custom Sender Example\n");

  // Create Auth instance with custom email sender
  let auth = Auth::builder()
    .database(Database::sqlite(":memory:").await?)
    .email_sender(Box::new(ConsoleEmailSender {
      base_url: "http://localhost:3000".to_string(),
    }))
    .build()?;

  // Run migrations
  println!("📦 Running database migrations...");
  auth.migrate().await?;
  println!("✅ Migrations complete\n");

  // Step 1: Register a new user
  println!("👤 Registering new user...");
  let user = auth
    .register(Register {
      email: "alice@example.com".into(),
      password: "SecurePass123!".into(),
    })
    .await?;

  println!("✅ User registered:");
  println!("   ID: {}", user.id);
  println!("   Email: {}", user.email);
  println!("   Verified: {}", user.email_verified);

  // Step 2: Send verification email (automatic with EmailSender)
  println!("\n📨 Sending verification email...");
  let verification = auth
    .send_email_verification(SendEmailVerification {
      user_id: user.id.clone(),
    })
    .await?;

  println!("✅ Verification token generated and email sent!");
  println!("   (Token is printed above in the email)");

  // Step 3: Simulate user clicking the verification link
  println!("\n🔗 Simulating user clicking verification link...");
  let verified_user = auth
    .verify_email(VerifyEmail {
      token: verification.token.clone(),
    })
    .await?;

  println!("✅ Email verified successfully!");
  println!("   Email: {}", verified_user.email);
  println!("   Verified: {}", verified_user.email_verified);
  println!(
    "   Verified at: {}",
    verified_user
      .email_verified_at
      .map(|ts| ts.to_string())
      .unwrap_or_else(|| "N/A".to_string())
  );

  // Step 4: Try to verify again (should fail - token already used)
  println!("\n🔄 Attempting to use the same token again...");
  match auth
    .verify_email(VerifyEmail {
      token: verification.token.clone(),
    })
    .await
  {
    Ok(_) => println!("❌ ERROR: Token should not be reusable!"),
    Err(e) => println!("✅ Correctly rejected: {}", e),
  }

  // Step 5: Try to send verification for already verified email
  println!("\n📧 Attempting to send verification for already verified email...");
  match auth
    .send_email_verification(SendEmailVerification {
      user_id: user.id.clone(),
    })
    .await
  {
    Ok(_) => println!("❌ ERROR: Should not send verification for verified email!"),
    Err(e) => println!("✅ Correctly rejected: {}", e),
  }

  // Step 6: Demonstrate resend functionality with a new user
  println!("\n👤 Registering another user for resend demo...");
  let user2 = auth
    .register(Register {
      email: "bob@example.com".into(),
      password: "AnotherSecure123!".into(),
    })
    .await?;

  println!("✅ User registered: {}", user2.email);

  println!("\n📨 Sending initial verification email...");
  let _first_verification = auth
    .send_email_verification(SendEmailVerification {
      user_id: user2.id.clone(),
    })
    .await?;

  println!("\n📨 Resending verification email...");
  let resent_verification = auth
    .resend_email_verification(ResendEmailVerification {
      email: user2.email.clone(),
    })
    .await?;

  println!("✅ Verification email resent!");
  println!("   New token generated (printed in email above)");

  // Verify with the resent token
  println!("\n🔗 Verifying with resent token...");
  let verified_user2 = auth
    .verify_email(VerifyEmail {
      token: resent_verification.token,
    })
    .await?;

  println!("✅ Email verified successfully!");
  println!("   Email: {}", verified_user2.email);
  println!("   Verified: {}", verified_user2.email_verified);

  println!("\n🎉 All operations completed successfully!");
  println!("\n💡 Key Takeaways:");
  println!("   1. EmailSender trait allows custom email sending logic");
  println!("   2. Emails are sent automatically when EmailSender is configured");
  println!("   3. Tokens are single-use and expire after 24 hours");
  println!("   4. Verified emails cannot be re-verified");
  println!("   5. Resend generates new tokens and invalidates old ones");

  Ok(())
}
