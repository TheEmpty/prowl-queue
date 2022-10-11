//! ## Example
//! ```
//! use prowl_queue::{ProwlQueue, ProwlQueueSender};
//!
//! fn application(sender: &ProwlQueueSender) {
//!     let notification = prowl::Notification::new(...)
//!     sender.add(notification).expect("Failed to add notification");
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let (sender, reciever) = ProwlQueue::default().into_parts();
//!     tokio::spawn(reciever.async_loop());
//!     application(&sender);
//! }
//! ```

mod options;
mod queue;

pub use options::{LinearRetry, ProwlQueueOptions, RetryMethod};
pub use queue::{ProwlQueue, ProwlQueueReceiver, ProwlQueueSender};
