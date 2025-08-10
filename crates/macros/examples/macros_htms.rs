use htms_macros::htms;
use htms_template::Template;

#[derive(Debug)]
#[htms(template = "examples/index.html")]
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
