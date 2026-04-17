// SPDX-License-Identifier: AGPL-3.0-or-later
//! Thin HTTP client for local/LAN communication (plain HTTP, no TLS).
//!
//! Built on `hyper` + `hyper-util` — both already in the dep tree from `axum`.
//! petalTongue never owns a TLS stack; external HTTPS is delegated to Songbird
//! via tower atomic IPC.
//!
//! This module replaces `reqwest` for all local HTTP needs (biomeOS API,
//! mDNS discovery, entropy streaming, SSE consumption, etc.).

use bytes::Bytes;
use http::StatusCode;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::time::Duration;
use thiserror::Error;

type HyperClient = Client<hyper_util::client::legacy::connect::HttpConnector, Full<Bytes>>;

/// Errors from local HTTP operations.
#[derive(Debug, Error)]
pub enum HttpClientError {
    /// Failed to parse URI.
    #[error("invalid URI: {0}")]
    InvalidUri(String),

    /// HTTP request/connection error.
    #[error("HTTP request failed: {0}")]
    Request(String),

    /// Timeout waiting for response.
    #[error("HTTP request timed out after {0:?}")]
    Timeout(Duration),

    /// Failed to read response body.
    #[error("failed to read response body: {0}")]
    Body(String),

    /// JSON deserialization error.
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

/// HTTP response wrapper with convenience methods.
#[derive(Debug)]
pub struct HttpResponse {
    status: StatusCode,
    body: Bytes,
}

impl HttpResponse {
    /// HTTP status code.
    #[must_use]
    pub const fn status(&self) -> StatusCode {
        self.status
    }

    /// Whether the status is 2xx.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Consume the response as raw bytes.
    #[must_use]
    pub fn bytes(self) -> Bytes {
        self.body
    }

    /// Consume the response as UTF-8 text.
    ///
    /// # Errors
    ///
    /// Returns `HttpClientError::Body` if the body is not valid UTF-8.
    pub fn text(self) -> Result<String, HttpClientError> {
        String::from_utf8(self.body.to_vec())
            .map_err(|e| HttpClientError::Body(format!("invalid UTF-8: {e}")))
    }

    /// Deserialize the response body as JSON.
    ///
    /// # Errors
    ///
    /// Returns `HttpClientError::Json` on deserialization failure.
    pub fn json<T: serde::de::DeserializeOwned>(self) -> Result<T, HttpClientError> {
        Ok(serde_json::from_slice(&self.body)?)
    }
}

/// Thin HTTP client for plain-HTTP local/LAN communication.
///
/// Wraps `hyper-util`'s connection-pooling client. Supports GET and POST with
/// configurable timeouts. No TLS — Songbird handles that via IPC relay.
#[derive(Clone)]
pub struct LocalHttpClient {
    client: HyperClient,
    timeout: Duration,
}

impl LocalHttpClient {
    /// Default request timeout (10 seconds).
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

    /// Create a new client with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: Client::builder(TokioExecutor::new()).build_http(),
            timeout: Self::DEFAULT_TIMEOUT,
        }
    }

    /// Create a client with a custom timeout.
    #[must_use]
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            client: Client::builder(TokioExecutor::new()).build_http(),
            timeout,
        }
    }

    /// Send a GET request.
    ///
    /// # Errors
    ///
    /// Returns `HttpClientError` on invalid URI, network failure, timeout, or body read error.
    pub async fn get(&self, url: &str) -> Result<HttpResponse, HttpClientError> {
        let uri: hyper::Uri = url
            .parse()
            .map_err(|e: http::uri::InvalidUri| HttpClientError::InvalidUri(e.to_string()))?;

        let request = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(uri)
            .body(Full::new(Bytes::new()))
            .map_err(|e| HttpClientError::Request(e.to_string()))?;

        self.send(request).await
    }

    /// Send a GET request with custom headers.
    ///
    /// # Errors
    ///
    /// Returns `HttpClientError` on invalid URI, network failure, timeout, or body read error.
    pub async fn get_with_headers(
        &self,
        url: &str,
        headers: &[(&str, &str)],
    ) -> Result<HttpResponse, HttpClientError> {
        let uri: hyper::Uri = url
            .parse()
            .map_err(|e: http::uri::InvalidUri| HttpClientError::InvalidUri(e.to_string()))?;

        let mut builder = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(uri);
        for &(name, value) in headers {
            builder = builder.header(name, value);
        }

        let request = builder
            .body(Full::new(Bytes::new()))
            .map_err(|e| HttpClientError::Request(e.to_string()))?;

        self.send(request).await
    }

    /// Send a POST request with a JSON body.
    ///
    /// # Errors
    ///
    /// Returns `HttpClientError` on invalid URI, serialization failure,
    /// network failure, timeout, or body read error.
    pub async fn post_json<T: serde::Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<HttpResponse, HttpClientError> {
        let json_bytes = serde_json::to_vec(body)?;
        self.post_raw(url, json_bytes, "application/json").await
    }

    /// Send a POST request with custom headers and a JSON body.
    ///
    /// # Errors
    ///
    /// Returns `HttpClientError` on invalid URI, serialization failure,
    /// network failure, timeout, or body read error.
    pub async fn post_json_with_headers<T: serde::Serialize>(
        &self,
        url: &str,
        body: &T,
        headers: &[(&str, &str)],
    ) -> Result<HttpResponse, HttpClientError> {
        let json_bytes = serde_json::to_vec(body)?;

        let uri: hyper::Uri = url
            .parse()
            .map_err(|e: http::uri::InvalidUri| HttpClientError::InvalidUri(e.to_string()))?;

        let mut builder = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(uri)
            .header("content-type", "application/json");
        for &(name, value) in headers {
            builder = builder.header(name, value);
        }

        let request = builder
            .body(Full::new(Bytes::from(json_bytes)))
            .map_err(|e| HttpClientError::Request(e.to_string()))?;

        self.send(request).await
    }

    /// Send a POST request with raw bytes and a content type.
    ///
    /// # Errors
    ///
    /// Returns `HttpClientError` on invalid URI, network failure, timeout, or body read error.
    pub async fn post_raw(
        &self,
        url: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<HttpResponse, HttpClientError> {
        let uri: hyper::Uri = url
            .parse()
            .map_err(|e: http::uri::InvalidUri| HttpClientError::InvalidUri(e.to_string()))?;

        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(uri)
            .header("content-type", content_type)
            .body(Full::new(Bytes::from(body)))
            .map_err(|e| HttpClientError::Request(e.to_string()))?;

        self.send(request).await
    }

    /// Internal: send a request with timeout, collect body into `HttpResponse`.
    async fn send(
        &self,
        request: hyper::Request<Full<Bytes>>,
    ) -> Result<HttpResponse, HttpClientError> {
        let response = tokio::time::timeout(self.timeout, self.client.request(request))
            .await
            .map_err(|_| HttpClientError::Timeout(self.timeout))?
            .map_err(|e| HttpClientError::Request(e.to_string()))?;

        let status = response.status();
        let body = collect_body(response.into_body()).await?;

        Ok(HttpResponse { status, body })
    }
}

impl Default for LocalHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Send a one-shot GET request (convenience, no connection pooling).
///
/// # Errors
///
/// Returns `HttpClientError` on any failure.
pub async fn http_get(url: &str) -> Result<HttpResponse, HttpClientError> {
    LocalHttpClient::new().get(url).await
}

/// Collect a `hyper::body::Incoming` into `Bytes`.
async fn collect_body(body: Incoming) -> Result<Bytes, HttpClientError> {
    body.collect()
        .await
        .map(http_body_util::Collected::to_bytes)
        .map_err(|e| HttpClientError::Body(e.to_string()))
}

/// SSE (Server-Sent Events) streaming support.
///
/// Wraps a `hyper::body::Incoming` stream for incremental chunk reading.
pub struct SseStream {
    body: Incoming,
}

impl SseStream {
    /// Read the next chunk of bytes from the stream.
    ///
    /// Returns `None` when the stream is exhausted.
    ///
    /// # Errors
    ///
    /// Returns `HttpClientError::Body` on stream read failure.
    pub async fn next_chunk(&mut self) -> Result<Option<Bytes>, HttpClientError> {
        use http_body_util::BodyExt;
        match self.body.frame().await {
            Some(Ok(frame)) => Ok(frame.into_data().ok()),
            Some(Err(e)) => Err(HttpClientError::Body(e.to_string())),
            None => Ok(None),
        }
    }
}

impl LocalHttpClient {
    /// Send a GET request and return a streaming body (for SSE / chunked responses).
    ///
    /// # Errors
    ///
    /// Returns `HttpClientError` on invalid URI or connection failure.
    pub async fn get_stream(
        &self,
        url: &str,
        headers: &[(&str, &str)],
    ) -> Result<(StatusCode, SseStream), HttpClientError> {
        let uri: hyper::Uri = url
            .parse()
            .map_err(|e: http::uri::InvalidUri| HttpClientError::InvalidUri(e.to_string()))?;

        let mut builder = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(uri);
        for &(name, value) in headers {
            builder = builder.header(name, value);
        }

        let request = builder
            .body(Full::new(Bytes::new()))
            .map_err(|e| HttpClientError::Request(e.to_string()))?;

        let response = self
            .client
            .request(request)
            .await
            .map_err(|e| HttpClientError::Request(e.to_string()))?;

        let status = response.status();
        let body = response.into_body();

        Ok((status, SseStream { body }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_construction() {
        let _client = LocalHttpClient::new();
        let _client2 = LocalHttpClient::with_timeout(Duration::from_secs(5));
        let _client3 = LocalHttpClient::default();
    }

    #[test]
    fn http_response_status() {
        let resp = HttpResponse {
            status: StatusCode::OK,
            body: Bytes::from("hello"),
        };
        assert!(resp.is_success());
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[test]
    fn http_response_text() {
        let resp = HttpResponse {
            status: StatusCode::OK,
            body: Bytes::from("hello world"),
        };
        assert_eq!(resp.text().expect("valid utf8"), "hello world");
    }

    #[test]
    fn http_response_json() {
        let resp = HttpResponse {
            status: StatusCode::OK,
            body: Bytes::from(r#"{"key":"value"}"#),
        };
        let parsed: serde_json::Value = resp.json().expect("valid json");
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn http_response_bytes() {
        let resp = HttpResponse {
            status: StatusCode::OK,
            body: Bytes::from("raw bytes"),
        };
        assert_eq!(resp.bytes().as_ref(), b"raw bytes");
    }

    #[tokio::test]
    async fn get_invalid_uri_returns_error() {
        let client = LocalHttpClient::new();
        let result = client.get("not a valid uri!!").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, HttpClientError::InvalidUri(_)));
    }

    #[tokio::test]
    async fn get_connection_refused() {
        let client = LocalHttpClient::with_timeout(Duration::from_millis(200));
        let result = client.get("http://127.0.0.1:1").await;
        assert!(result.is_err());
    }

    #[test]
    fn error_display() {
        let err = HttpClientError::InvalidUri("bad".into());
        assert!(err.to_string().contains("invalid URI"));

        let err = HttpClientError::Timeout(Duration::from_secs(5));
        assert!(err.to_string().contains("timed out"));

        let err = HttpClientError::Body("read failed".into());
        assert!(err.to_string().contains("response body"));

        let err = HttpClientError::Request("conn refused".into());
        assert!(err.to_string().contains("request failed"));
    }
}
