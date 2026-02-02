// src/email_queue/mod.rs

mod config;
mod error;
mod queue;
mod types;
mod worker;

pub use config::EmailWorkerConfig;
pub use error::EmailQueueError;
pub use queue::EmailQueue;
pub use types::{EmailJob, EmailJobType};
pub use worker::EmailWorker;

use crate::email::EmailSender;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub fn create_email_queue(
  email_sender: Arc<Box<dyn EmailSender>>,
  config: EmailWorkerConfig,
) -> (EmailQueue, EmailWorker) {
  let (sender, receiver) = mpsc::channel(config.channel_buffer_size);

  let queue = EmailQueue::new(sender, config.non_blocking);
  let worker = EmailWorker::new(receiver, email_sender, config.clone());

  (queue, worker)
}
pub struct EmailWorkerHandle {
  handle: JoinHandle<()>,
  queue: EmailQueue,
}

impl EmailWorkerHandle {
  pub fn new(handle: JoinHandle<()>, queue: EmailQueue) -> Self {
    Self { handle, queue }
  }
  pub fn queue(&self) -> EmailQueue {
    self.queue.clone()
  }
  pub fn is_running(&self) -> bool {
    !self.handle.is_finished()
  }
  pub fn abort(&self) {
    self.handle.abort();
  }
  pub async fn shutdown(self) -> Result<(), tokio::task::JoinError> {
    drop(self.queue);
    self.handle.await
  }
}
