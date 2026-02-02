pub use crate::auth::Auth;
pub use crate::builder::AuthBuilder;
pub use crate::email::{EmailContext, EmailSender};
pub use crate::error::{AuthError, Result};
pub use crate::operations::{
  Login, Logout, Register, ResendEmailVerification, SendEmailVerification, Verify, VerifyEmail,
};
pub use crate::types::{Database, Session, User, VerificationToken};

// Email queue exports (only available with email-queue feature)
#[cfg(feature = "email-queue")]
pub use crate::email_job::{
  EmailJob, EmailJobType, EmailQueue, EmailQueueError, EmailWorker, EmailWorkerConfig,
  EmailWorkerHandle,
};
