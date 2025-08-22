use std::convert::Infallible;

use axum::{
    BoxError,
    body::{Body, Bytes},
    http::header::{CONTENT_TYPE, TRANSFER_ENCODING},
    response::{IntoResponse, Response},
};
use futures_core::{Stream, TryStream};
use futures_util::{StreamExt, stream::Map};

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

impl<S, E> From<S> for HtmlStream<S>
where
    S: TryStream<Ok = Bytes, Error = E> + Send + 'static,
    E: Into<BoxError> + 'static,
{
    fn from(inner: S) -> Self {
        Self(inner)
    }
}

impl<S> From<S> for HtmlStream<Map<S, fn(Bytes) -> Result<Bytes, Infallible>>>
where
    S: Stream<Item = Bytes> + Send + 'static,
{
    fn from(stream: S) -> Self {
        Self(stream.map(Ok::<Bytes, Infallible>))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use axum::{
        body::{Bytes, to_bytes},
        http::header::{CONTENT_TYPE, TRANSFER_ENCODING},
        response::IntoResponse,
    };
    use futures_util::stream;
    use http::HeaderValue;

    use super::HtmlStream;

    fn hello_world_chunks() -> Vec<Result<Bytes, axum::BoxError>> {
        vec![
            Ok::<Bytes, axum::BoxError>(Bytes::from_static(b"Hel")),
            Ok::<Bytes, axum::BoxError>(Bytes::from_static(b"lo")),
            Ok::<Bytes, axum::BoxError>(Bytes::from_static(b" ")),
            Ok::<Bytes, axum::BoxError>(Bytes::from_static(b"World")),
            Ok::<Bytes, axum::BoxError>(Bytes::from_static(b"!")),
        ]
    }

    #[tokio::test]
    async fn html_stream_sets_expected_headers() {
        let stream = stream::iter(hello_world_chunks());
        let response = HtmlStream(stream).into_response();
        let headers = response.headers();

        assert_eq!(
            headers.get(CONTENT_TYPE),
            Some(&HeaderValue::from_static("text/html; charset=utf-8"))
        );

        assert_eq!(
            headers.get(TRANSFER_ENCODING),
            Some(&HeaderValue::from_static("chunked"))
        );
    }

    #[tokio::test]
    async fn html_stream_streams_body_chunks_in_order() {
        let stream = stream::iter(hello_world_chunks());
        let response = HtmlStream(stream).into_response();
        let body = response.into_body();
        let bytes = to_bytes(body, usize::MAX).await.expect("to_bytes failed");

        assert_eq!(&bytes[..], b"Hello World!");
    }

    #[tokio::test]
    async fn from_impl_wraps_stream() {
        let stream = stream::iter(hello_world_chunks());
        let html_stream = HtmlStream::from(stream);
        let response = html_stream.into_response();
        let body = response.into_body();
        let bytes = to_bytes(body, usize::MAX).await.expect("to_bytes failed");

        assert_eq!(&bytes[..], b"Hello World!");
    }

    #[tokio::test]
    async fn from_plain_stream_bytes() {
        let stream = stream::iter([Bytes::from_static(b"Hello"), Bytes::from_static(b"World")]);
        let html_stream = HtmlStream::from(stream).into_response();
        let response = html_stream.into_response();
        let body = response.into_body();
        let bytes = to_bytes(body, usize::MAX).await.expect("to_bytes failed");

        assert_eq!(&bytes[..], b"HelloWorld");
    }
}
