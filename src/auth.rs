use crate::database::DatabaseTrait;
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
  #[allow(dead_code)]
  pub(crate) token_strategy: Box<dyn TokenStrategy>,
}

impl std::fmt::Debug for AuthInner {
  /// Formats a non-exhaustive debug representation for `AuthInner`.
  ///
  /// This writes a debug struct named `AuthInner` without exposing any internal fields.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// // The usual way to obtain a debug string for an `AuthInner` value is via the `Debug`
  /// // implementation (e.g. through `format!("{:?}", value)`).
  /// let s = format!("{:?}", auth_inner);
  /// assert!(s.starts_with("AuthInner"));
  /// ```
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AuthInner").finish_non_exhaustive()
  }
}

impl Auth {
  /// Creates a new AuthBuilder for constructing an Auth instance.
  ///
  /// Returns an `AuthBuilder` ready to configure and build an `Auth`.
  ///
  /// # Examples
  ///
  /// ```
  /// let builder = crate::auth::builder();
  /// // configure the builder as needed, then build the Auth instance:
  /// // let auth = builder.build();
  /// ```
  pub fn builder() -> crate::builder::AuthBuilder {
    crate::builder::AuthBuilder::new()
  }
  /// Registers a new user and returns the created user.
  ///
  /// # Returns
  ///
  /// A `Result` containing the created `User` on success.
  ///
  /// # Examples
  ///
  /// ```
  /// // async context required
  /// // let user = auth.register(Register { /* ... */ }).await.unwrap();
  /// ```
  pub async fn register(&self, request: Register) -> Result<User> {
    crate::operations::register::execute(self, request).await
  }
  /// Authenticates a user using the provided credentials and returns an active session.
  ///
  /// # Parameters
  ///
  /// - `request`: login credentials and options used to authenticate the user.
  ///
  /// # Returns
  ///
  /// A `Session` representing the authenticated user's session.
  ///
  /// # Examples
  ///
  /// ```
  /// # use crate::{Auth, Login};
  /// # async fn run_example() -> anyhow::Result<()> {
  /// let auth = Auth::builder().build();
  /// let req = Login {
  ///     email: "user@example.com".into(),
  ///     password: "s3cr3t".into(),
  ///     // fill other fields as required
  /// };
  /// let session = auth.login(req).await?;
  /// // use `session` (for example, inspect session.user_id)
  /// # Ok(())
  /// # }
  /// ```
  pub async fn login(&self, request: Login) -> Result<Session> {
    crate::operations::login::execute(self, request).await
  }
  /// Verifies an authentication request and returns the associated user.
  ///
  /// # Examples
  ///
  /// ```
  /// # use crate::{Auth, types::Verify};
  /// # fn example(auth: &Auth, req: Verify) {
  /// let user = futures::executor::block_on(auth.verify(req)).unwrap();
  /// # }
  /// ```
  pub async fn verify(&self, request: Verify) -> Result<User> {
    crate::operations::verify::execute(self, request).await
  }
  /// Logs out an authenticated session.
  ///
  /// Performs the logout operation for the provided `Logout` request.
  ///
  /// # Examples
  ///
  /// ```rust
  /// # async fn example(auth: crate::Auth, request: crate::operations::logout::Logout) {
  /// auth.logout(request).await.unwrap();
  /// # }
  /// ```
  pub async fn logout(&self, request: Logout) -> Result<()> {
    crate::operations::logout::execute(self, request).await
  }
  /// Sends an email verification to the user described by `request`.
  ///
  /// @param request The details required to send the verification email (recipient and any options).
  ///
  /// # Returns
  ///
  /// `VerificationToken` for the sent verification email.
  ///
  /// # Examples
  ///
  /// ```
  /// // async context
  /// let request = SendEmailVerification { /* recipient, etc. */ };
  /// let token = auth.send_email_verification(request).await.unwrap();
  /// assert!(!token.id.is_empty());
  /// ```
  pub async fn send_email_verification(
    &self,
    request: SendEmailVerification,
  ) -> Result<VerificationToken> {
    crate::operations::email_verification::send_email_verification(self, request).await
  }
  /// Verifies an email verification request and returns the corresponding user.
  ///
  /// Verifies the provided `VerifyEmail` request (for example, a token or code) and, on success,
  /// yields the `User` whose email was verified.
  ///
  /// # Parameters
  ///
  /// - `request`: a `VerifyEmail` request containing the verification token or code to validate.
  ///
  /// # Returns
  ///
  /// A `User` representing the account whose email was successfully verified.
  ///
  /// # Examples
  ///
  /// ```
  /// // async context required
  /// # async fn example(auth: &crate::Auth, request: crate::operations::email_verification::VerifyEmail) {
  /// let user = auth.verify_email(request).await.unwrap();
  /// assert_eq!(user.email_verified, true);
  /// # }
  /// ```
  pub async fn verify_email(&self, request: VerifyEmail) -> Result<User> {
    crate::operations::email_verification::verify_email(self, request).await
  }
  /// Requests a new email verification token for the given user and returns it.
  ///
  /// On success, yields a `VerificationToken` containing the token to be sent to the user's email.
  ///
  /// # Examples
  ///
  /// ```
  /// # use crate::{Auth, SendEmailVerification};
  /// # async fn example(auth: &Auth) -> Result<(), Box<dyn std::error::Error>> {
  /// let req = SendEmailVerification { user_id: 42 };
  /// let token = auth.resend_email_verification(req).await?;
  /// assert!(!token.value.is_empty());
  /// # Ok(()) }
  /// ```
  pub async fn resend_email_verification(
    &self,
    request: ResendEmailVerification,
  ) -> Result<VerificationToken> {
    crate::operations::email_verification::resend_email_verification(self, request).await
  }
  /// Applies any pending database migrations for the configured database.
  ///
  /// # Returns
  ///
  /// `Ok(())` if migrations completed successfully, otherwise an error describing the failure.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # async fn example(auth: &crate::Auth) {
  /// auth.migrate().await.unwrap();
  /// # }
  /// ```
  pub async fn migrate(&self) -> Result<()> {
    self.inner.db.migrate().await
  }
}

unsafe impl Send for Auth {}
unsafe impl Sync for Auth {}