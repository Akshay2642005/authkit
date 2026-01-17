mod auth;
mod builder;
mod database;
mod email;
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
pub use types::{Database, Session, User, VerificationToken};
