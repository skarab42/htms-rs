use axum::{
    BoxError,
    body::{Body, Bytes},
    http::header::{CONTENT_TYPE, TRANSFER_ENCODING},
    response::{IntoResponse, Response},
};
use futures_core::TryStream;

pub struct HtmlStream<S>(pub S);

impl<S> IntoResponse for HtmlStream<S>
where
    S: TryStream + Send + 'static,
    S::Ok: Into<Bytes>,
    S::Error: Into<BoxError>,
{
    fn into_response(self) -> Response {
        (
            [
                (CONTENT_TYPE, "text/html; charset=utf-8"),
                (TRANSFER_ENCODING, "chunked"), // optional, since axum add it automatically if content-length is omitted
            ],
            Body::from_stream(self.0),
        )
            .into_response()
    }
}

impl<S> From<S> for HtmlStream<S> {
    fn from(inner: S) -> Self {
        Self(inner)
    }
}
