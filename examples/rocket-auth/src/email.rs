//! SMTP Email Sender Implementation
//!
//! This module provides an SMTP-based email sender using the lettre crate.
//! It implements the AuthKit EmailSender trait for sending verification emails.

use async_trait::async_trait;
use authkit::email::{EmailContext, EmailSender};
use authkit::{AuthError, Result};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

/// SMTP Email Sender
///
/// Sends verification emails using SMTP protocol.
/// Supports common providers like Gmail, Outlook, SendGrid, etc.
pub struct SmtpEmailSender {
  smtp_host: String,
  smtp_port: u16,
  smtp_username: String,
  smtp_password: String,
  from_address: String,
  app_url: String,
}

impl SmtpEmailSender {
  /// Create a new SMTP email sender
  ///
  /// # Arguments
  ///
  /// * `smtp_host` - SMTP server hostname (e.g., "smtp.gmail.com")
  /// * `smtp_port` - SMTP server port (typically 587 for STARTTLS, 465 for SSL)
  /// * `smtp_username` - SMTP authentication username (usually your email)
  /// * `smtp_password` - SMTP authentication password (app password for Gmail)
  /// * `from_address` - Email address to send from
  /// * `app_url` - Base URL of your application (for verification links)
  pub fn new(
    smtp_host: String,
    smtp_port: u16,
    smtp_username: String,
    smtp_password: String,
    from_address: String,
    app_url: String,
  ) -> Self {
    Self {
      smtp_host,
      smtp_port,
      smtp_username,
      smtp_password,
      from_address,
      app_url,
    }
  }

  /// Build the HTML email body
  fn build_html_body(&self, token: &str, expires_at: i64) -> String {
    let verification_url = format!("{}/email/verify?token={}", self.app_url, token);

    // Convert Unix timestamp to readable format
    let expiry_time = chrono::DateTime::from_timestamp(expires_at, 0)
      .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
      .unwrap_or_else(|| "24 hours".to_string());

    format!(
      r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Verify Your Email</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 30px; text-align: center; border-radius: 10px 10px 0 0;">
        <h1 style="color: white; margin: 0; font-size: 28px;">🔐 Email Verification</h1>
    </div>

    <div style="background: #f9f9f9; padding: 30px; border-radius: 0 0 10px 10px; border: 1px solid #e0e0e0;">
        <h2 style="color: #333; margin-top: 0;">Hello!</h2>

        <p style="font-size: 16px;">Thank you for registering with us. To complete your registration, please verify your email address by clicking the button below:</p>

        <div style="text-align: center; margin: 30px 0;">
            <a href="{}" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 15px 40px; text-decoration: none; border-radius: 5px; font-size: 16px; font-weight: bold; display: inline-block;">
                Verify Email Address
            </a>
        </div>

        <p style="font-size: 14px; color: #666;">Or copy and paste this link into your browser:</p>
        <p style="background: #fff; padding: 10px; border: 1px solid #ddd; border-radius: 5px; word-break: break-all; font-size: 12px; font-family: monospace;">
            {}
        </p>

        <hr style="border: none; border-top: 1px solid #e0e0e0; margin: 30px 0;">

        <p style="font-size: 14px; color: #666;">
            <strong>⏰ This link will expire at:</strong><br>
            {}
        </p>

        <p style="font-size: 14px; color: #666;">
            If you didn't create an account with us, you can safely ignore this email.
        </p>

        <div style="margin-top: 30px; padding-top: 20px; border-top: 1px solid #e0e0e0; font-size: 12px; color: #999; text-align: center;">
            <p>This is an automated email, please do not reply.</p>
            <p>© 2024 AuthKit Example. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
"#,
      verification_url, verification_url, expiry_time
    )
  }

  /// Build the plain text email body (fallback)
  #[allow(dead_code)]
  fn build_text_body(&self, token: &str, expires_at: i64) -> String {
    let verification_url = format!("{}/email/verify?token={}", self.app_url, token);

    let expiry_time = chrono::DateTime::from_timestamp(expires_at, 0)
      .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
      .unwrap_or_else(|| "24 hours".to_string());

    format!(
      r#"
Email Verification Required

Hello!

Thank you for registering with us. To complete your registration, please verify your email address by clicking the link below:

{}

This link will expire at: {}

If you didn't create an account with us, you can safely ignore this email.

---
This is an automated email, please do not reply.
© 2024 AuthKit Example. All rights reserved.
"#,
      verification_url, expiry_time
    )
  }
}

#[async_trait]
impl EmailSender for SmtpEmailSender {
  async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
    println!("🔧 SMTP Configuration:");
    println!("   Host: {}:{}", self.smtp_host, self.smtp_port);
    println!("   Username: {}", self.smtp_username);
    println!("   From: {}", self.from_address);
    println!("   To: {}", context.email);

    // Build email message
    let email = Message::builder()
      .from(self.from_address.parse().map_err(|e| {
        AuthError::EmailSendFailed(format!(
          "Invalid from address '{}': {}",
          self.from_address, e
        ))
      })?)
      .to(context.email.parse().map_err(|e| {
        AuthError::EmailSendFailed(format!(
          "Invalid recipient address '{}': {}",
          context.email, e
        ))
      })?)
      .subject("Verify Your Email Address")
      .header(ContentType::TEXT_HTML)
      .body(self.build_html_body(&context.token, context.expires_at))
      .map_err(|e| AuthError::EmailSendFailed(format!("Failed to build email: {}", e)))?;

    println!("✅ Email message built successfully");

    // Create SMTP transport with STARTTLS
    let creds = Credentials::new(self.smtp_username.clone(), self.smtp_password.clone());

    let mailer = SmtpTransport::starttls_relay(&self.smtp_host)
      .map_err(|e| {
        AuthError::EmailSendFailed(format!(
          "Failed to create SMTP relay for '{}': {}. Check SMTP_HOST.",
          self.smtp_host, e
        ))
      })?
      .port(self.smtp_port)
      .credentials(creds)
      .build();

    println!("✅ SMTP transport created");

    // Send email
    mailer.send(&email).map_err(|e| {
      eprintln!("❌ SMTP Send Error: {}", e);
      eprintln!("   This usually means:");
      eprintln!("   1. Invalid SMTP credentials (check SMTP_USERNAME and SMTP_PASSWORD)");
      eprintln!("   2. For Gmail: Use an App Password, not your regular password");
      eprintln!("   3. Wrong SMTP host/port (check SMTP_HOST and SMTP_PORT)");
      eprintln!("   4. Firewall blocking the connection");
      AuthError::EmailSendFailed(format!("Failed to send email: {}", e))
    })?;

    println!("📧 Verification email sent to: {}", context.email);
    println!("   Token: {}", context.token);
    println!(
      "   Expires: {}",
      chrono::DateTime::from_timestamp(context.expires_at, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string())
    );

    Ok(())
  }
}
