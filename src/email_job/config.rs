#[derive(Clone, Debug)]
pub struct EmailWorkerConfig {
  pub channel_buffer_size: usize,
  pub base_retry_delay: std::time::Duration,
  pub max_retry_delay: std::time::Duration,
  pub default_max_attempts: u32,
  pub non_blocking: bool,
}

impl Default for EmailWorkerConfig {
  fn default() -> Self {
    Self {
      channel_buffer_size: 100,
      base_retry_delay: std::time::Duration::from_secs(1),
      max_retry_delay: std::time::Duration::from_secs(60),
      default_max_attempts: 2,
      non_blocking: false,
    }
  }
}

impl EmailWorkerConfig {
  pub fn with_buffer_size(mut self, size: usize) -> Self {
    self.channel_buffer_size = size;
    self
  }
  pub fn with_retry_delay(mut self, delay: std::time::Duration) -> Self {
    self.base_retry_delay = delay;
    self
  }
  pub fn blocking(mut self) -> Self {
    self.non_blocking = false;
    self
  }
}
