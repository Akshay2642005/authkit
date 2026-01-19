use super::error::EmailQueueError;
use super::types::EmailJob;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct EmailQueue {
  sender: mpsc::Sender<EmailJob>,
  non_blocking: bool,
}

impl EmailQueue {
  pub(crate) fn new(sender: mpsc::Sender<EmailJob>, non_blocking: bool) -> Self {
    Self {
      sender,
      non_blocking,
    }
  }
  pub async fn enqueue(&self, job: EmailJob) -> Result<(), EmailQueueError> {
    if self.non_blocking {
      self.sender.try_send(job).map_err(|e| match e {
        mpsc::error::TrySendError::Full(_) => EmailQueueError::QueueFull,
        mpsc::error::TrySendError::Closed(_) => EmailQueueError::WorkerStopped,
      })
    } else {
      self
        .sender
        .send(job)
        .await
        .map_err(|_| EmailQueueError::WorkerStopped)
    }
  }

  pub fn is_closed(&self) -> bool {
    self.sender.is_closed()
  }
  pub fn capacity(&self) -> usize {
    self.sender.capacity()
  }
}

impl std::fmt::Debug for EmailQueue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("EmailQueue")
      .field("non_blocking", &self.non_blocking)
      .field("is_closed", &self.is_closed())
      .finish()
  }
}
