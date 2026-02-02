mod auth;
mod builder;
mod database;
mod email;
#[cfg(feature = "email-queue")]
mod email_job;
mod error;
mod operations;
mod security;
mod strategies;
mod types;
mod validation;

#[cfg(test)]
mod tests;

pub mod prelude;
pub use auth::Auth;
pub use builder::AuthBuilder;
pub use email::{EmailContext, EmailSender};
pub use error::{AuthError, Result};
pub use operations::{
  Login, Logout, Register, ResendEmailVerification, SendEmailVerification, Verify, VerifyEmail,
};
pub use types::{Account, Database, Session, User, VerificationToken};

// Email queue exports (only available with email-queue feature)
#[cfg(feature = "email-queue")]
pub use email_job::{
  EmailJob, EmailJobType, EmailQueue, EmailQueueError, EmailWorker, EmailWorkerConfig,
  EmailWorkerHandle,
};
