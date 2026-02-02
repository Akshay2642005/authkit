use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum EmailJobType {
  EmailVerification,
  PasswordReset,
  MagicLink,
  Welcome,
}

impl EmailJobType {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::EmailVerification => "email_verification",
      Self::PasswordReset => "password_reset",
      Self::MagicLink => "magic_link",
      Self::Welcome => "welcome",
    }
  }
}

pub struct EmailJob {
  pub job_type: EmailJobType,
  pub recipient: String,
  pub token: String,
  pub token_expires_at: i64,
  pub user_id: String,
  pub attempts: u32,
  pub max_attempts: u32,
  pub created_at: i64,
}

impl EmailJob {
  pub fn new(
    job_type: EmailJobType,
    recipient: String,
    token: String,
    token_expires_at: i64,
    user_id: String,
  ) -> Self {
    let created_at = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;
    Self {
      job_type,
      recipient,
      token,
      token_expires_at,
      user_id,
      attempts: 0,
      max_attempts: 2,
      created_at,
    }
  }
  pub fn verification(
    recipient: String,
    token: String,
    token_expires_at: i64,
    user_id: String,
  ) -> Self {
    Self::new(
      EmailJobType::EmailVerification,
      recipient,
      token,
      token_expires_at,
      user_id,
    )
  }
}
