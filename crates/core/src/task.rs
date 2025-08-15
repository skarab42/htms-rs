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
