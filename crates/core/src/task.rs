//! Task system for **htms**.
//!
//! Defines a [`Task`] abstraction wrapping a future that produces a `String`.
//! Useful for scheduling or executing asynchronous jobs identified by an ID.
//!
//! # Example
//! ```rust
//! use htms_core::task::{Task, TaskFuture};
//! use std::future;
//!
//! let task = Task::new("hello", future::ready("world".to_string()));
//! ```

use futures_core::future::BoxFuture;

/// Boxed future returning a `String`.
pub type TaskFuture = BoxFuture<'static, String>;

/// Represents an asynchronous task with an identifier and a future.
pub struct Task {
    /// Unique identifier of the task.
    pub id: String,
    /// The asynchronous computation to be executed.
    pub future: TaskFuture,
}

impl Task {
    /// Create a new [`Task`] from an identifier and a future.
    ///
    /// # Example
    /// ```rust
    /// use htms_core::task::Task;
    /// use std::future;
    ///
    /// let task = Task::new("id", future::ready("done".to_string()));
    /// ```
    pub fn new<I: Into<String>, F>(id: I, future: F) -> Self
    where
        F: Future<Output = String> + Send + 'static,
    {
        Self {
            id: id.into(),
            future: Box::pin(future),
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::expect_used, clippy::unused_async)]
mod tests {
    use super::Task;

    async fn message_task(message: &str) -> String {
        message.to_string()
    }

    #[tokio::test]
    async fn task_new_identifier() {
        let task = Task::new("identifier", message_task("done"));

        assert_eq!(task.id, "identifier");
    }

    #[tokio::test]
    async fn task_future_resolves_to_expected_output() {
        let task = Task::new("id", message_task("expected output"));
        let output = task.future.await;

        assert_eq!(output, "expected output");
    }

    #[tokio::test]
    async fn task_future_is_send_and_static_and_can_be_spawned() {
        let task = Task::new("id", message_task("spawned output"));
        let handle = tokio::spawn(task.future);
        let output = handle.await.expect("join handle failed");

        assert_eq!(output, "spawned output");
    }
}
