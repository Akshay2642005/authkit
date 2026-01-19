use crate::auth::{Auth, AuthInner};
use crate::email::EmailSender;
#[cfg(feature = "email-queue")]
use crate::email_job::EmailWorkerConfig;
use crate::error::{AuthError, Result};
use crate::strategies::password::PasswordStrategyType;
use crate::strategies::session::SessionStrategyType;
use crate::strategies::token::TokenStrategyType;
use crate::types::Database;
use std::sync::Arc;

pub struct AuthBuilder {
  database: Option<Database>,
  password_strategy: Option<PasswordStrategyType>,
  session_strategy: Option<SessionStrategyType>,
  token_strategy: Option<TokenStrategyType>,
  email_sender: Option<Box<dyn EmailSender>>,

  /// Whether to automatically send verification email on registration
  /// Defaults to false
  send_verification_on_register: bool,

  /// Whether login requires email to be verified
  /// Defaults to false
  require_email_verification: bool,

  #[cfg(feature = "email-queue")]
  email_queue_config: Option<EmailWorkerConfig>,
}

impl AuthBuilder {
  pub(crate) fn new() -> Self {
    Self {
      database: None,
      password_strategy: None,
      session_strategy: None,
      token_strategy: None,
      email_sender: None,
      send_verification_on_register: false,
      require_email_verification: false,
      #[cfg(feature = "email-queue")]
      email_queue_config: None,
    }
  }
  pub fn database(mut self, db: Database) -> Self {
    self.database = Some(db);
    self
  }
  pub fn password_strategy(mut self, strategy: PasswordStrategyType) -> Self {
    self.password_strategy = Some(strategy);
    self
  }
  pub fn session_strategy(mut self, strategy: SessionStrategyType) -> Self {
    self.session_strategy = Some(strategy);
    self
  }
  pub fn token_strategy(mut self, strategy: TokenStrategyType) -> Self {
    self.token_strategy = Some(strategy);
    self
  }

  /// Set a custom email sender for verification emails
  ///
  /// If not set, verification tokens are generated but emails are not sent automatically.
  /// The application must handle email sending manually using the returned token.
  ///
  /// # Example
  ///
  /// ```rust,ignore
  /// use authkit::prelude::*;
  /// use authkit::email::{EmailSender, EmailContext};
  /// use async_trait::async_trait;
  ///
  /// struct MyEmailSender;
  ///
  /// #[async_trait]
  /// impl EmailSender for MyEmailSender {
  ///     async fn send_verification_email(&self, context: EmailContext) -> Result<()> {
  ///         // Send email using your service
  ///         Ok(())
  ///     }
  /// }
  ///
  /// let auth = Auth::builder()
  ///     .database(Database::sqlite("auth.db").await?)
  ///     .email_sender(Box::new(MyEmailSender))
  ///     .build()?;
  /// ```
  pub fn email_sender(mut self, sender: Box<dyn EmailSender>) -> Self {
    self.email_sender = Some(sender);
    self
  }

  /// Configure whether to automatically send verification email on registration
  ///
  /// When set to `true`, a verification email is automatically sent when a user registers
  /// (requires `email_sender` to be configured).
  ///
  /// When set to `false` (default), registration creates the user but does NOT send a
  /// verification email. The application must call `send_email_verification()`
  /// manually if email verification is desired.
  ///
  /// # Example
  ///
  /// ```rust,ignore
  /// // Enable automatic verification emails on registration
  /// let auth = Auth::builder()
  ///     .database(Database::sqlite("auth.db").await?)
  ///     .email_sender(Box::new(MyEmailSender))
  ///     .send_verification_on_register(true)
  ///     .build()?;
  ///
  /// // Register user (verification email sent automatically)
  /// let user = auth.register(Register { ... }).await?;
  /// ```
  pub fn send_verification_on_register(mut self, enabled: bool) -> Self {
    self.send_verification_on_register = enabled;
    self
  }

  /// Configure whether login requires email verification
  ///
  /// When set to `true`, users cannot login until their email is verified.
  /// Login will return `AuthError::EmailNotVerified` for unverified users.
  ///
  /// When set to `false` (default), users can login regardless of email verification status.
  ///
  /// # Example
  ///
  /// ```rust,ignore
  /// // Require email verification before login
  /// let auth = Auth::builder()
  ///     .database(Database::sqlite("auth.db").await?)
  ///     .email_sender(Box::new(MyEmailSender))
  ///     .send_verification_on_register(true)
  ///     .require_email_verification(true)
  ///     .build()?;
  ///
  /// // Register user
  /// let user = auth.register(Register { ... }).await?;
  ///
  /// // This will fail with EmailNotVerified error
  /// let session = auth.login(Login { ... }).await; // Error!
  ///
  /// // User must verify email first, then login will succeed
  /// ```
  pub fn require_email_verification(mut self, required: bool) -> Self {
    self.require_email_verification = required;
    self
  }

  /// Enable email job queue for async background email processing
  ///
  /// When enabled, emails are queued and sent in a background task
  /// instead of blocking the request. This significantly improves
  /// registration response times.
  ///
  /// # Requirements
  ///
  /// - An `email_sender` must be configured
  /// - You must call `auth.start_email_worker()` after building
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
  /// // Start the background worker
  /// let worker_handle = auth.start_email_worker();
  /// ```
  #[cfg(feature = "email-queue")]
  pub fn email_queue(mut self, config: EmailWorkerConfig) -> Self {
    self.email_queue_config = Some(config);
    self
  }

  pub fn build(self) -> Result<Auth> {
    let database = self.database.ok_or(AuthError::MissingDatabase)?;

    #[cfg(feature = "argon2")]
    let password_strategy = self
      .password_strategy
      .unwrap_or_default()
      .create_strategy()?;

    #[cfg(not(feature = "argon2"))]
    let password_strategy = self
      .password_strategy
      .ok_or(AuthError::MissingPasswordStrategy)?
      .create_strategy()?;

    let session_strategy = self.session_strategy.unwrap_or_default().create_strategy();

    let db_trait = crate::database::create_database_trait(database.inner);
    let db_arc = Arc::new(db_trait);

    let token_strategy = self.token_strategy.unwrap_or_default().create_strategy();

    let email_sender = self.email_sender.map(Arc::new);

    // Build email queue if configured
    #[cfg(feature = "email-queue")]
    let (email_queue, email_worker_config) = {
      if let (Some(config), Some(ref sender)) = (&self.email_queue_config, &email_sender) {
        let (queue, _worker) = crate::email_job::create_email_queue(sender.clone(), config.clone());
        (Some(queue), Some(config.clone()))
      } else {
        (None, None)
      }
    };

    Ok(Auth {
      inner: Arc::new(AuthInner {
        db: db_arc,
        password_strategy,
        session_strategy,
        token_strategy,
        email_sender,
        send_verification_on_register: self.send_verification_on_register,
        require_email_verification: self.require_email_verification,
        #[cfg(feature = "email-queue")]
        email_queue,
        #[cfg(feature = "email-queue")]
        email_worker_config,
      }),
    })
  }
}
