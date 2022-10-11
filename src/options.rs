use derive_getters::Getters;
use derive_new::new;
use tokio::time::Duration;

/// Options for the queue to know how to operate.
#[derive(Debug, Getters)]
pub struct ProwlQueueOptions {
    retry_method: RetryMethod,
}

/// Wrapper of the different retry methods
#[derive(Debug)]
pub enum RetryMethod {
    Linear(LinearRetry),
}

/// Your most generic type of retry. Retry every X until Y retries.
#[derive(new, Debug, Getters)]
pub struct LinearRetry {
    backoff: Duration,
    max_retries: Option<usize>,
}

impl ProwlQueueOptions {
    pub fn new(retry_method: RetryMethod) -> Self {
        Self { retry_method }
    }
}
