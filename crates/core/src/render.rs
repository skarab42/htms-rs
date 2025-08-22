//! Rendering system for **htms**.
//!
//! Provides the [`Render`] trait, which defines how to generate an initial template
//! plus optional asynchronous [`Task`]s producing dynamic HTML chunks.
//!
//! # Example: stream to stdout
//! ```rust
//! use htms_core::render::Render;
//! use bytes::Bytes;
//! use futures_util::StreamExt;
//!
//! #[derive(Default)]
//! struct Page {}
//!
//! impl Render for Page {
//!     fn template() -> Bytes {
//!         "<h1>Hello</h1>".into()
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let page = Page::default();
//!     let mut stream = Box::pin(page.render());
//!
//!     while let Some(bytes) = stream.next().await {
//!         print!("{}", String::from_utf8_lossy(&bytes));
//!     }
//! }
//! ```
//!
//! # Example: with tasks
//! ```rust
//! use bytes::Bytes;
//! use futures_util::StreamExt;
//! use htms_core::{render::Render, task::Task};
//! use std::future;
//!
//! #[derive(Default)]
//! struct Page {}
//!
//! impl Render for Page {
//!     fn template() -> Bytes {
//!         "<h1>Users</h1>".into()
//!     }
//!
//!     fn tasks(self) -> Option<Vec<Task>> {
//!         Some(vec![
//!             Task::new("u1", future::ready("<p>Alice</p>".to_string())),
//!             Task::new("u2", future::ready("<p>Bob</p>".to_string())),
//!         ])
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let page = Page::default();
//!     let mut stream = Box::pin(page.render());
//!
//!     while let Some(bytes) = stream.next().await {
//!         print!("{}", String::from_utf8_lossy(&bytes));
//!     }
//! }
//! ```

use async_stream::stream;
use bytes::Bytes;
use futures_core::Stream;
use futures_util::{FutureExt, StreamExt, stream::FuturesUnordered};

use crate::task::Task;

/// Trait defining rendering logic.
///
/// A type implementing [`Render`] produces an initial template,
/// optional async [`Task`]s, and a final chunk if needed.
pub trait Render: Sized {
    /// Return the static HTML template for this renderer.
    fn template() -> Bytes;

    /// Return the list of tasks associated with this renderer.
    ///
    /// Defaults to `None`.
    fn tasks(self) -> Option<Vec<Task>> {
        None
    }

    /// Build a `<htms-chunk>` wrapper from an ID and HTML fragment.
    #[must_use]
    fn response(id: &str, html: &str) -> Bytes {
        format!(r#"<htms-chunk target="{id}">{html}</htms-chunk>{}"#, "\n").into()
    }

    /// Optionally, return a final chunk to yield after all tasks complete.
    /// Mainly used for cleaning htms dirty tags and close the body/html tags.
    #[must_use]
    fn final_chunk() -> Option<Bytes> {
        None
    }

    /// Render the template plus task results as a stream of HTML chunks.
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::expect_used)]
mod render_template_only {
    use bytes::Bytes;
    use futures_util::StreamExt;

    use crate::Render;

    const TEMPLATE: &[u8; 27] = b"<html>template only</html>\n";
    const FINAL_CHUNK: &[u8; 21] = b"<!-- final chunk -->\n";

    struct Template;
    struct TemplateWithFinalChunk;

    impl Render for Template {
        fn template() -> Bytes {
            Bytes::from_static(TEMPLATE)
        }
    }

    impl Render for TemplateWithFinalChunk {
        fn template() -> Bytes {
            Bytes::from_static(TEMPLATE)
        }

        fn final_chunk() -> Option<Bytes> {
            Some(Bytes::from_static(FINAL_CHUNK))
        }
    }

    #[tokio::test]
    async fn without_final_chunk() {
        let stream = Template.render();
        let chunks: Vec<Bytes> = stream.collect().await;

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], Bytes::from_static(TEMPLATE));
    }

    #[tokio::test]
    async fn with_final_chunk() {
        let stream = TemplateWithFinalChunk.render();
        let chunks: Vec<Bytes> = stream.collect().await;

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], Bytes::from_static(TEMPLATE));
        assert_eq!(chunks[1], Bytes::from_static(FINAL_CHUNK));
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(clippy::expect_used)]
mod render_template_with_tasks {
    use bytes::Bytes;
    use futures_util::StreamExt;

    use crate::{Render, Task};

    const TEMPLATE: &[u8; 33] = b"<html>template with tasks</html>\n";
    const FINAL_CHUNK: &[u8; 21] = b"<!-- final chunk -->\n";

    const FIRST_TASK_OUTPUT: &str = "first task done";
    const SECOND_TASK_OUTPUT: &str = "second task done";

    #[allow(clippy::unnecessary_wraps)]
    fn some_tasks() -> Option<Vec<Task>> {
        Some(vec![
            Task::new("first_task", async { FIRST_TASK_OUTPUT.into() }),
            Task::new("second_task", async { SECOND_TASK_OUTPUT.into() }),
        ])
    }

    struct Template;
    struct TemplateWithFinalChunk;

    impl Render for Template {
        fn template() -> Bytes {
            Bytes::from_static(TEMPLATE)
        }

        fn tasks(self) -> Option<Vec<Task>> {
            some_tasks()
        }
    }

    impl Render for TemplateWithFinalChunk {
        fn template() -> Bytes {
            Bytes::from_static(TEMPLATE)
        }

        fn tasks(self) -> Option<Vec<Task>> {
            some_tasks()
        }

        fn final_chunk() -> Option<Bytes> {
            Some(Bytes::from_static(FINAL_CHUNK))
        }
    }

    #[tokio::test]
    async fn without_final_chunk() {
        let stream = Template.render();
        let chunks: Vec<Bytes> = stream.collect().await;
        let responses: Vec<Bytes> = chunks[1..chunks.len()].to_vec();
        let expected_responses = [
            Template::response("first_task", FIRST_TASK_OUTPUT),
            Template::response("second_task", SECOND_TASK_OUTPUT),
        ];

        assert_eq!(chunks.len(), 3);
        assert_eq!(responses.len(), 2);
        assert_eq!(chunks[0], Bytes::from_static(TEMPLATE));
        assert!(expected_responses.iter().all(|e| responses.contains(e)));
    }

    #[tokio::test]
    async fn with_final_chunk() {
        let stream = TemplateWithFinalChunk.render();
        let chunks: Vec<Bytes> = stream.collect().await;
        let responses: Vec<Bytes> = chunks[1..chunks.len() - 1].to_vec();
        let expected_responses = [
            Template::response("first_task", FIRST_TASK_OUTPUT),
            Template::response("second_task", SECOND_TASK_OUTPUT),
        ];

        assert_eq!(chunks.len(), 4);
        assert_eq!(responses.len(), 2);
        assert_eq!(chunks[0], Bytes::from_static(TEMPLATE));
        assert!(expected_responses.iter().all(|e| responses.contains(e)));
        assert_eq!(chunks[chunks.len() - 1], Bytes::from_static(FINAL_CHUNK));
    }

    #[tokio::test]
    async fn response_returns_expected_format() {
        let bytes = TemplateWithFinalChunk::response("identifier", "<h1>html payload</h1>");
        let expected = Bytes::from_static(
            br#"<htms-chunk target="identifier"><h1>html payload</h1></htms-chunk>
"#,
        );

        assert_eq!(bytes, expected);
    }
}
