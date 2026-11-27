// SPDX-License-Identifier: AGPL-3.0-or-later
//! The few HTTP requests the IDE window makes to its own server.
//!
//! The server is on this device, on the loopback address, and speaks plain
//! HTTP/1.1. One request per connection, JSON both ways. That is small
//! enough to say in full here, so the window brings no HTTP library.

use std::time::Duration;

use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// The header the server reads the session token from.
pub const TOKEN_HEADER: &str = "X-Gaia-Token";

/// How long a connection may take. The server is on this device.
const CONNECT: Duration = Duration::from_secs(3);
/// How long an answer may take: reading a file, listing a folder.
const ANSWER: Duration = Duration::from_secs(30);
/// The largest answer taken. The server itself refuses to read bigger files.
const MOST: usize = 64 * 1024 * 1024;

/// Why a request came to nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// The status the server answered with; `None` when it did not answer.
    pub status: Option<u16>,
    /// Why, in the server's words when it gave any.
    pub why: String,
}

impl Failure {
    fn unanswered(why: impl Into<String>) -> Self {
        Self {
            status: None,
            why: why.into(),
        }
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.why)
    }
}

impl From<Failure> for String {
    fn from(failure: Failure) -> Self {
        failure.why
    }
}

/// Where the server is and what to show at its door.
#[derive(Debug, Clone)]
pub struct Door {
    /// `127.0.0.1:3142`
    pub host: String,
    /// The session token.
    pub token: String,
}

impl Door {
    /// `GET path` (the path with its query); the JSON that came back.
    pub async fn get(&self, path: &str) -> Result<Value, Failure> {
        self.send("GET", path, None).await
    }

    /// `POST path` with a JSON body; the JSON that came back.
    pub async fn post(&self, path: &str, body: &Value) -> Result<Value, Failure> {
        self.send("POST", path, Some(body.to_string())).await
    }

    async fn send(&self, method: &str, path: &str, body: Option<String>) -> Result<Value, Failure> {
        let request = request(method, &self.host, path, &self.token, body.as_deref());
        let raw = tokio::time::timeout(ANSWER, exchange(&self.host, request.as_bytes()))
            .await
            .map_err(|_| Failure::unanswered("the IDE's server took too long to answer"))?
            .map_err(Failure::unanswered)?;
        let (status, body) = parse(&raw).map_err(Failure::unanswered)?;
        let json: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
        if (200..300).contains(&status) {
            return Ok(json);
        }
        // The server says why in `error`; say it as it did.
        Err(Failure {
            status: Some(status),
            why: match json["error"].as_str() {
                Some(why) => why.to_string(),
                None if status == 401 => "the session token was refused".to_string(),
                None => format!("the IDE's server answered {status}"),
            },
        })
    }
}

/// The request as it goes on the wire.
fn request(method: &str, host: &str, path: &str, token: &str, body: Option<&str>) -> String {
    let mut out = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}\r\n{TOKEN_HEADER}: {token}\r\nAccept: application/json\r\nConnection: close\r\n"
    );
    if let Some(body) = body {
        out.push_str("Content-Type: application/json\r\n");
        out.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    out.push_str("\r\n");
    out.push_str(body.unwrap_or_default());
    out
}

/// Send the request; everything the server wrote until it closed.
async fn exchange(host: &str, request: &[u8]) -> Result<Vec<u8>, String> {
    let mut stream = tokio::time::timeout(CONNECT, TcpStream::connect(host))
        .await
        .map_err(|_| format!("nothing answered at {host}"))?
        .map_err(|e| format!("nothing is listening at {host} ({e})"))?;
    stream
        .write_all(request)
        .await
        .map_err(|e| format!("the request could not be sent ({e})"))?;
    let mut raw = Vec::new();
    let mut piece = vec![0_u8; 64 * 1024];
    loop {
        let n = stream
            .read(&mut piece)
            .await
            .map_err(|e| format!("the answer broke off ({e})"))?;
        if n == 0 {
            return Ok(raw);
        }
        raw.extend_from_slice(&piece[..n]);
        if raw.len() > MOST {
            return Err("the answer is too large".to_string());
        }
    }
}

/// The status and the body of a whole HTTP/1.1 answer.
pub fn parse(raw: &[u8]) -> Result<(u16, Vec<u8>), String> {
    let broken = || "the IDE's server gave an answer that is not HTTP".to_string();
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(broken)?;
    let head = std::str::from_utf8(&raw[..split]).map_err(|_| broken())?;
    let body = &raw[split + 4..];

    let mut lines = head.split("\r\n");
    let status: u16 = lines
        .next()
        .and_then(|line| line.split(' ').nth(1))
        .and_then(|code| code.parse().ok())
        .ok_or_else(broken)?;

    let mut length = None;
    let mut chunked = false;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        if name.eq_ignore_ascii_case("content-length") {
            length = value.parse::<usize>().ok();
        } else if name.eq_ignore_ascii_case("transfer-encoding") {
            chunked = value.to_ascii_lowercase().contains("chunked");
        }
    }

    if chunked {
        return Ok((status, unchunk(body).ok_or_else(broken)?));
    }
    match length {
        Some(n) if body.len() < n => Err("the answer broke off before its end".to_string()),
        Some(n) => Ok((status, body[..n].to_vec())),
        None => Ok((status, body.to_vec())),
    }
}

/// Put a chunked body back together. `None` when it is cut short.
fn unchunk(mut rest: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    loop {
        let line_end = rest.windows(2).position(|w| w == b"\r\n")?;
        let size_line = std::str::from_utf8(&rest[..line_end]).ok()?;
        let size = usize::from_str_radix(size_line.split(';').next()?.trim(), 16).ok()?;
        rest = &rest[line_end + 2..];
        if size == 0 {
            return Some(out);
        }
        if rest.len() < size + 2 {
            return None;
        }
        out.extend_from_slice(&rest[..size]);
        rest = &rest[size + 2..];
    }
}

/// A path made safe to put in a query: `?path=` followed by this.
pub fn query(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~' | b'/') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_answer_with_a_length_is_read_to_that_length() {
        let (status, body) =
            parse(b"HTTP/1.1 200 OK\r\ncontent-length: 7\r\n\r\n{\"a\":1}").expect("parses");
        assert_eq!((status, body.as_slice()), (200, b"{\"a\":1}".as_slice()));
    }

    #[test]
    fn an_answer_cut_short_is_said_to_be_cut_short() {
        let cut = parse(b"HTTP/1.1 200 OK\r\nContent-Length: 70\r\n\r\n{\"a\":1}");
        assert_eq!(cut, Err("the answer broke off before its end".to_string()));
    }

    #[test]
    fn a_chunked_answer_is_put_back_together() {
        let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\n{\"a\"\r\n3;x=y\r\n:1}\r\n0\r\n\r\n";
        let (status, body) = parse(raw).expect("parses");
        assert_eq!((status, body.as_slice()), (200, b"{\"a\":1}".as_slice()));
    }

    #[test]
    fn a_chunked_answer_without_its_end_is_refused() {
        let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\n{\"a\"\r\n";
        assert!(parse(raw).is_err());
    }

    #[test]
    fn what_is_not_http_is_refused() {
        assert!(parse(b"hello").is_err());
        assert!(parse(b"nonsense\r\n\r\n").is_err());
    }

    #[test]
    fn a_path_with_spaces_and_signs_is_safe_in_a_query() {
        assert_eq!(
            query("/home/k/a b&c=d#e.rs"),
            "/home/k/a%20b%26c%3Dd%23e.rs"
        );
        assert_eq!(query("/tmp/ünï"), "/tmp/%C3%BCn%C3%AF");
    }

    #[test]
    fn the_request_carries_the_host_the_token_and_the_length() {
        let sent = request("POST", "127.0.0.1:3142", "/api/fs/write", "tok", Some("{}"));
        assert!(sent.starts_with("POST /api/fs/write HTTP/1.1\r\nHost: 127.0.0.1:3142\r\n"));
        assert!(sent.contains("X-Gaia-Token: tok\r\n"));
        assert!(sent.ends_with("Content-Length: 2\r\n\r\n{}"));
    }

    /// A whole request against a server that answers once and closes.
    #[tokio::test]
    async fn a_refusal_is_told_in_the_servers_words() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("binds");
        let host = listener.local_addr().expect("has an address").to_string();
        tokio::spawn(async move {
            let (mut caller, _) = listener.accept().await.expect("accepts");
            let mut asked = vec![0_u8; 4096];
            let _ = caller.read(&mut asked).await;
            let body = r#"{"error":"outside the folders this IDE may open"}"#;
            let answer = format!(
                "HTTP/1.1 403 Forbidden\r\ncontent-length: {}\r\n\r\n{body}",
                body.len()
            );
            let _ = caller.write_all(answer.as_bytes()).await;
        });
        let door = Door {
            host,
            token: "t".to_string(),
        };
        let refused = door.get("/api/fs/read?path=/etc/shadow").await;
        assert_eq!(
            refused,
            Err(Failure {
                status: Some(403),
                why: "outside the folders this IDE may open".to_string()
            })
        );
    }

    #[tokio::test]
    async fn no_server_is_said_plainly() {
        // A port nothing listens on: bind, note the address, let it go.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("binds");
        let host = listener.local_addr().expect("has an address").to_string();
        drop(listener);
        let door = Door {
            host: host.clone(),
            token: "t".to_string(),
        };
        let said = door.get("/api/boot").await.expect_err("nothing is there");
        assert_eq!(said.status, None);
        assert!(
            said.why
                .starts_with(&format!("nothing is listening at {host}")),
            "{said}"
        );
    }
}
