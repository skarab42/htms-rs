use async_stream::stream;
use bytes::Bytes;
use futures_core::Stream;
use futures_util::{FutureExt, StreamExt, stream::FuturesUnordered};

use crate::task::Task;

pub trait Render: Sized {
    fn template() -> Bytes;
    fn tasks(self) -> Option<Vec<Task>>;

    #[must_use]
    fn response(id: &str, html: &str) -> Bytes {
        format!(
            r#"<script data-htms="{id}">onHtmsResponse("{id}", "{html}")</script>{}"#,
            "\n"
        )
        .into()
    }

    #[must_use]
    fn final_chunk() -> Option<Bytes> {
        None
    }

    #[must_use]
    fn render(self) -> impl Stream<Item = Bytes> {
        stream! {
            yield Self::template();

            if let Some(tasks) = self.tasks() {
                let mut tasks_unordered = FuturesUnordered::new();

                for task in tasks {
                    let id = task.id;
                    let future = task.future.map(move |output| Self::response(&id, &output));

                    tasks_unordered.push(future.boxed());
                }

                while let Some(bytes) = tasks_unordered.next().await {
                    yield bytes;
                }
            }

            if let Some(chunk) = Self::final_chunk() {
                yield chunk;
            }
        }
    }
}
