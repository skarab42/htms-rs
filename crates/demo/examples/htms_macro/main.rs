use htms_core::render::Render;
use htms_macros::htms;

#[derive(Debug)]
#[htms(template = "examples/htms_macro/index.html")]
struct Index;

impl IndexRender for Index {
    async fn blog_posts_task() -> String {
        todo!()
    }

    async fn news_task() -> String {
        todo!()
    }
}

fn main() {
    let output = Index::render();
    println!("output: {}", output);
}
