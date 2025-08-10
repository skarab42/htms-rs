use htms_core::template::Template;
use htms_macros::htms;

#[derive(Debug)]
#[htms(template = "examples/htms_macro/index.html")]
struct Index;

impl IndexHtmsTemplate for Index {
    async fn news_task(&self) -> String {
        todo!()
    }

    async fn blog_posts_task(&self) -> String {
        todo!()
    }
}

fn main() {
    let index = Index {};
    println!("index: {index:?}");

    let output = index.render();
    println!("output: {}", output);
}
