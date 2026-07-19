// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration tests for the /ws WebSocket JSON-RPC route on the web server.

use super::*;

use std::sync::Arc;
use std::time::Duration;

use crate::data_service::DataService;

fn reserve_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind")
        .local_addr()
        .expect("addr")
        .port()
}

async fn wait_for_ready(port: u16) -> bool {
    let deadline = tokio::time::Instant::now() + Duration::from_millis(2000);
    while tokio::time::Instant::now() < deadline {
        if tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .is_ok()
        {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    false
}

async fn spawn_server(port: u16) {
    let data_service = Arc::new(DataService::new());
    let bind: String = format!("127.0.0.1:{port}");
    let cfg = WebConfig {
        bind: &bind,
        scenario: None,
        docroot: None,
        backend: "filesystem",
        workers: 2,
        strip_sources: false,
        cache_ttl_secs: 0,
        spa: false,
        allowed_origins: Vec::new(),
        compositions: Vec::new(),
    };
    let _ = run(cfg, data_service).await;
}

#[tokio::test]
async fn ws_health_check_via_web_server() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    let port = reserve_port();
    tokio::task::spawn(spawn_server(port));

    assert!(wait_for_ready(port).await, "server should start");

    let url = format!("ws://127.0.0.1:{port}/ws");
    let (mut ws, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("WS connect");

    let req = r#"{"jsonrpc":"2.0","id":1,"method":"health.check","params":{}}"#;
    ws.send(Message::Text(req.to_owned())).await.expect("send");

    let resp = ws.next().await.expect("response").expect("no error");
    let v: serde_json::Value =
        serde_json::from_str(resp.to_text().expect("text")).expect("valid JSON");

    assert_eq!(v["id"], 1);
    assert_eq!(v["result"]["status"], "ok");
}

#[tokio::test]
async fn ws_metrics_via_web_server() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    let port = reserve_port();
    tokio::task::spawn(spawn_server(port));

    assert!(wait_for_ready(port).await, "server should start");

    let url = format!("ws://127.0.0.1:{port}/ws");
    let (mut ws, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("WS connect");

    let req = r#"{"jsonrpc":"2.0","id":2,"method":"pt.metrics","params":{}}"#;
    ws.send(Message::Text(req.to_owned())).await.expect("send");

    let resp = ws.next().await.expect("response").expect("no error");
    let v: serde_json::Value =
        serde_json::from_str(resp.to_text().expect("text")).expect("valid JSON");

    assert_eq!(v["id"], 2);
    assert!(v["result"]["cpu_count"].as_u64().unwrap_or(0) >= 1);
    assert!(v["result"]["source"].is_string());
}

#[tokio::test]
async fn ws_capabilities_via_web_server() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    let port = reserve_port();
    tokio::task::spawn(spawn_server(port));

    assert!(wait_for_ready(port).await, "server should start");

    let url = format!("ws://127.0.0.1:{port}/ws");
    let (mut ws, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("WS connect");

    let req = r#"{"jsonrpc":"2.0","id":3,"method":"capabilities.list","params":{}}"#;
    ws.send(Message::Text(req.to_owned())).await.expect("send");

    let resp = ws.next().await.expect("response").expect("no error");
    let v: serde_json::Value =
        serde_json::from_str(resp.to_text().expect("text")).expect("valid JSON");

    assert_eq!(v["id"], 3);
    let caps = v["result"]["capabilities"].as_array().expect("array");
    assert!(caps.iter().any(|c| c == "health.check"));
    assert!(caps.iter().any(|c| c == "pt.metrics"));
    assert!(caps.iter().any(|c| c == "pt.render_svg"));
}
