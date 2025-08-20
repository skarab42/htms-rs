use std::convert::Infallible;

use axum::{
    Router,
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    serve,
};
use color_eyre::eyre::Result;
use futures_util::StreamExt;
use htms::{Bytes, Render, axum::HtmlStream};
use tokio::net;

use crate::{index::AxumWithStateExample, state::AppState};

#[path = "pages/index.rs"]
mod index;
mod state;

async fn handler(State(state): State<AppState>) -> Response {
    let stream = AxumWithStateExample { state }
        .render()
        .map(Ok::<Bytes, Infallible>);

    HtmlStream::from(stream).into_response()
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState {
        title: "App title".into(),
    };

    let app = Router::new().route("/", get(handler)).with_state(state);
    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("listening on {}", listener.local_addr()?);
    serve(listener, app).await?;

    Ok(())
}
