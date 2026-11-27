// SPDX-License-Identifier: AGPL-3.0-or-later
//! The line to the server, against a stand-in for the server.
//!
//! The stand-in listens on a loopback port of its own, answers the few
//! paths the window asks, and keeps what it was asked. It checks the
//! session token the way the real server does.

use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

use super::super::state::{Entry, Event, Model, NotSaved};
use super::*;

/// What the stand-in was asked: `METHOD path body`.
type Asked = Arc<Mutex<Vec<String>>>;

struct StandIn {
    place: Place,
    asked: Asked,
    /// Keeps the stand-in's own threads alive for the length of the test.
    _runtime: tokio::runtime::Runtime,
    _folder: tempfile::TempDir,
}

impl StandIn {
    fn asked(&self) -> Vec<String> {
        self.asked.lock().expect("not poisoned").clone()
    }

    fn set_token(&self, token: &str) {
        std::fs::write(&self.place.token_file, token).expect("the token file can be written");
    }
}

/// How the stand-in behaves.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// As the server of today.
    Current,
    /// As a server from before `/api/boot`.
    Older,
}

fn stand_in(kind: Kind) -> StandIn {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("a runtime");
    let listener = runtime
        .block_on(TcpListener::bind("127.0.0.1:0"))
        .expect("a loopback port");
    let host = listener.local_addr().expect("an address").to_string();
    let folder = tempfile::tempdir().expect("a folder");
    let place = Place {
        host,
        token_file: folder.path().join("gaia-ide.token"),
    };
    std::fs::write(&place.token_file, "first-token\n").expect("the token file can be written");

    let asked = Asked::default();
    let (kept, token_file) = (Arc::clone(&asked), place.token_file.clone());
    runtime.spawn(async move {
        while let Ok((caller, _)) = listener.accept().await {
            let (kept, token_file) = (Arc::clone(&kept), token_file.clone());
            tokio::spawn(async move {
                // The token as it is now: the real server has one per start.
                let token = std::fs::read_to_string(&token_file)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                serve(caller, kind, &token, &kept).await;
            });
        }
    });
    StandIn {
        place,
        asked,
        _runtime: runtime,
        _folder: folder,
    }
}

async fn serve(mut caller: TcpStream, kind: Kind, token: &str, asked: &Asked) {
    // Look without taking: a WebSocket's handshake must be read by the
    // WebSocket code itself.
    let mut start = [0_u8; 32];
    let seen = caller.peek(&mut start).await.unwrap_or(0);
    if start[..seen].starts_with(b"GET /api/agent") {
        agent(caller, token, asked).await;
        return;
    }

    let mut raw = Vec::new();
    let mut piece = [0_u8; 4096];
    let (head_end, length) = loop {
        let n = caller.read(&mut piece).await.unwrap_or(0);
        if n == 0 {
            return;
        }
        raw.extend_from_slice(&piece[..n]);
        if let Some(at) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&raw[..at]).to_lowercase();
            let length = head
                .lines()
                .find_map(|line| line.strip_prefix("content-length:"))
                .and_then(|v| v.trim().parse::<usize>().ok())
                .unwrap_or(0);
            break (at + 4, length);
        }
    };
    while raw.len() < head_end + length {
        let n = caller.read(&mut piece).await.unwrap_or(0);
        if n == 0 {
            return;
        }
        raw.extend_from_slice(&piece[..n]);
    }
    let head = String::from_utf8_lossy(&raw[..head_end]).to_string();
    let body = String::from_utf8_lossy(&raw[head_end..]).to_string();
    let mut request_line = head.lines().next().unwrap_or_default().split(' ');
    let method = request_line.next().unwrap_or_default().to_string();
    let path = request_line.next().unwrap_or_default().to_string();
    asked
        .lock()
        .expect("not poisoned")
        .push(format!("{method} {path} {body}").trim().to_string());

    let shown = head
        .lines()
        .find_map(|line| line.strip_prefix("X-Gaia-Token: "))
        .unwrap_or_default();
    let (status, answer) = if shown == token {
        answer(kind, &method, &path, &body)
    } else {
        (401, json!({"error": "session token missing or wrong"}))
    };
    let answer = answer.to_string();
    let reply = format!(
        "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{answer}",
        answer.len()
    );
    let _ = caller.write_all(reply.as_bytes()).await;
}

fn answer(kind: Kind, method: &str, path: &str, body: &str) -> (u16, Value) {
    match (method, path) {
        ("GET", "/api/boot") if kind == Kind::Current => (
            200,
            json!({"version": "9.9.9", "home": "/home/me", "roots": ["/work", "/other"]}),
        ),
        ("GET", "/api/reach") if kind == Kind::Current => {
            (200, json!({"primals": true, "cursor": false}))
        }
        ("GET", "/api/models") => (
            200,
            json!({"via": "squirrel", "engine": "http://127.0.0.1:11436", "models": [
                {"name": "gaia:latest", "tools": true, "provider": "local-ai"},
                {"name": "tinyllama:latest", "tools": false},
            ]}),
        ),
        ("GET", "/api/fs/list?path=/work") => (
            200,
            json!({"path": "/work", "count": 2, "entries": [
                {"name": "a b&c.rs", "path": "/work/a b&c.rs", "is_dir": false, "is_link": false, "size": 4},
                {"name": "src", "path": "/work/src", "is_dir": true, "is_link": false, "size": 0},
            ]}),
        ),
        ("GET", "/api/fs/read?path=/work/a%20b%26c.rs") => (
            200,
            json!({"path": "/work/a b&c.rs", "content": "héllo\n", "size": 7, "mtime": 41}),
        ),
        ("GET", "/api/fs/read?path=/etc/shadow") => (
            403,
            json!({"error": "outside the folders this IDE may open"}),
        ),
        ("POST", "/api/fs/write") => {
            let sent: Value = serde_json::from_str(body).unwrap_or(Value::Null);
            if sent["if_mtime"] == json!(1) {
                (
                    409,
                    json!({"error": "the file changed on disk since it was opened", "mtime": 50}),
                )
            } else {
                (
                    200,
                    json!({"path": sent["path"], "written": 7, "created": false, "mtime": 42, "ok": true}),
                )
            }
        }
        _ => (404, Value::Null),
    }
}

/// The agent's side of a conversation: checks the token on the way in,
/// answers an `ask` with two pieces and an end, and hangs up on `bye`.
#[expect(
    clippy::result_large_err,
    reason = "the shape of the handshake's callback is tungstenite's"
)]
async fn agent(caller: TcpStream, token: &str, asked: &Asked) {
    let mut shown = String::new();
    let check = |request: &Request, response: Response| {
        shown = request
            .headers()
            .get("X-Gaia-Token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();
        Ok(response)
    };
    let Ok(mut socket) = tokio_tungstenite::accept_hdr_async(caller, check).await else {
        return;
    };
    if shown != token {
        let _ = socket.close(None).await;
        return;
    }
    while let Some(Ok(Message::Text(text))) = socket.next().await {
        asked
            .lock()
            .expect("not poisoned")
            .push(format!("WS {text}"));
        let heard: Value = serde_json::from_str(text.as_str()).unwrap_or(Value::Null);
        if heard["text"] == json!("bye") {
            let _ = socket.close(None).await;
            return;
        }
        if heard["t"] == json!("ask") {
            for message in [
                json!({"t": "delta", "text": "Hel"}),
                json!({"t": "delta", "text": "lo."}),
                json!({"t": "done", "how": "finished", "did": {}}),
            ] {
                let _ = socket.send(Message::text(message.to_string())).await;
            }
        }
    }
}

fn linked(place: &Place) -> (Link, Receiver<Event>, Arc<Mutex<usize>>) {
    let (sender, events) = channel();
    let woken = Arc::new(Mutex::new(0));
    let wake = {
        let woken = Arc::clone(&woken);
        Arc::new(move || *woken.lock().expect("not poisoned") += 1)
    };
    let link = Link::new(place.clone(), sender, wake).expect("the link starts");
    (link, events, woken)
}

fn next(events: &Receiver<Event>) -> Event {
    events
        .recv_timeout(Duration::from_secs(10))
        .expect("an event comes within ten seconds")
}

#[test]
fn the_server_is_found_with_its_version_and_its_folders() {
    let server = stand_in(Kind::Current);
    let (link, events, woken) = linked(&server.place);
    link.connect();
    let Event::Connected { version, roots } = next(&events) else {
        panic!("the server is there");
    };
    assert_eq!(version, "9.9.9");
    assert_eq!(roots, ["/work", "/other"]);
    assert_eq!(server.asked(), ["GET /api/boot"]);

    // The window is woken for every event, or it would sit on the news.
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(*woken.lock().expect("not poisoned"), 1);
}

#[test]
fn an_older_server_is_named_as_older() {
    let server = stand_in(Kind::Older);
    let (link, events, _) = linked(&server.place);
    link.connect();
    let Event::Offline(why) = next(&events) else {
        panic!("an older server cannot serve this window");
    };
    assert!(
        why.starts_with("the IDE's server is older than this window"),
        "{why}"
    );
}

#[test]
fn a_server_that_left_no_token_is_said_with_the_place_looked_in() {
    let server = stand_in(Kind::Current);
    std::fs::remove_file(&server.place.token_file).expect("the token file was there");
    let (link, events, _) = linked(&server.place);
    link.connect();
    let Event::Offline(why) = next(&events) else {
        panic!("without a token nothing can be asked");
    };
    assert!(why.contains("has left no session token at"), "{why}");
    assert!(why.contains("gaia-ide.token"), "{why}");
    assert_eq!(
        server.asked(),
        Vec::<String>::new(),
        "nothing was asked without a token"
    );
}

#[test]
fn a_wrong_token_is_refused_and_said() {
    let server = stand_in(Kind::Current);
    // A token the server does not have.
    let stale = Place {
        token_file: server.place.token_file.with_file_name("stale.token"),
        ..server.place.clone()
    };
    std::fs::write(&stale.token_file, "old-token").expect("written");
    let (link, events, _) = linked(&stale);
    link.connect();
    let Event::Offline(why) = next(&events) else {
        panic!("a wrong token is refused");
    };
    assert_eq!(why, "session token missing or wrong");
}

#[test]
fn the_token_is_read_anew_so_a_restarted_server_is_still_reached() {
    let server = stand_in(Kind::Current);
    let (link, events, _) = linked(&server.place);
    link.connect();
    assert!(matches!(next(&events), Event::Connected { .. }));

    // The server starts again: a new token, in the same file.
    server.set_token("second-token");
    link.models();
    let Event::Models(Ok((models, via))) = next(&events) else {
        panic!("the models come with the new token");
    };
    assert_eq!(via, "squirrel");
    assert_eq!(
        models,
        [
            Model {
                name: "gaia:latest".to_string(),
                tools: true
            },
            Model {
                name: "tinyllama:latest".to_string(),
                tools: false
            },
        ]
    );
}

#[test]
fn a_folder_is_listed_and_a_file_with_an_awkward_name_is_read() {
    let server = stand_in(Kind::Current);
    let (link, events, _) = linked(&server.place);

    link.list("/work".to_string());
    let Event::Listed(path, Ok(entries)) = next(&events) else {
        panic!("the folder is listed");
    };
    assert_eq!(path, "/work");
    assert_eq!(
        entries,
        [
            Entry {
                name: "a b&c.rs".to_string(),
                path: "/work/a b&c.rs".to_string(),
                is_dir: false
            },
            Entry {
                name: "src".to_string(),
                path: "/work/src".to_string(),
                is_dir: true
            },
        ]
    );

    link.open("/work/a b&c.rs".to_string());
    let Event::Opened(path, Ok((text, mtime))) = next(&events) else {
        panic!("the file is read");
    };
    assert_eq!(
        (path.as_str(), text.as_str(), mtime),
        ("/work/a b&c.rs", "héllo\n", Some(41))
    );
    assert_eq!(
        server.asked(),
        [
            "GET /api/fs/list?path=/work",
            "GET /api/fs/read?path=/work/a%20b%26c.rs"
        ]
    );
}

#[test]
fn a_file_the_server_will_not_read_says_why_in_the_servers_words() {
    let server = stand_in(Kind::Current);
    let (link, events, _) = linked(&server.place);
    link.open("/etc/shadow".to_string());
    let Event::Opened(_, Err(why)) = next(&events) else {
        panic!("the server refuses");
    };
    assert_eq!(why, "outside the folders this IDE may open");
}

#[test]
fn a_save_carries_the_time_last_seen_and_brings_back_the_new_one() {
    let server = stand_in(Kind::Current);
    let (link, events, _) = linked(&server.place);
    link.save("/work/a.rs".to_string(), "héllo\n".to_string(), Some(41));
    let Event::Saved(path, Ok((text, mtime))) = next(&events) else {
        panic!("the file is written");
    };
    assert_eq!(
        (path.as_str(), text.as_str(), mtime),
        ("/work/a.rs", "héllo\n", Some(42))
    );
    let asked = server.asked();
    let sent: Value = serde_json::from_str(asked[0].trim_start_matches("POST /api/fs/write "))
        .expect("the body is JSON");
    assert_eq!(
        sent,
        json!({"path": "/work/a.rs", "content": "héllo\n", "if_mtime": 41})
    );
}

#[test]
fn a_new_file_is_saved_without_a_time() {
    let server = stand_in(Kind::Current);
    let (link, events, _) = linked(&server.place);
    link.save("/work/new.rs".to_string(), "x".to_string(), None);
    assert!(matches!(next(&events), Event::Saved(_, Ok(_))));
    let asked = server.asked();
    let sent: Value = serde_json::from_str(asked[0].trim_start_matches("POST /api/fs/write "))
        .expect("the body is JSON");
    assert_eq!(sent, json!({"path": "/work/new.rs", "content": "x"}));
}

#[test]
fn a_save_over_a_file_that_changed_is_refused_in_the_servers_words() {
    let server = stand_in(Kind::Current);
    let (link, events, _) = linked(&server.place);
    link.save("/work/a.rs".to_string(), "mine".to_string(), Some(1));
    let Event::Saved(_, Err(not_saved)) = next(&events) else {
        panic!("the server refuses to write over a changed file");
    };
    assert_eq!(
        not_saved,
        NotSaved {
            why: "the file changed on disk since it was opened".to_string(),
            changed_on_disk: true,
        }
    );
}

#[test]
fn a_save_refused_for_another_reason_is_not_called_a_change_on_disk() {
    let server = stand_in(Kind::Current);
    std::fs::remove_file(&server.place.token_file).expect("the token file was there");
    let (link, events, _) = linked(&server.place);
    link.save("/work/a.rs".to_string(), "mine".to_string(), Some(41));
    let Event::Saved(_, Err(not_saved)) = next(&events) else {
        panic!("nothing can be written without a token");
    };
    assert!(!not_saved.changed_on_disk);
    assert!(
        not_saved.why.contains("has left no session token"),
        "{}",
        not_saved.why
    );
}

#[test]
fn what_this_device_can_reach_is_asked_and_an_older_server_has_nothing() {
    let server = stand_in(Kind::Current);
    let (link, events, _) = linked(&server.place);
    link.reach();
    assert!(matches!(
        next(&events),
        Event::Reach {
            primals: true,
            cursor: false
        }
    ));

    let older = stand_in(Kind::Older);
    let (link, events, _) = linked(&older.place);
    link.reach();
    assert!(matches!(
        next(&events),
        Event::Reach {
            primals: false,
            cursor: false
        }
    ));
}

#[test]
fn a_conversation_is_carried_and_a_new_one_is_opened_when_the_old_one_ended() {
    let server = stand_in(Kind::Current);
    let (link, events, _) = linked(&server.place);

    link.agent(&json!({"t": "ask", "text": "hello"}));
    let mut heard = Vec::new();
    while heard.len() < 3 {
        match next(&events) {
            Event::Agent(message) => heard.push(message),
            other => panic!("only the agent speaks here: {other:?}"),
        }
    }
    assert_eq!(heard[0], json!({"t": "delta", "text": "Hel"}));
    assert_eq!(heard[2]["t"], json!("done"));

    // The same conversation carries the next message.
    link.agent(&json!({"t": "ask", "text": "bye"}));
    let Event::AgentClosed(why) = next(&events) else {
        panic!("the server hangs up on bye");
    };
    assert_eq!(why, "the server closed it");

    // The next message opens a new conversation by itself.
    link.agent(&json!({"t": "ask", "text": "again"}));
    assert!(matches!(next(&events), Event::Agent(_)));
    let asked = server.asked();
    assert_eq!(
        asked,
        [
            r#"WS {"t":"ask","text":"hello"}"#,
            r#"WS {"t":"ask","text":"bye"}"#,
            r#"WS {"t":"ask","text":"again"}"#,
        ]
    );
}

#[test]
fn a_conversation_shows_the_token_as_it_is_now_and_without_one_says_why() {
    let server = stand_in(Kind::Current);
    let (link, events, _) = linked(&server.place);
    server.set_token("second-token");
    // The link reads the new token, so this is let in...
    link.agent(&json!({"t": "ask", "text": "hello"}));
    assert!(matches!(next(&events), Event::Agent(_)));

    // ...and with no token file at all it is said plainly.
    let gone = Place {
        token_file: server.place.token_file.with_file_name("none.token"),
        ..server.place.clone()
    };
    let (link, events, _) = linked(&gone);
    link.agent(&json!({"t": "ask", "text": "hello"}));
    let Event::AgentClosed(why) = next(&events) else {
        panic!("nothing can be opened without a token");
    };
    assert!(why.contains("has left no session token"), "{why}");
}

#[test]
fn the_place_of_the_token_follows_the_port() {
    let usual = Place::on_this_device(DEFAULT_PORT);
    assert_eq!(usual.host, "127.0.0.1:3142");
    assert!(usual.token_file.ends_with("gaia-ide.token"));
    let other = Place::on_this_device(3143);
    assert_eq!(other.host, "127.0.0.1:3143");
    assert!(other.token_file.ends_with("gaia-ide-3143.token"));
}
