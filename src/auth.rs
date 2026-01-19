use crate::database::DatabaseTrait;
use crate::email::EmailSender;
#[cfg(feature = "email-queue")]
use crate::email_job::{EmailQueue, EmailWorkerConfig, EmailWorkerHandle};
use crate::error::Result;
use crate::operations::email_verification::{
  ResendEmailVerification, SendEmailVerification, VerifyEmail,
};
use crate::operations::{Login, Logout, Register, Verify};
use crate::strategies::password::PasswordStrategy;
use crate::strategies::session::SessionStrategy;
use crate::strategies::token::TokenStrategy;
use crate::types::{Session, User, VerificationToken};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Auth {
  pub(crate) inner: Arc<AuthInner>,
}

pub(crate) struct AuthInner {
  pub(crate) db: Arc<Box<dyn DatabaseTrait>>,
  pub(crate) password_strategy: Box<dyn PasswordStrategy>,
  pub(crate) session_strategy: Box<dyn SessionStrategy>,
  pub(crate) token_strategy: Box<dyn TokenStrategy>,
  pub(crate) email_sender: Option<Arc<Box<dyn EmailSender>>>,

  /// Whether to automatically send verification email on registration
  /// Defaults to false
  pub(crate) send_verification_on_register: bool,

  /// Whether login requires email to be verified
  /// Defaults to false
  pub(crate) require_email_verification: bool,

  #[cfg(feature = "email-queue")]
  pub(crate) email_queue: Option<EmailQueue>,

  #[cfg(feature = "email-queue")]
  pub(crate) email_worker_config: Option<EmailWorkerConfig>,
}

impl std::fmt::Debug for AuthInner {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AuthInner")
      .field(
        "send_verification_on_register",
        &self.send_verification_on_register,
      )
      .field(
        "require_email_verification",
        &self.require_email_verification,
      )
      .finish_non_exhaustive()
  }
}

impl Auth {
  pub fn builder() -> crate::builder::AuthBuilder {
    crate::builder::AuthBuilder::new()
  }
  pub async fn register(&self, request: Register) -> Result<User> {
    crate::operations::register::execute(self, request).await
  }
  pub async fn login(&self, request: Login) -> Result<Session> {
    crate::operations::login::execute(self, request).await
  }
  pub async fn verify(&self, request: Verify) -> Result<User> {
    crate::operations::verify::execute(self, request).await
  }
  pub async fn logout(&self, request: Logout) -> Result<()> {
    crate::operations::logout::execute(self, request).await
  }
  pub async fn send_email_verification(
    &self,
    request: SendEmailVerification,
  ) -> Result<VerificationToken> {
    crate::operations::email_verification::send_email_verification(self, request).await
  }
  pub async fn verify_email(&self, request: VerifyEmail) -> Result<User> {
    crate::operations::email_verification::verify_email(self, request).await
  }
  pub async fn resend_email_verification(
    &self,
    request: ResendEmailVerification,
  ) -> Result<VerificationToken> {
    crate::operations::email_verification::resend_email_verification(self, request).await
  }
  pub async fn migrate(&self) -> Result<()> {
    self.inner.db.migrate().await
  }

  /// Check if an email sender is configured
  pub fn has_email_sender(&self) -> bool {
    self.inner.email_sender.is_some()
  }

  /// Check if verification emails are sent automatically on registration
  pub fn sends_verification_on_register(&self) -> bool {
    self.inner.send_verification_on_register
  }

  /// Check if login requires email verification
  pub fn requires_email_verification(&self) -> bool {
    self.inner.require_email_verification
  }

  /// Start the email background worker
  ///
  /// Returns a handle that can be used to monitor or stop the worker.
  /// The worker will process queued emails until shutdown.
  ///
  /// # Panics
  ///
  /// Panics if email queue is not enabled or email_sender is not configured.
  ///
  /// # Example
  ///
  /// ```rust,ignore
  /// let auth = Auth::builder()
  ///     .database(Database::sqlite("auth.db").await?)
  ///     .email_sender(Box::new(MyEmailSender))
  ///     .email_queue(EmailWorkerConfig::default())
  ///     .build()?;
  ///
  /// // Start worker
  /// let handle = auth.start_email_worker();
  ///
  /// // ... application runs ...
  ///
  /// // Graceful shutdown
  /// handle.shutdown().await?;
  /// ```
  #[cfg(feature = "email-queue")]
  pub fn start_email_worker(&self) -> EmailWorkerHandle {
    let email_sender = self
      .inner
      .email_sender
      .clone()
      .expect("email_sender must be configured to use email queue");

    let config = self
      .inner
      .email_worker_config
      .clone()
      .expect("email_queue must be configured");

    let (queue, worker) = crate::email_job::create_email_queue(email_sender, config);

    let handle = tokio::spawn(worker.run());

    EmailWorkerHandle::new(handle, queue)
  }

  /// Check if email queue is enabled
  #[cfg(feature = "email-queue")]
  pub fn has_email_queue(&self) -> bool {
    self.inner.email_queue.is_some()
  }

  /// Get a clone of the email queue (if enabled)
  #[cfg(feature = "email-queue")]
  pub fn email_queue(&self) -> Option<EmailQueue> {
    self.inner.email_queue.clone()
  }
}

unsafe impl Send for Auth {}
unsafe impl Sync for Auth {}
