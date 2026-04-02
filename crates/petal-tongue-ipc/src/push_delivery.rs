// SPDX-License-Identifier: AGPL-3.0-or-later
//! Push delivery for callback dispatches (PT-06).
//!
//! When a subscriber registers with both `callback_method` and `callback_socket`,
//! interaction events are delivered as JSON-RPC notifications to the subscriber's
//! socket instead of requiring poll. This completes the healthSpring V12 callback
//! pattern.
//!
//! ## Transport
//!
//! Notifications are newline-delimited JSON-RPC 2.0 (no `id` field per spec).
//! Socket paths starting with `/` are treated as Unix domain sockets; all others
//! are treated as `host:port` TCP addresses.

use crate::visualization_handler::CallbackDispatch;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

/// Spawn the push delivery background loop.
///
/// Returns a sender that RPC handlers use to enqueue dispatches, and a
/// `JoinHandle` for the background task. The task runs until the sender
/// is dropped.
pub fn spawn_push_delivery() -> (
    mpsc::UnboundedSender<CallbackDispatch>,
    tokio::task::JoinHandle<()>,
) {
    let (tx, rx) = mpsc::unbounded_channel();
    let handle = tokio::spawn(run_push_delivery_loop(rx));
    (tx, handle)
}

/// Background loop: drains the channel and delivers each callback.
async fn run_push_delivery_loop(mut rx: mpsc::UnboundedReceiver<CallbackDispatch>) {
    while let Some(dispatch) = rx.recv().await {
        let Some(ref socket) = dispatch.callback_socket else {
            tracing::trace!(
                subscriber = %dispatch.subscriber_id,
                "callback dispatch has no socket — skipping push (poll-only)"
            );
            continue;
        };

        if let Err(e) = deliver_notification(socket, &dispatch).await {
            tracing::warn!(
                subscriber = %dispatch.subscriber_id,
                socket = %socket,
                error = %e,
                "push delivery failed — subscriber should fall back to poll"
            );
        }
    }
    tracing::debug!("push delivery loop exiting (channel closed)");
}

/// Send a JSON-RPC notification to a subscriber socket.
///
/// Notifications have no `id` per JSON-RPC 2.0 spec. Best-effort: a failure
/// here does not affect the caller's response; the subscriber can still poll.
async fn deliver_notification(
    socket_addr: &str,
    dispatch: &CallbackDispatch,
) -> Result<(), std::io::Error> {
    let notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": dispatch.method,
        "params": {
            "subscriber_id": dispatch.subscriber_id,
            "events": dispatch.events,
        }
    });

    let mut buf = serde_json::to_vec(&notification)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    buf.push(b'\n');

    if socket_addr.starts_with('/') {
        let mut stream = tokio::net::UnixStream::connect(socket_addr).await?;
        stream.write_all(&buf).await?;
        stream.flush().await?;
    } else {
        let mut stream = tokio::net::TcpStream::connect(socket_addr).await?;
        stream.write_all(&buf).await?;
        stream.flush().await?;
    }

    tracing::debug!(
        subscriber = %dispatch.subscriber_id,
        method = %dispatch.method,
        events = dispatch.events.len(),
        "pushed callback notification"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization_handler::InteractionEventNotification;
    use tokio::io::AsyncBufReadExt;

    #[tokio::test]
    async fn push_delivery_to_unix_socket() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("test-push.sock");
        let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");

        let dispatch = CallbackDispatch {
            subscriber_id: "spring-1".to_string(),
            method: "spring.on_interaction".to_string(),
            events: vec![InteractionEventNotification {
                event_type: "select".to_string(),
                targets: vec!["node-1".to_string()],
                timestamp: "2026-04-02T00:00:00Z".to_string(),
                perspective_id: None,
            }],
            callback_socket: Some(sock_path.to_string_lossy().into_owned()),
        };

        let (tx, handle) = spawn_push_delivery();
        tx.send(dispatch).expect("send");

        let (stream, _) = listener.accept().await.expect("accept");
        let mut reader = tokio::io::BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("read");

        drop(tx);
        handle.await.expect("join");

        let parsed: serde_json::Value = serde_json::from_str(&line).expect("parse");
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["method"], "spring.on_interaction");
        assert_eq!(parsed["params"]["subscriber_id"], "spring-1");
        assert!(parsed.get("id").is_none());
    }

    #[tokio::test]
    async fn push_delivery_skips_no_socket() {
        let dispatch = CallbackDispatch {
            subscriber_id: "poll-only".to_string(),
            method: "cb".to_string(),
            events: vec![],
            callback_socket: None,
        };

        let (tx, handle) = spawn_push_delivery();
        tx.send(dispatch).expect("send");
        drop(tx);
        handle.await.expect("join");
    }

    #[tokio::test]
    async fn push_delivery_handles_unreachable_socket() {
        let dispatch = CallbackDispatch {
            subscriber_id: "gone".to_string(),
            method: "cb".to_string(),
            events: vec![],
            callback_socket: Some("/tmp/nonexistent-push-test.sock".to_string()),
        };

        let (tx, handle) = spawn_push_delivery();
        tx.send(dispatch).expect("send");
        drop(tx);
        handle.await.expect("join");
    }

    #[tokio::test]
    async fn push_delivery_to_tcp() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("local_addr").to_string();

        let dispatch = CallbackDispatch {
            subscriber_id: "tcp-sub".to_string(),
            method: "spring.on_event".to_string(),
            events: vec![InteractionEventNotification {
                event_type: "hover".to_string(),
                targets: vec![],
                timestamp: "2026-04-02T00:00:00Z".to_string(),
                perspective_id: Some(1),
            }],
            callback_socket: Some(addr),
        };

        let (tx, handle) = spawn_push_delivery();
        tx.send(dispatch).expect("send");

        let (stream, _) = listener.accept().await.expect("accept");
        let mut reader = tokio::io::BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("read");

        drop(tx);
        handle.await.expect("join");

        let parsed: serde_json::Value = serde_json::from_str(&line).expect("parse");
        assert_eq!(parsed["method"], "spring.on_event");
        assert_eq!(parsed["params"]["events"][0]["event_type"], "hover");
    }
}
