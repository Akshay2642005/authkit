use crate::error::Result;
use async_trait::async_trait;

/// Context provided to email senders containing information about the email to send
#[derive(Debug, Clone)]
pub struct EmailContext {
  /// The recipient's email address
  pub email: String,
  /// The verification token (plaintext)
  pub token: String,
  /// When the token expires (Unix timestamp)
  pub expires_at: i64,
}

/// Trait for sending verification emails
///
/// Implement this trait to provide your own email sending logic.
/// AuthKit will call this after generating a verification token.
///
/// # Example
///
/// ```rust,ignore
/// use authkit::email::{EmailSender, EmailContext};
/// use authkit::error::Result;
/// use async_trait::async_trait;
///
/// struct MyEmailSender {
///     api_key: String,
/// }
///
/// #[async_trait]
/// impl EmailSender for MyEmailSender {
///     async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
///         // Use your email service (SendGrid, AWS SES, SMTP, etc.)
///         let verification_url = format!(
///             "https://myapp.com/verify?token={}",
///             context.token
///         );
///
///         // Send email using your service
///         my_email_service::send(
///             &context.email,
///             "Verify your email",
///             &format!("Click here to verify: {}", verification_url),
///         ).await?;
///
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait EmailSender: Send + Sync {
  /// Send a verification email to the user
  ///
  /// This method is called by AuthKit after generating a verification token.
  /// Implement your email sending logic here.
  ///
  /// # Arguments
  ///
  /// * `context` - Contains the email address, token, and expiration time
  ///
  /// # Returns
  ///
  /// * `Ok(())` if the email was sent successfully
  /// * `Err(_)` if there was an error sending the email
  async fn send_verification_email(&self, context: EmailContext) -> Result<()>;
}
