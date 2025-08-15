use std::time::Duration;

use htms_macros::htms;
use tokio::time::sleep;

#[derive(Debug)]
#[htms(template = "examples/axum/pages/index.html")]
pub struct AxumExample;

impl AxumExampleRender for AxumExample {
    async fn blog_posts_task() -> String {
        sleep(Duration::from_millis(5000)).await;
        "<p>Some blog posts here :)</p>".to_string()
    }

    async fn news_task() -> String {
        sleep(Duration::from_millis(10000)).await;
        "<p>Some news here :)</p>".to_string()
    }
}
