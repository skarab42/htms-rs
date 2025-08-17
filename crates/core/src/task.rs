use futures_core::future::BoxFuture;

pub type TaskFuture = BoxFuture<'static, String>;

pub struct Task {
    pub id: String,
    pub future: TaskFuture,
}

impl Task {
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
