use std::time::Duration;

use futures_util::StreamExt;
use htms_derive::Template;
use tokio::{
    io::{AsyncWriteExt, stdout},
    time::sleep,
};

#[derive(Template, Debug, Default)]
#[template = "examples/template_include/index.html"]
struct TemplateIncludeExample {}

impl TemplateIncludeExampleRender for TemplateIncludeExample {
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
    let example = TemplateIncludeExample::default();
    let mut stream = Box::pin(example.render());

    while let Some(bytes) = stream.next().await {
        stdout.write_all(&bytes).await.unwrap();
        stdout.flush().await.unwrap();
    }
}
