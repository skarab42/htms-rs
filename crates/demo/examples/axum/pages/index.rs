use std::time::Duration;

use htms_derive::Template;
use tokio::time::sleep;

#[derive(Template, Debug, Default)]
#[template = "examples/axum/pages/index.html"]
pub struct AxumExample {}

impl AxumExampleRender for AxumExample {
    async fn blog_posts_task() -> String {
        sleep(Duration::from_millis(2000)).await;
        "<p>Some blog posts here :)</p>".to_string()
    }

    async fn news_task() -> String {
        sleep(Duration::from_millis(4000)).await;
        "<p>Some news here :)</p>".to_string()
    }
}
