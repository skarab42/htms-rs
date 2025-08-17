use std::time::Duration;

use futures_util::StreamExt;
use htms_derive::Template;
use tokio::{
    io::{AsyncWriteExt, stdout},
    time::sleep,
};

#[derive(Debug, Clone)]
struct Context {
    title: String,
}

#[derive(Template, Debug)]
#[template = "examples/derive_with_context/index.html"]
struct DeriveWithContextExample {
    context: Context,
}

impl DeriveWithContextExampleRender for DeriveWithContextExample {
    async fn blog_posts_task(context: Context) -> String {
        sleep(Duration::from_millis(2000)).await;
        format!("<h1>{}</h1><p>Some blog posts here :)</p>", context.title)
    }

    async fn news_task(context: Context) -> String {
        sleep(Duration::from_millis(1000)).await;
        format!("<h1>{}</h1><p>Some news here :)</p>", context.title)
    }
}

#[tokio::main]
async fn main() {
    let mut stdout = stdout();
    let example = DeriveWithContextExample {
        context: Context {
            title: "Hello World".to_string(),
        },
    };
    let mut stream = Box::pin(example.render());

    while let Some(bytes) = stream.next().await {
        stdout.write_all(&bytes).await.unwrap();
        stdout.flush().await.unwrap();
    }
}
