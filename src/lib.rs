pub mod prelude;

pub use auth::Auth;
pub use builder::AuthBuilder;
pub use error::{AuthError, Result};
pub use operations::{
  Login, Logout, Register, ResendEmailVerification, SendEmailVerification, Verify, VerifyEmail,
};
pub use types::{Database, Session, User, VerificationToken};

mod auth;
mod builder;
mod database;
mod error;
mod operations;
mod security;
mod strategies;
mod types;
mod validation;

#[cfg(test)]
mod tests;
