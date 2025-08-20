use std::time::Duration;

use htms::Template;
use tokio::time::sleep;

use crate::state::AppState;

#[derive(Template, Debug)]
#[template = "examples/axum_with_state/pages/index.html"]
pub struct AxumWithStateExample {
    #[context]
    pub state: AppState,
}

impl AxumWithStateExampleRender for AxumWithStateExample {
    async fn blog_posts_task(state: AppState) -> String {
        sleep(Duration::from_millis(2000)).await;
        format!("<h1>{}</h1><p>Some blog posts here :)</p>", state.title)
    }

    async fn news_task(state: AppState) -> String {
        sleep(Duration::from_millis(1000)).await;
        format!("<h1>{}</h1><p>Some news here :)</p>", state.title)
    }
}
