//! Example: Custom Email Sender Implementation
//!
//! This example demonstrates how to implement a custom email sender for AuthKit.
//! You can use any email service you prefer: SendGrid, AWS SES, SMTP, Resend, etc.

use async_trait::async_trait;
use authkit::email::{EmailContext, EmailSender};
use authkit::prelude::*;

// ============================================================================
// Example 1: Simple Console Email Sender (for testing)
// ============================================================================

/// A simple email sender that prints emails to the console
/// Perfect for development and testing
struct ConsoleEmailSender {
  base_url: String,
}

#[async_trait]
impl EmailSender for ConsoleEmailSender {
  async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
    println!("📧 ============= EMAIL =============");
    println!("To: {}", context.email);
    println!("Subject: Verify your email address");
    println!("-----------------------------------");
    println!("Click the link below to verify your email:");
    println!();
    println!("  {}/verify?token={}", self.base_url, context.token);
    println!();
    println!("This link expires at: {}", context.expires_at);
    println!("===================================\n");

    Ok(())
  }
}

// ============================================================================
// Example 2: SendGrid Email Sender (production-ready)
// ============================================================================

/// Email sender using SendGrid API
/// Requires: sendgrid crate and API key
struct SendGridEmailSender {
  api_key: String,
  from_email: String,
  from_name: String,
  base_url: String,
}

#[async_trait]
impl EmailSender for SendGridEmailSender {
  async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
    let verification_url = format!("{}/verify?token={}", self.base_url, context.token);

    // In a real implementation, you would:
    // 1. Use the sendgrid crate or make HTTP requests
    // 2. Create a proper HTML email template
    // 3. Handle errors appropriately

    println!("📧 Sending email via SendGrid to: {}", context.email);
    println!("   Verification URL: {}", verification_url);

    // Example pseudo-code (requires sendgrid crate):
    /*
    use sendgrid::v3::*;

    let message = Message::new(Email::new(&self.from_email))
        .set_subject("Verify your email address")
        .add_content(
            Content::new()
                .set_content_type("text/html")
                .set_value(&format!(
                    r#"
                    <html>
                        <body>
                            <h1>Welcome!</h1>
                            <p>Click the button below to verify your email:</p>
                            <a href="{}" style="background: blue; color: white; padding: 10px;">
                                Verify Email
                            </a>
                        </body>
                    </html>
                    "#,
                    verification_url
                ))
        )
        .add_personalization(
            Personalization::new(Email::new(&context.email))
        );

    let sender = Sender::new(&self.api_key);
    sender.send(&message).await
        .map_err(|e| AuthError::EmailSendFailed(e.to_string()))?;
    */

    Ok(())
  }
}

// ============================================================================
// Example 3: SMTP Email Sender (self-hosted)
// ============================================================================

/// Email sender using SMTP
/// Requires: lettre crate
struct SmtpEmailSender {
  smtp_host: String,
  smtp_port: u16,
  smtp_username: String,
  smtp_password: String,
  from_email: String,
  base_url: String,
}

#[async_trait]
impl EmailSender for SmtpEmailSender {
  async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
    let verification_url = format!("{}/verify?token={}", self.base_url, context.token);

    println!("📧 Sending email via SMTP to: {}", context.email);
    println!("   SMTP Host: {}:{}", self.smtp_host, self.smtp_port);

    // Example pseudo-code (requires lettre crate):
    /*
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};

    let email = Message::builder()
        .from(self.from_email.parse().unwrap())
        .to(context.email.parse().unwrap())
        .subject("Verify your email address")
        .body(format!(
            "Click here to verify your email: {}",
            verification_url
        ))
        .unwrap();

    let creds = Credentials::new(
        self.smtp_username.clone(),
        self.smtp_password.clone(),
    );

    let mailer = SmtpTransport::relay(&self.smtp_host)
        .unwrap()
        .credentials(creds)
        .build();

    mailer.send(&email)
        .map_err(|e| AuthError::EmailSendFailed(e.to_string()))?;
    */

    Ok(())
  }
}

// ============================================================================
// Example 4: AWS SES Email Sender
// ============================================================================

/// Email sender using AWS Simple Email Service
/// Requires: aws-sdk-sesv2 crate
struct AwsSesEmailSender {
  client: String, // In reality: aws_sdk_sesv2::Client
  from_email: String,
  base_url: String,
}

#[async_trait]
impl EmailSender for AwsSesEmailSender {
  async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
    let verification_url = format!("{}/verify?token={}", self.base_url, context.token);

    println!("📧 Sending email via AWS SES to: {}", context.email);

    // Example pseudo-code (requires aws-sdk-sesv2):
    /*
    use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};

    let dest = Destination::builder()
        .to_addresses(&context.email)
        .build();

    let subject = Content::builder()
        .data("Verify your email address")
        .build()
        .unwrap();

    let body_content = Content::builder()
        .data(format!(
            "Click here to verify your email: {}",
            verification_url
        ))
        .build()
        .unwrap();

    let body = Body::builder()
        .text(body_content)
        .build();

    let msg = Message::builder()
        .subject(subject)
        .body(body)
        .build();

    let email_content = EmailContent::builder()
        .simple(msg)
        .build();

    self.client
        .send_email()
        .from_email_address(&self.from_email)
        .destination(dest)
        .content(email_content)
        .send()
        .await
        .map_err(|e| AuthError::EmailSendFailed(e.to_string()))?;
    */

    Ok(())
  }
}

// ============================================================================
// Usage Examples
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
  println!("AuthKit Email Sender Examples\n");

  // Example 1: Without custom email sender (manual handling)
  println!("=== Example 1: Manual Email Handling ===\n");
  {
    let auth = Auth::builder()
      .database(Database::sqlite(":memory:").await?)
      .build()?;

    auth.migrate().await?;

    // Register user
    let user = auth
      .register(Register {
        email: "user@example.com".into(),
        password: "secure-password".into(),
      })
      .await?;

    // Send verification - token is returned, no email sent automatically
    let verification = auth
      .send_email_verification(SendEmailVerification {
        user_id: user.id.clone(),
      })
      .await?;

    println!("✅ Token generated: {}", verification.token);
    println!("   You handle email sending manually\n");
  }

  // Example 2: With ConsoleEmailSender (development)
  println!("=== Example 2: Console Email Sender ===\n");
  {
    let auth = Auth::builder()
      .database(Database::sqlite(":memory:").await?)
      .email_sender(Box::new(ConsoleEmailSender {
        base_url: "http://localhost:3000".to_string(),
      }))
      .build()?;

    auth.migrate().await?;

    let user = auth
      .register(Register {
        email: "dev@example.com".into(),
        password: "secure-password".into(),
      })
      .await?;

    // Send verification - email is automatically "sent" to console
    let _verification = auth
      .send_email_verification(SendEmailVerification {
        user_id: user.id.clone(),
      })
      .await?;

    println!("✅ Email printed to console\n");
  }

  // Example 3: With SendGrid (production)
  println!("=== Example 3: SendGrid Email Sender ===\n");
  {
    let auth = Auth::builder()
      .database(Database::sqlite(":memory:").await?)
      .email_sender(Box::new(SendGridEmailSender {
        api_key: std::env::var("SENDGRID_API_KEY").unwrap_or_else(|_| "sg_test_key".to_string()),
        from_email: "noreply@myapp.com".to_string(),
        from_name: "MyApp".to_string(),
        base_url: "https://myapp.com".to_string(),
      }))
      .build()?;

    auth.migrate().await?;

    let user = auth
      .register(Register {
        email: "prod@example.com".into(),
        password: "secure-password".into(),
      })
      .await?;

    let _verification = auth
      .send_email_verification(SendEmailVerification {
        user_id: user.id.clone(),
      })
      .await?;

    println!("✅ Email sent via SendGrid\n");
  }

  // Example 4: Resending verification
  println!("=== Example 4: Resend Verification ===\n");
  {
    let auth = Auth::builder()
      .database(Database::sqlite(":memory:").await?)
      .email_sender(Box::new(ConsoleEmailSender {
        base_url: "http://localhost:3000".to_string(),
      }))
      .build()?;

    auth.migrate().await?;

    auth
      .register(Register {
        email: "resend@example.com".into(),
        password: "secure-password".into(),
      })
      .await?;

    // Resend verification email
    let _verification = auth
      .resend_email_verification(ResendEmailVerification {
        email: "resend@example.com".into(),
      })
      .await?;

    println!("✅ Verification email resent\n");
  }

  println!("All examples completed successfully!");
  Ok(())
}

// ============================================================================
// Integration with Web Frameworks
// ============================================================================

/*
Example with Axum:

use axum::{extract::State, Json};

#[derive(Clone)]
struct AppState {
    auth: Auth,
}

async fn register_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    // Register user
    let user = state.auth.register(Register {
        email: req.email.clone(),
        password: req.password,
    }).await?;

    // Send verification email (automatically sent if EmailSender is configured)
    state.auth.send_email_verification(SendEmailVerification {
        user_id: user.id,
    }).await?;

    Ok(Json(RegisterResponse {
        message: "Registration successful. Please check your email.".into(),
    }))
}

// Build the Auth instance with email sender
let auth = Auth::builder()
    .database(Database::postgres(&database_url).await?)
    .email_sender(Box::new(SendGridEmailSender {
        api_key: env::var("SENDGRID_API_KEY")?,
        from_email: "noreply@myapp.com".into(),
        from_name: "MyApp".into(),
        base_url: "https://myapp.com".into(),
    }))
    .build()?;
*/
