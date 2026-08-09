// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tests for the /ws/scene WebSocket scene stream (G19).

use crate::data_service::DataService;
use crate::web_mode::scene_stream::{SceneFrame, SceneStreamState};
use std::sync::Arc;
use std::time::Duration;

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

#[tokio::test]
async fn scene_stream_subscribe_ack() {
    let port = reserve_port();
    let bind = format!("127.0.0.1:{port}");
    let data_service = Arc::new(DataService::new());

    let (scene_tx, _) = tokio::sync::broadcast::channel::<SceneFrame>(16);
    let scene_state = SceneStreamState {
        data_service: Arc::clone(&data_service),
        scene_tx,
    };

    let app = axum::Router::new()
        .route(
            "/ws/scene",
            axum::routing::get(crate::web_mode::scene_stream::scene_stream_handler),
        )
        .with_state(scene_state);

    let listener = tokio::net::TcpListener::bind(&bind).await.unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    assert!(wait_for_ready(port).await, "server never became ready");

    let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{port}/ws/scene"))
        .await
        .expect("WS connect");

    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite;

    ws.send(tungstenite::Message::Text(
        r#"{"action":"subscribe","session_id":"test-sess"}"#.to_owned(),
    ))
    .await
    .unwrap();

    let msg = tokio::time::timeout(Duration::from_secs(2), ws.next())
        .await
        .expect("timeout")
        .expect("stream end")
        .expect("msg");

    let text = msg.into_text().unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["type"], "subscribed");
    assert_eq!(v["session_id"], "test-sess");
    assert_eq!(v["modality"], "webgl");
}

#[tokio::test]
async fn scene_stream_receives_published_frame() {
    let port = reserve_port();
    let bind = format!("127.0.0.1:{port}");
    let data_service = Arc::new(DataService::new());

    let (scene_tx, _) = tokio::sync::broadcast::channel::<SceneFrame>(16);
    let scene_state = SceneStreamState {
        data_service: Arc::clone(&data_service),
        scene_tx: scene_tx.clone(),
    };

    let app = axum::Router::new()
        .route(
            "/ws/scene",
            axum::routing::get(crate::web_mode::scene_stream::scene_stream_handler),
        )
        .with_state(scene_state);

    let listener = tokio::net::TcpListener::bind(&bind).await.unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    assert!(wait_for_ready(port).await, "server never became ready");

    let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{port}/ws/scene"))
        .await
        .expect("WS connect");

    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite;

    ws.send(tungstenite::Message::Text(
        r#"{"action":"subscribe","session_id":"live"}"#.to_owned(),
    ))
    .await
    .unwrap();

    // Read subscribe ack
    let _ack = tokio::time::timeout(Duration::from_secs(2), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    // Publish a frame
    let payload = r#"{"vertices":[0.0,1.0],"indices":[0],"draw_calls":[],"view_projection":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"viewport":[800,600]}"#;
    crate::web_mode::scene_stream::publish_scene_frame(&scene_tx, "live", payload.to_owned());

    // Should receive the frame
    let msg = tokio::time::timeout(Duration::from_secs(2), ws.next())
        .await
        .expect("timeout")
        .expect("stream end")
        .expect("msg");

    let text = msg.into_text().unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["type"], "frame");
    assert_eq!(v["session_id"], "live");
    assert!(v["scene"]["vertices"].is_array());
}
