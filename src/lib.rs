pub mod prelude;

pub use auth::Auth;
pub use builder::AuthBuilder;
pub use error::{AuthError, Result};
pub use types::{Database, Session, User};

mod auth;
mod builder;
mod error;
mod operations;
mod types;
mod validation;

#[cfg(test)]
mod tests;
