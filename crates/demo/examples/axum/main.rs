use std::convert::Infallible;

use axum::{
    Router,
    response::{IntoResponse, Response},
    routing::get,
    serve,
};
use color_eyre::eyre::Result;
use futures_util::StreamExt;
use htms_core::{Bytes, Render, axum::HtmlStream};
use tokio::net;

use crate::index::AxumExample;

#[path = "pages/index.rs"]
mod index;

async fn handler() -> Response {
    let stream = AxumExample::default().render().map(Ok::<Bytes, Infallible>);

    HtmlStream::from(stream).into_response()
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().route("/", get(handler));
    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("listening on {}", listener.local_addr()?);
    serve(listener, app).await?;

    Ok(())
}
