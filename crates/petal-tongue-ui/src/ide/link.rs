// SPDX-License-Identifier: AGPL-3.0-or-later
//! The window's line to the IDE's server.
//!
//! The window never waits on the server: it asks here, the asking is done
//! on other threads, and what comes back arrives as an [`Event`] with a
//! wake-up for the window to draw again.
//!
//! The session token is read from its file for every request. It is a new
//! token each time the server starts, so a window left open across a
//! restart of the server keeps working.

use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use super::http::{Door, Failure, TOKEN_HEADER, query};
use super::state::{Entry, Event, Model, NotSaved};

/// The port the IDE's server listens on unless told otherwise.
pub const DEFAULT_PORT: u16 = 3142;

/// Called after every event, so the window draws again.
pub type Wake = Arc<dyn Fn() + Send + Sync>;

/// Where the server is and where it leaves its token.
#[derive(Debug, Clone)]
pub struct Place {
    /// `127.0.0.1:3142`
    pub host: String,
    /// The file the server writes its session token to.
    pub token_file: PathBuf,
}

impl Place {
    /// The IDE's server on this device, on `port`.
    pub fn on_this_device(port: u16) -> Self {
        let name = if port == DEFAULT_PORT {
            "gaia-ide.token".to_string()
        } else {
            format!("gaia-ide-{port}.token")
        };
        Self {
            host: format!("127.0.0.1:{port}"),
            token_file: runtime_dir().join(name),
        }
    }

    /// The door as it is now: the token as the server last wrote it.
    fn door(&self) -> Result<Door, String> {
        let token = std::fs::read_to_string(&self.token_file)
            .map_err(|e| {
                format!(
                    "the IDE's server has left no session token at {} ({e}). Is `gaia-ide` running?",
                    self.token_file.display()
                )
            })?
            .trim()
            .to_string();
        if token.is_empty() {
            return Err(format!(
                "the session token at {} is empty",
                self.token_file.display()
            ));
        }
        Ok(Door {
            host: self.host.clone(),
            token,
        })
    }
}

/// The folder the server keeps its token in: the same rule the server uses.
fn runtime_dir() -> PathBuf {
    let base = std::env::var("XDG_RUNTIME_DIR")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map_or_else(std::env::temp_dir, PathBuf::from);
    let biomeos = base.join("biomeos");
    if biomeos.is_dir() { biomeos } else { base }
}

/// The line to the server.
pub struct Link {
    place: Place,
    runtime: tokio::runtime::Runtime,
    events: Sender<Event>,
    wake: Wake,
    /// The way into the agent's conversation, while one is open.
    agent: Arc<Mutex<Option<UnboundedSender<String>>>>,
}

impl Link {
    /// A line to the server at `place`. Events go to `events`; `wake` is
    /// called after each.
    pub fn new(place: Place, events: Sender<Event>, wake: Wake) -> Result<Self, String> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("gaia-ide-link")
            .enable_all()
            .build()
            .map_err(|e| format!("the window's background work could not start ({e})"))?;
        Ok(Self {
            place,
            runtime,
            events,
            wake,
            agent: Arc::new(Mutex::new(None)),
        })
    }

    /// Do `work` in the background and hand what it found to the window.
    fn ask<F>(&self, work: F)
    where
        F: Future<Output = Event> + Send + 'static,
    {
        let (events, wake) = (self.events.clone(), Arc::clone(&self.wake));
        self.runtime.spawn(async move {
            let _ = events.send(work.await);
            wake();
        });
    }

    /// Look for the server: its version and the folders it may open.
    pub fn connect(&self) {
        let place = self.place.clone();
        self.ask(async move {
            match boot(&place).await {
                Ok((version, roots)) => Event::Connected { version, roots },
                Err(why) => Event::Offline(why),
            }
        });
    }

    /// List a folder.
    pub fn list(&self, path: String) {
        let place = self.place.clone();
        self.ask(async move {
            let listed = async {
                let answer = place
                    .door()?
                    .get(&format!("/api/fs/list?path={}", query(&path)))
                    .await?;
                Ok(entries_of(&answer))
            }
            .await;
            Event::Listed(path, listed)
        });
    }

    /// Read a file.
    pub fn open(&self, path: String) {
        let place = self.place.clone();
        self.ask(async move {
            let read = async {
                let answer = place
                    .door()?
                    .get(&format!("/api/fs/read?path={}", query(&path)))
                    .await?;
                let text = answer["content"]
                    .as_str()
                    .ok_or("the server sent no text for it")?
                    .to_string();
                Ok((text, answer["mtime"].as_u64()))
            }
            .await;
            Event::Opened(path, read)
        });
    }

    /// Write a file. `mtime` is the time the window last saw on it; the
    /// server refuses the write when the file changed since. With `None`
    /// the file is written whatever is there.
    pub fn save(&self, path: String, text: String, mtime: Option<u64>) {
        let place = self.place.clone();
        self.ask(async move {
            let written = async {
                let mut body = json!({ "path": path, "content": text });
                if let Some(mtime) = mtime {
                    body["if_mtime"] = json!(mtime);
                }
                let door = place.door().map_err(|why| Failure { status: None, why })?;
                let answer = door.post("/api/fs/write", &body).await?;
                Ok(answer["mtime"].as_u64())
            }
            .await;
            Event::Saved(
                path,
                written
                    .map(|mtime| (text, mtime))
                    .map_err(|failure: Failure| NotSaved {
                        // 409: the file on disk is not the one that was opened.
                        changed_on_disk: failure.status == Some(409),
                        why: failure.why,
                    }),
            )
        });
    }

    /// Ask which models there are.
    pub fn models(&self) {
        let place = self.place.clone();
        self.ask(async move {
            let found = async {
                let answer = place.door()?.get("/api/models").await?;
                Ok(models_of(&answer))
            }
            .await;
            Event::Models(found)
        });
    }

    /// Ask what this device has for tools that reach past the workspace.
    /// An older server has no such thing to say: then there is none.
    pub fn reach(&self) {
        let place = self.place.clone();
        self.ask(async move {
            let here = match place.door() {
                Ok(door) => door.get("/api/reach").await.unwrap_or(Value::Null),
                Err(_) => Value::Null,
            };
            Event::Reach {
                primals: here["primals"].as_bool().unwrap_or(false),
                cursor: here["cursor"].as_bool().unwrap_or(false),
            }
        });
    }

    /// Send a message into the agent's conversation, opening the
    /// connection when there is none.
    pub fn agent(&self, message: &Value) {
        let line = message.to_string();
        let Ok(mut way_in) = self.agent.lock() else {
            return;
        };
        let line = match way_in.as_ref().map(|open| open.send(line)) {
            Some(Ok(())) => return,
            // The connection ended since: the line comes back unsent.
            Some(Err(unsent)) => unsent.0,
            None => message.to_string(),
        };
        let (sender, outgoing) = unbounded_channel();
        let _ = sender.send(line);
        *way_in = Some(sender);
        drop(way_in);

        let (place, events, wake) = (
            self.place.clone(),
            self.events.clone(),
            Arc::clone(&self.wake),
        );
        let way_in = Arc::clone(&self.agent);
        self.runtime.spawn(async move {
            let why = converse(&place, outgoing, &events, &wake).await;
            // Nothing more can be sent this way; the next message opens anew.
            // (Unless it already did: then the way in is a newer one.)
            if let Ok(mut way_in) = way_in.lock()
                && way_in.as_ref().is_some_and(UnboundedSender::is_closed)
            {
                *way_in = None;
            }
            let _ = events.send(Event::AgentClosed(why));
            wake();
        });
    }
}

/// What the window needs to start. An older server has no `/api/boot`;
/// say so, since nothing else it offers tells which folders may be opened.
async fn boot(place: &Place) -> Result<(String, Vec<String>), String> {
    let door = place.door()?;
    let answer = match door.get("/api/boot").await {
        Ok(answer) => answer,
        Err(failure) if failure.status == Some(404) => {
            return Err(
                "the IDE's server is older than this window and cannot tell it which folders \
                 to open. Restart `gaia-ide` with its newer build."
                    .to_string(),
            );
        }
        Err(failure) => return Err(failure.why),
    };
    let roots = answer["roots"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|root| root.as_str().map(str::to_string))
        .collect();
    let version = answer["version"].as_str().unwrap_or("?").to_string();
    Ok((version, roots))
}

/// Carry one conversation with the agent until it ends; why it ended.
async fn converse(
    place: &Place,
    mut outgoing: UnboundedReceiver<String>,
    events: &Sender<Event>,
    wake: &Wake,
) -> String {
    let door = match place.door() {
        Ok(door) => door,
        Err(why) => return why,
    };
    let mut request = match format!("ws://{}/api/agent", door.host).into_client_request() {
        Ok(request) => request,
        Err(e) => return format!("the agent's address is not valid ({e})"),
    };
    match door.token.parse() {
        Ok(token) => request.headers_mut().insert(TOKEN_HEADER, token),
        Err(_) => return "the session token cannot be sent as a header".to_string(),
    };
    let (mut socket, _) = match tokio_tungstenite::connect_async(request).await {
        Ok(connected) => connected,
        Err(e) => return format!("the agent could not be reached ({e})"),
    };

    loop {
        tokio::select! {
            line = outgoing.recv() => {
                let Some(line) = line else {
                    return "the window closed".to_string();
                };
                if let Err(e) = socket.send(Message::text(line)).await {
                    return format!("the message could not be sent ({e})");
                }
            }
            heard = socket.next() => match heard {
                Some(Ok(Message::Text(text))) => {
                    if let Ok(message) = serde_json::from_str::<Value>(text.as_str()) {
                        let _ = events.send(Event::Agent(message));
                        wake();
                    }
                }
                Some(Ok(Message::Close(_))) | None => return "the server closed it".to_string(),
                Some(Ok(_)) => {}
                Some(Err(e)) => return e.to_string(),
            },
        }
    }
}

/// The entries of a folder as the server lists them.
fn entries_of(answer: &Value) -> Vec<Entry> {
    answer["entries"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            Some(Entry {
                name: entry["name"].as_str()?.to_string(),
                path: entry["path"].as_str()?.to_string(),
                is_dir: entry["is_dir"].as_bool().unwrap_or(false),
            })
        })
        .collect()
}

/// The models as the server lists them, and the way they are reached.
fn models_of(answer: &Value) -> (Vec<Model>, String) {
    let models = answer["models"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|model| {
            Some(Model {
                name: model["name"].as_str()?.to_string(),
                tools: model["tools"].as_bool().unwrap_or(false),
            })
        })
        .collect();
    (
        models,
        answer["via"].as_str().unwrap_or("engine").to_string(),
    )
}

#[cfg(test)]
#[path = "link_tests.rs"]
mod tests;
