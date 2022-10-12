use crate::{LinearRetry, ProwlQueueOptions, RetryMethod};
use prowl::Notification;
use thiserror::Error;
use tokio::{
    sync::mpsc::{error::SendError, unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::{sleep, Duration},
};

pub struct ProwlQueue {
    prowl_queue_reciever: ProwlQueueReceiver,
    prowl_queue_sender: ProwlQueueSender,
}

#[derive(Clone)]
pub struct ProwlQueueSender {
    sender: UnboundedSender<Notification>,
}

pub struct ProwlQueueReceiver {
    options: ProwlQueueOptions,
    reciever: UnboundedReceiver<Notification>,
}

#[derive(Debug, Error)]
pub enum AddError {
    /// Tokio failed to queue the notification.
    #[error("{0}")]
    SendError(SendError<Notification>),
}

/// Implements a Tokio mpsc backed notification queue.
impl ProwlQueue {
    pub fn new(options: ProwlQueueOptions) -> Self {
        let (sender, reciever) = unbounded_channel();
        let prowl_queue_reciever = ProwlQueueReceiver { options, reciever };
        let prowl_queue_sender = ProwlQueueSender { sender };

        Self {
            prowl_queue_reciever,
            prowl_queue_sender,
        }
    }

    pub fn into_parts(self) -> (ProwlQueueSender, ProwlQueueReceiver) {
        (self.prowl_queue_sender, self.prowl_queue_reciever)
    }
}

impl ProwlQueueSender {
    /// Queue a notification for sending
    pub fn add(&self, notification: Notification) -> Result<(), Box<AddError>> {
        self.sender
            .send(notification)
            .map_err(AddError::SendError)
            .map_err(Box::new)?;
        Ok(())
    }
}

impl ProwlQueueReceiver {
    /// Spawn a recv'ing loop that will continue to process and
    /// retry notifications. Stops processing with the sender half
    /// is dropped.
    pub async fn async_loop(mut self) {
        log::debug!("Notifications channel processor started.");
        while let Some(notification) = self.reciever.recv().await {
            let mut retry = 0;
            'notification: loop {
                match notification.add().await {
                    Ok(_) => break 'notification,
                    Err(prowl::AddError::Send(e)) => {
                        log::warn!("Will retry notification. Try {retry} failed due to {:?}", e);
                    }
                    Err(e) => {
                        // API or internal error - lets not hammer with invalid requests.
                        // TODO: don't break if 5xx response
                        log::error!("Terminally failed to send notification due to {:?}", e);
                        break 'notification;
                    }
                }

                match self.options.retry_method() {
                    RetryMethod::Linear(linear_retry) => {
                        if let Some(max) = linear_retry.max_retries() {
                            if retry + 1 > *max {
                                log::warn!("Dropping notification {:?} because it's on retry {retry} when max_retries is {}", notification, max);
                                break 'notification;
                            }
                        }
                        sleep(*linear_retry.backoff()).await;
                    }
                }

                retry += 1;
            }
        }
        log::warn!("Notification channel has been closed.");
    }
}

impl Default for ProwlQueue {
    fn default() -> Self {
        let retry_method = LinearRetry::new(Duration::from_secs(60), None);
        let retry_method = RetryMethod::Linear(retry_method);
        let options = ProwlQueueOptions::new(retry_method);

        let (sender, reciever) = unbounded_channel();
        let prowl_queue_reciever = ProwlQueueReceiver { options, reciever };
        let prowl_queue_sender = ProwlQueueSender { sender };

        Self {
            prowl_queue_reciever,
            prowl_queue_sender,
        }
    }
}
