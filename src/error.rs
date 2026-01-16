use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
  #[error("Database error: {0}")]
  DatabaseError(#[from] sqlx::Error),

  #[error("User with email {0} already exists")]
  UserAlreadyExists(String),

  #[error("User not found")]
  UserNotFound,

  #[error("Invalid email or password")]
  InvalidCredentials,

  #[error("Session not found or expired")]
  InvalidSession,

  #[error("Password validation failed: {0}")]
  WeakPassword(String),

  #[error("Invalid email format")]
  InvalidEmailFormat,

  #[error("Missing required configuration: Database")]
  MissingDatabase,

  #[error("Missing required configuration: Password strategy")]
  MissingPasswordStrategy,

  #[error("Password hasing error: {0}")]
  PasswordHashingError(String),

  #[error("Token generation error: {0}")]
  TokenGenerationError(String),

  #[error("Internal error: {0}")]
  InternalError(String),

  #[error("Invalid Token: {0}")]
  InvalidToken(String),

  #[error("Token Already Used: {0}")]
  TokenAlreadyUsed(String),

  #[error("Email Already Verified: {0}")]
  EmailAlreadyVerified(String),

  #[error("Token Expired: {0}")]
  TokenExpired(String),

  #[error("Email send failed: {0}")]
  EmailSendFailed(String),

  #[error("Rate limit exceeded: {0}")]
  RateLimitExceeded(String),
}

pub type Result<T> = std::result::Result<T, AuthError>;
