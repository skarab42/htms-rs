use axum::{
    Router,
    response::{IntoResponse, Response},
    routing::get,
    serve,
};
use color_eyre::eyre::Result;
use htms::{Render, axum::HtmlStream};
use tokio::net;

use crate::index::Dashboard;

#[path = "pages/index.rs"]
mod index;

async fn news_portal_handler() -> Response {
    let stream = Dashboard::default().render();

    HtmlStream::from(stream).into_response()
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().route("/", get(news_portal_handler));
    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("listening on {}", listener.local_addr()?);
    serve(listener, app).await?;

    Ok(())
}
