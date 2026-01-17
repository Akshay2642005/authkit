// src/email_queue/error.rs

use thiserror::Error;

/// Errors that can occur with the email queue
#[derive(Error, Debug)]
pub enum EmailQueueError {
  /// Queue is full (when using non-blocking send)
  #[error("Email queue is full, try again later")]
  QueueFull,

  /// Channel is closed (worker has stopped)
  #[error("Email worker has stopped")]
  WorkerStopped,

  /// Email sending failed after all retries
  #[error("Failed to send email after {attempts} attempts: {message}")]
  SendFailed { attempts: u32, message: String },

  /// Worker configuration error
  #[error("Invalid worker configuration: {0}")]
  ConfigError(String),
}
