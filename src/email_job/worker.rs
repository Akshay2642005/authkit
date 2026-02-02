use super::config::EmailWorkerConfig;
use super::types::{EmailJob, EmailJobType};
use crate::email::{EmailContext, EmailSender};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct EmailWorker {
  receiver: mpsc::Receiver<EmailJob>,
  email_sender: Arc<Box<dyn EmailSender>>,
  config: EmailWorkerConfig,
}

impl EmailWorker {
  pub(crate) fn new(
    receiver: mpsc::Receiver<EmailJob>,
    email_sender: Arc<Box<dyn EmailSender>>,
    config: EmailWorkerConfig,
  ) -> Self {
    Self {
      receiver,
      email_sender,
      config,
    }
  }
  pub async fn run(mut self) {
    log::info!("Email worker started");

    while let Some(job) = self.receiver.recv().await {
      self.process_job(job).await;
    }

    log::info!("Email worker stopped (channel closed)");
  }
  async fn process_job(&self, mut job: EmailJob) {
    log::debug!(
      "Processing email job: type={}, recipient={}, user_id={}",
      job.job_type.as_str(),
      job.recipient,
      job.user_id
    );

    loop {
      job.attempts += 1;

      match self.send_email(&job).await {
        Ok(()) => {
          log::info!(
            "Email sent successfully: type={}, recipient={}, attempts={}",
            job.job_type.as_str(),
            job.recipient,
            job.attempts
          );
          return;
        }
        Err(e) => {
          log::warn!(
            "Email send failed (attempt {}/{}): type={}, recipient={}, error={}",
            job.attempts,
            job.max_attempts,
            job.job_type.as_str(),
            job.recipient,
            e
          );

          if job.attempts >= job.max_attempts {
            log::error!(
              "Email job failed permanently after {} attempts: type={}, recipient={}, user_id={}",
              job.attempts,
              job.job_type.as_str(),
              job.recipient,
              job.user_id
            );
            return;
          }

          // Exponential backoff with jitter
          let delay = self.calculate_backoff(job.attempts);
          log::debug!("Retrying in {:?}...", delay);
          tokio::time::sleep(delay).await;
        }
      }
    }
  }

  async fn send_email(&self, job: &EmailJob) -> Result<(), crate::error::AuthError> {
    let context = EmailContext {
      email: job.recipient.clone(),
      token: job.token.clone(),
      expires_at: job.token_expires_at,
    };

    match job.job_type {
      EmailJobType::EmailVerification => self.email_sender.send_verification_email(context).await,
      EmailJobType::PasswordReset => self.email_sender.send_verification_email(context).await,
      EmailJobType::MagicLink => self.email_sender.send_verification_email(context).await,
      EmailJobType::Welcome => Ok(()),
    }
  }

  fn calculate_backoff(&self, attempt: u32) -> Duration {
    let base = self.config.base_retry_delay.as_millis() as u64;
    let max = self.config.max_retry_delay.as_millis() as u64;

    let exponential = base.saturating_mul(2u64.saturating_pow(attempt.saturating_sub(1)));

    let clamped = exponential.min(max);

    let jitter_range = clamped / 10;
    let jitter = if jitter_range > 0 {
      use rand::Rng;
      let mut rng = rand::rng();
      rng.random_range(0..jitter_range * 2) as i64 - jitter_range as i64
    } else {
      0
    };

    Duration::from_millis((clamped as i64 + jitter).max(0) as u64)
  }
}
