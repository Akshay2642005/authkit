pub mod prelude;

pub use auth::Auth;
pub use builder::AuthBuilder;
pub use error::{AuthError, Result};
pub use operations::{Login, Logout, Register, Verify};
pub use types::{Database, Session, User};

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
