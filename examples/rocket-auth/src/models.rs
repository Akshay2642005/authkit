//! Request and Response Models for Rocket API
//!
//! This module defines all the JSON request and response types
//! used by the Rocket authentication API.

use serde::{Deserialize, Serialize};

// ============================================================================
// Authentication Request/Response Models
// ============================================================================

/// Request body for user registration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterRequest {
  pub email: String,
  pub password: String,
}

/// Response for successful registration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterResponse {
  pub id: String,
  pub email: String,
  pub email_verified: bool,
  pub created_at: i64,
  pub message: String,
}

/// Request body for user login
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginRequest {
  pub email: String,
  pub password: String,
}

/// Response for successful login
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginResponse {
  pub token: String,
  pub user_id: String,
  pub expires_at: i64,
  pub message: String,
}

/// Request body for logout
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogoutRequest {
  pub token: String,
}

/// Response for successful logout
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogoutResponse {
  pub message: String,
}

/// Response for session verification
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VerifyResponse {
  pub id: String,
  pub email: String,
  pub email_verified: bool,
  pub created_at: i64,
}

// ============================================================================
// Email Verification Request/Response Models
// ============================================================================

/// Request body for sending email verification
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SendVerificationRequest {
  pub user_id: String,
}

/// Response for sending verification email
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SendVerificationResponse {
  pub token: String,
  pub email: String,
  pub expires_at: i64,
  pub message: String,
}

/// Request body for verifying email
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VerifyEmailRequest {
  pub token: String,
}

/// Response for email verification
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VerifyEmailResponse {
  pub id: String,
  pub email: String,
  pub email_verified: bool,
  pub email_verified_at: Option<i64>,
  pub message: String,
}

/// Request body for resending verification email
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResendVerificationRequest {
  pub email: String,
}

/// Response for resending verification email
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResendVerificationResponse {
  pub token: String,
  pub email: String,
  pub expires_at: i64,
  pub message: String,
}

// ============================================================================
// Error Response Models
// ============================================================================

/// Generic error response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorResponse {
  pub error: String,
  pub message: String,
}

impl ErrorResponse {
  /// Create a new error response
  #[allow(dead_code)]
  pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
    Self {
      error: error.into(),
      message: message.into(),
    }
  }

  /// Create error response from AuthError
  pub fn from_auth_error(err: &authkit::AuthError) -> Self {
    let error = match err {
      authkit::AuthError::DatabaseError(_) => "DatabaseError",
      authkit::AuthError::UserAlreadyExists(_) => "UserAlreadyExists",
      authkit::AuthError::UserNotFound => "UserNotFound",
      authkit::AuthError::InvalidCredentials => "InvalidCredentials",
      authkit::AuthError::InvalidSession => "InvalidSession",
      authkit::AuthError::WeakPassword(_) => "WeakPassword",
      authkit::AuthError::InvalidEmailFormat => "InvalidEmailFormat",
      authkit::AuthError::MissingDatabase => "MissingDatabase",
      authkit::AuthError::MissingPasswordStrategy => "MissingPasswordStrategy",
      authkit::AuthError::PasswordHashingError(_) => "PasswordHashingError",
      authkit::AuthError::TokenGenerationError(_) => "TokenGenerationError",
      authkit::AuthError::InternalError(_) => "InternalError",
      authkit::AuthError::InvalidToken(_) => "InvalidToken",
      authkit::AuthError::TokenAlreadyUsed(_) => "TokenAlreadyUsed",
      authkit::AuthError::EmailAlreadyVerified(_) => "EmailAlreadyVerified",
      authkit::AuthError::TokenExpired(_) => "TokenExpired",
      authkit::AuthError::EmailSendFailed(_) => "EmailSendFailed",
      authkit::AuthError::RateLimitExceeded(_) => "RateLimitExceeded",
    };

    Self {
      error: error.to_string(),
      message: err.to_string(),
    }
  }
}
