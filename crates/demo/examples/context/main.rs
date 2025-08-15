use std::time::Duration;

use futures_util::StreamExt;
use htms_core::Render;
use htms_macros::htms;
use tokio::{
    io::{AsyncWriteExt, stdout},
    time::sleep,
};

#[derive(Debug)]
struct Context {
    title: String,
}

#[derive(Debug)]
#[htms(template = "examples/context/index.html")]
struct ContextExample {
    context: Context,
}

impl ContextExampleRender for ContextExample {
    async fn blog_posts_task() -> String {
        sleep(Duration::from_millis(2000)).await;
        "<p>Some blog posts here :)</p>".to_string()
    }

    async fn news_task() -> String {
        sleep(Duration::from_millis(1000)).await;
        "<p>Some news here :)</p>".to_string()
    }
}

#[tokio::main]
async fn main() {
    let mut stdout = stdout();
    let mut stream = Box::pin(ContextExample::render());

    while let Some(bytes) = stream.next().await {
        stdout.write_all(&bytes).await.unwrap();
        stdout.flush().await.unwrap();
    }
}
