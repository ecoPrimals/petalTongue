// SPDX-License-Identifier: AGPL-3.0-or-later
//! What the Gaia IDE window knows, and how it changes.
//!
//! Nothing here draws or talks to the server. The view reads this and
//! returns [`Action`]s; the link turns actions into requests and answers
//! into [`Event`]s; [`State::apply`] takes the events in. So everything the
//! window does can be tested without a display or a server.

use std::collections::BTreeMap;

use serde_json::Value;

/// One name in a folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The name shown.
    pub name: String,
    /// The whole path.
    pub path: String,
    /// Whether it is a folder.
    pub is_dir: bool,
}

/// What is known of a folder's contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dir {
    /// Asked for, not answered yet.
    Loading,
    /// Its entries, folders first.
    Listed(Vec<Entry>),
    /// Why it could not be listed.
    Failed(String),
}

/// A file open in the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    /// The file's whole path.
    pub path: String,
    /// The text as it is in the editor.
    pub text: String,
    /// The text as it is on disk, as far as the window knows.
    pub saved: String,
    /// When the file on disk was last changed, as the server said. `None`
    /// for a file that is not on disk yet.
    pub mtime: Option<u64>,
    /// Something the user should know about this file (a save refused).
    pub notice: Option<String>,
    /// A save was refused because the file on disk is no longer the one
    /// that was opened. The user chooses: read it again, or write over it.
    pub changed_on_disk: bool,
}

impl Buffer {
    /// Whether the editor holds changes that are not on disk.
    pub fn dirty(&self) -> bool {
        self.text != self.saved
    }

    /// The file's name without its folders.
    pub fn name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }

    /// The language the highlighter should use, from the file's ending.
    /// The highlighter knows Rust, Python, TOML and C; other languages
    /// that write comments and strings the way C does borrow C's colours,
    /// and the rest are shown plain.
    pub fn language(&self) -> &'static str {
        match self.name().rsplit_once('.').map(|(_, ending)| ending) {
            Some("rs") => "rs",
            Some("py") => "py",
            Some("toml") => "toml",
            Some(
                "c" | "h" | "cpp" | "hpp" | "cc" | "cxx" | "js" | "mjs" | "ts" | "tsx" | "jsx"
                | "go" | "java" | "kt" | "swift" | "cs" | "json",
            ) => "cpp",
            _ => "",
        }
    }
}

/// A model Gaia can be asked with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    /// The model's name.
    pub name: String,
    /// Whether it can use tools.
    pub tools: bool,
}

/// Where a tool call stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepState {
    /// Being carried out.
    Running,
    /// Waiting for the user's yes or no.
    Asking,
    /// Finished; whether it went well.
    Done(bool),
}

/// One tool call Gaia made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// The server's number for the call.
    pub id: u64,
    /// The tool's name: `run_command`, `primal_call`, `ask_cursor`, …
    pub name: String,
    /// What the call does, in a line.
    pub summary: String,
    /// Whether it is asked every time ("always allow" is not offered).
    pub once: bool,
    /// A warning the server attached.
    pub warn: Option<String>,
    /// Where it stands.
    pub state: StepState,
    /// What came back.
    pub result: Option<String>,
}

/// A change Gaia proposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Replace `old`, which is in the file exactly once, with `fresh`.
    Replace {
        /// The lines there now.
        old: String,
        /// The lines to put in their place.
        fresh: String,
    },
    /// A new file with this content.
    Whole(String),
}

/// A proposed change and what became of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    /// The file to change.
    pub file: String,
    /// The change.
    pub change: Change,
    /// `Some(Ok)`: put into the editor. `Some(Err)`: why it could not be.
    pub applied: Option<Result<(), String>>,
}

/// One thing in the conversation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// What the user asked.
    User(String),
    /// What Gaia wrote.
    Gaia(String),
    /// A tool call.
    Step(Step),
    /// A proposed change.
    Proposal(Proposal),
    /// Something the IDE did that the user should know.
    Note(String),
    /// What a run really did, counted by the IDE and not by the model.
    Facts(String),
    /// Something that went wrong.
    Error(String),
}

/// Whether the IDE's server is there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Server {
    /// Being looked for.
    Connecting,
    /// Answering; its version.
    Ready(String),
    /// Not there, and why.
    Offline(String),
}

/// Why a file was not written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotSaved {
    /// Why, in the server's words when it gave any.
    pub why: String,
    /// The file on disk changed after it was opened here, so the server
    /// would not write over it unseen.
    pub changed_on_disk: bool,
}

/// What came back from the server.
#[derive(Debug, Clone)]
pub enum Event {
    /// The server answered: the folders it may open.
    Connected {
        /// The server's version.
        version: String,
        /// Folders the IDE may open.
        roots: Vec<String>,
    },
    /// The server is not there.
    Offline(String),
    /// A folder's entries, or why not.
    Listed(String, Result<Vec<Entry>, String>),
    /// A file's text and when it was last changed, or why not.
    Opened(String, Result<(String, Option<u64>), String>),
    /// A file was written: the text that was written and its new time; or
    /// why not.
    Saved(String, Result<(String, Option<u64>), NotSaved>),
    /// The models and the way they are reached, or why not.
    Models(Result<(Vec<Model>, String), String>),
    /// What this device has for tools that reach past the workspace.
    Reach {
        /// The primals on this device can be called.
        primals: bool,
        /// Cursor's cloud agent can be asked.
        cursor: bool,
    },
    /// A message of the agent's conversation.
    Agent(Value),
    /// The agent's connection ended.
    AgentClosed(String),
}

/// What the user (or an event) asks the link to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Look for the server again.
    Connect,
    /// List a folder.
    List(String),
    /// Read a file into the editor.
    Open(String),
    /// Write buffer `n` to disk, unless the file changed since it was read.
    Save(usize),
    /// Write buffer `n` to disk whatever is there now. Only ever asked by
    /// the user, after a save was refused because the file had changed.
    SaveOver(usize),
    /// Ask for the models.
    Models,
    /// Ask what this device has for tools that reach past the workspace.
    Reach,
    /// Send this to the agent.
    Agent(Value),
}

/// Everything the window knows.
#[derive(Debug)]
pub struct State {
    /// Whether the server is there.
    pub server: Server,
    /// Folders the IDE may open.
    pub roots: Vec<String>,
    /// The folder being worked in.
    pub workspace: String,
    /// The workspace as typed in the top bar, before it is opened.
    pub workspace_typed: String,
    /// What is known of each folder asked about.
    pub dirs: BTreeMap<String, Dir>,
    /// The files open in the editor.
    pub buffers: Vec<Buffer>,
    /// Which of them is shown.
    pub current: Option<usize>,
    /// A buffer with changes the user asked to close: the view asks first.
    pub closing: Option<usize>,
    /// Where the cursor is in the file shown: line and column, from 1.
    pub cursor: Option<(usize, usize)>,
    /// The models that can use tools.
    pub models: Vec<Model>,
    /// How the models are reached: `squirrel` or `engine`.
    pub via: String,
    /// The model chosen.
    pub model: String,
    /// Whether Gaia is told which file is open.
    pub share_file: bool,
    /// Whether this device can let Gaia call its primals.
    pub can_primals: bool,
    /// Whether this device can let Gaia ask Cursor's cloud agent.
    pub can_cursor: bool,
    /// Whether Gaia may call primals (she asks before each call).
    pub use_primals: bool,
    /// Whether Gaia may ask Cursor's cloud agent, which sends the question
    /// off this device (she asks before each question). Off at every start.
    pub use_cursor: bool,
    /// The conversation.
    pub chat: Vec<Item>,
    /// The message being typed.
    pub input: String,
    /// Whether Gaia is answering.
    pub running: bool,
    /// Whether the last thing in the chat is text Gaia is still writing.
    writing: bool,
    /// A proposal waiting for its file to be read into the editor.
    pending: Option<usize>,
    /// A file to open as soon as the server answers.
    open_first: Option<String>,
}

impl State {
    /// A window that has not heard from the server yet. `workspace`: the
    /// folder to work in, when the user named one. `open_first`: a file to
    /// open at the start.
    pub fn new(workspace: Option<String>, open_first: Option<String>) -> Self {
        let workspace = workspace.unwrap_or_default();
        Self {
            server: Server::Connecting,
            roots: Vec::new(),
            workspace_typed: workspace.clone(),
            workspace,
            dirs: BTreeMap::new(),
            buffers: Vec::new(),
            current: None,
            closing: None,
            cursor: None,
            models: Vec::new(),
            via: String::new(),
            model: String::new(),
            share_file: true,
            can_primals: false,
            can_cursor: false,
            use_primals: false,
            use_cursor: false,
            chat: Vec::new(),
            input: String::new(),
            running: false,
            writing: false,
            pending: None,
            open_first,
        }
    }

    /// The buffer shown.
    pub fn buffer(&self) -> Option<&Buffer> {
        self.current.and_then(|n| self.buffers.get(n))
    }

    /// Take in something that came back. Returns what to ask for next.
    pub fn apply(&mut self, event: Event) -> Vec<Action> {
        match event {
            Event::Connected { version, roots } => {
                self.server = Server::Ready(version);
                if self.workspace.is_empty() {
                    self.workspace = roots.first().cloned().unwrap_or_default();
                    self.workspace_typed.clone_from(&self.workspace);
                }
                self.roots = roots;
                let mut next = vec![Action::Models, Action::Reach];
                next.extend(self.open_workspace());
                if let Some(file) = self.open_first.take() {
                    next.push(Action::Open(file));
                }
                next
            }
            Event::Offline(why) => {
                self.server = Server::Offline(why);
                Vec::new()
            }
            Event::Listed(path, listed) => {
                let dir = match listed {
                    Ok(mut entries) => {
                        entries.sort_by(|a, b| {
                            b.is_dir
                                .cmp(&a.is_dir)
                                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                        });
                        Dir::Listed(entries)
                    }
                    Err(why) => Dir::Failed(why),
                };
                self.dirs.insert(path, dir);
                Vec::new()
            }
            Event::Opened(path, read) => {
                self.opened(&path, read);
                Vec::new()
            }
            Event::Saved(path, written) => {
                if let Some(buffer) = self.buffers.iter_mut().find(|b| b.path == path) {
                    match written {
                        Ok((text, mtime)) => {
                            buffer.saved = text;
                            buffer.mtime = mtime;
                            buffer.notice = None;
                            buffer.changed_on_disk = false;
                        }
                        Err(not_saved) => {
                            buffer.changed_on_disk = not_saved.changed_on_disk;
                            buffer.notice = Some(if not_saved.changed_on_disk {
                                format!(
                                    "Not saved: {} was changed on disk by something else \
                                     after you opened it. Nothing was written.",
                                    buffer.name()
                                )
                            } else {
                                format!("Not saved: {}", not_saved.why)
                            });
                        }
                    }
                }
                // A new file shows up in its folder.
                let parent = parent_of(&path);
                if self.dirs.contains_key(parent) {
                    return vec![Action::List(parent.to_string())];
                }
                Vec::new()
            }
            Event::Models(Ok((models, via))) => {
                self.models = models.into_iter().filter(|m| m.tools).collect();
                self.via = via;
                if !self.models.iter().any(|m| m.name == self.model) {
                    self.model = ["gaia:latest", "gaia-tissue-cortex:latest"]
                        .iter()
                        .find(|want| self.models.iter().any(|m| m.name == **want))
                        .map(ToString::to_string)
                        .or_else(|| self.models.first().map(|m| m.name.clone()))
                        .unwrap_or_default();
                }
                Vec::new()
            }
            Event::Models(Err(why)) => {
                self.models.clear();
                self.via.clear();
                self.chat
                    .push(Item::Error(format!("The models cannot be listed: {why}")));
                Vec::new()
            }
            Event::Reach { primals, cursor } => {
                self.can_primals = primals;
                self.can_cursor = cursor;
                self.use_primals &= primals;
                self.use_cursor &= cursor;
                Vec::new()
            }
            Event::Agent(message) => {
                self.agent(&message);
                Vec::new()
            }
            Event::AgentClosed(why) => {
                if self.running {
                    self.running = false;
                    self.writing = false;
                    self.chat.push(Item::Error(format!(
                        "The connection to the IDE's server ended before Gaia finished: {why}"
                    )));
                }
                Vec::new()
            }
        }
    }

    /// Make `workspace` the folder shown in the tree.
    pub fn open_workspace(&mut self) -> Vec<Action> {
        if self.workspace.is_empty() {
            return Vec::new();
        }
        self.dirs.clear();
        self.dirs.insert(self.workspace.clone(), Dir::Loading);
        vec![Action::List(self.workspace.clone())]
    }

    /// The user chose another folder to work in.
    pub fn set_workspace(&mut self, path: &str) -> Vec<Action> {
        let path = path.trim().trim_end_matches('/');
        if path.is_empty() || path == self.workspace {
            return Vec::new();
        }
        self.workspace = path.to_string();
        self.workspace_typed.clone_from(&self.workspace);
        self.open_workspace()
    }

    /// The user opened a folder in the tree.
    pub fn want_listing(&mut self, path: &str) -> Vec<Action> {
        if self.dirs.contains_key(path) {
            return Vec::new();
        }
        self.dirs.insert(path.to_string(), Dir::Loading);
        vec![Action::List(path.to_string())]
    }

    /// The user picked a file: show it, reading it when it is not open yet.
    pub fn want_file(&mut self, path: &str) -> Vec<Action> {
        if let Some(n) = self.buffers.iter().position(|b| b.path == path) {
            self.current = Some(n);
            return Vec::new();
        }
        vec![Action::Open(path.to_string())]
    }

    /// The user asked to close buffer `n`. With changes not saved, the
    /// view asks first and nothing is closed yet.
    pub fn want_close(&mut self, n: usize) {
        if self.buffers.get(n).is_some_and(Buffer::dirty) {
            self.closing = Some(n);
            self.current = Some(n);
        } else {
            self.close(n);
        }
    }

    /// Close buffer `n`. Changes not saved go with it.
    pub fn close(&mut self, n: usize) {
        self.closing = None;
        if n >= self.buffers.len() {
            return;
        }
        self.buffers.remove(n);
        self.current = match self.current {
            _ if self.buffers.is_empty() => None,
            Some(c) if c > n => Some(c - 1),
            Some(c) => Some(c.min(self.buffers.len() - 1)),
            None => None,
        };
    }

    /// After a refused save: drop what is in the editor and read the file
    /// as it is on disk now.
    pub fn read_again(&self, n: usize) -> Vec<Action> {
        self.buffers
            .get(n)
            .map_or_else(Vec::new, |buffer| vec![Action::Open(buffer.path.clone())])
    }

    /// After a refused save: write what is in the editor over what is on
    /// disk now.
    pub fn write_over(&self, n: usize) -> Vec<Action> {
        match self.buffers.get(n) {
            Some(buffer) if buffer.changed_on_disk => vec![Action::SaveOver(n)],
            _ => Vec::new(),
        }
    }

    /// Save the buffer shown, when it has changes.
    pub fn save_current(&self) -> Vec<Action> {
        match self.current {
            Some(n) if self.buffers.get(n).is_some_and(Buffer::dirty) => vec![Action::Save(n)],
            _ => Vec::new(),
        }
    }

    fn opened(&mut self, path: &str, read: Result<(String, Option<u64>), String>) {
        let waiting = self.pending.take();
        match read {
            Ok((text, mtime)) => {
                let fresh = Buffer {
                    path: path.to_string(),
                    saved: text.clone(),
                    text,
                    mtime,
                    notice: None,
                    changed_on_disk: false,
                };
                let n = if let Some(n) = self.buffers.iter().position(|b| b.path == path) {
                    self.buffers[n] = fresh;
                    n
                } else {
                    self.buffers.push(fresh);
                    self.buffers.len() - 1
                };
                self.current = Some(n);
                if let Some(proposal) = waiting {
                    self.apply_proposal(proposal);
                }
            }
            Err(why) => {
                let said = format!("Cannot open {path}: {why}");
                match waiting.and_then(|n| self.chat.get_mut(n)) {
                    Some(Item::Proposal(proposal)) => proposal.applied = Some(Err(said)),
                    _ => self.chat.push(Item::Error(said)),
                }
            }
        }
    }

    /// Send what is typed to Gaia.
    pub fn ask(&mut self) -> Vec<Action> {
        let text = self.input.trim().to_string();
        if text.is_empty() || self.running || self.model.is_empty() || self.workspace.is_empty() {
            return Vec::new();
        }
        self.input.clear();
        self.chat.push(Item::User(text.clone()));
        self.running = true;
        self.writing = false;
        let open = self
            .buffer()
            .filter(|_| self.share_file)
            .map(|b| b.path.clone());
        vec![Action::Agent(serde_json::json!({
            "t": "ask", "text": text, "model": self.model, "workspace": self.workspace,
            "open": open, "selection": null,
            "primals": self.can_primals && self.use_primals,
            "cursor": self.can_cursor && self.use_cursor,
        }))]
    }

    /// Stop Gaia.
    pub fn stop(&self) -> Vec<Action> {
        if self.running {
            vec![Action::Agent(serde_json::json!({ "t": "stop" }))]
        } else {
            Vec::new()
        }
    }

    /// Forget the conversation, here and on the server.
    pub fn clear_chat(&mut self) -> Vec<Action> {
        if self.running {
            return Vec::new();
        }
        let had_any = !self.chat.is_empty();
        self.chat.clear();
        self.pending = None;
        if had_any {
            vec![Action::Agent(serde_json::json!({ "t": "reset" }))]
        } else {
            Vec::new()
        }
    }

    /// Answer the question a tool call is waiting on.
    pub fn answer(&mut self, id: u64, allow: bool, always: bool) -> Vec<Action> {
        let Some(step) = self.chat.iter_mut().find_map(|item| match item {
            Item::Step(step) if step.id == id && step.state == StepState::Asking => Some(step),
            _ => None,
        }) else {
            return Vec::new();
        };
        step.state = StepState::Running;
        vec![Action::Agent(if allow {
            serde_json::json!({ "t": "allow", "id": id, "always": always && !step.once })
        } else {
            serde_json::json!({ "t": "deny", "id": id })
        })]
    }

    /// Take in one message of the agent's conversation.
    fn agent(&mut self, message: &Value) {
        let text = |key: &str| message[key].as_str().unwrap_or_default().to_string();
        match message["t"].as_str().unwrap_or_default() {
            "delta" => {
                let piece = text("text");
                match self.chat.last_mut() {
                    Some(Item::Gaia(so_far)) if self.writing => so_far.push_str(&piece),
                    _ => {
                        self.chat.push(Item::Gaia(piece));
                        self.writing = true;
                    }
                }
            }
            "turn" => self.writing = false,
            "tool" => {
                // Some models write a tool call out as text and the server
                // recovers it. The step says what was done; the same thing
                // once more as raw JSON only gets in the way.
                if self.writing
                    && matches!(self.chat.last(), Some(Item::Gaia(text)) if is_a_call_written_out(text))
                {
                    self.chat.pop();
                }
                self.writing = false;
                self.chat.push(Item::Step(Step {
                    id: message["id"].as_u64().unwrap_or(0),
                    name: text("name"),
                    summary: text("summary"),
                    once: message["once"].as_bool().unwrap_or(false),
                    warn: message["warn"].as_str().map(str::to_string),
                    state: if message["ask"].as_bool().unwrap_or(false) {
                        StepState::Asking
                    } else {
                        StepState::Running
                    },
                    result: None,
                }));
            }
            "result" => {
                let id = message["id"].as_u64().unwrap_or(0);
                let ok = message["ok"].as_bool().unwrap_or(false);
                if let Some(step) = self.chat.iter_mut().rev().find_map(|item| match item {
                    Item::Step(step) if step.id == id => Some(step),
                    _ => None,
                }) {
                    step.state = StepState::Done(ok);
                    step.result = Some(text("text"));
                }
                if let Some(proposal) = proposal_of(&message["proposal"]) {
                    self.chat.push(Item::Proposal(proposal));
                }
            }
            "note" => self.chat.push(Item::Note(text("text"))),
            "error" => self.chat.push(Item::Error(text("error"))),
            "done" => {
                let wrote_nothing = !self
                    .chat
                    .iter()
                    .rev()
                    .take_while(|i| !matches!(i, Item::User(_)))
                    .any(|i| {
                        matches!(i, Item::Gaia(t) if !t.trim().is_empty())
                            || matches!(i, Item::Error(_))
                    });
                self.running = false;
                self.writing = false;
                match message["how"].as_str() {
                    Some("stopped") => self.chat.push(Item::Note("Stopped.".to_string())),
                    Some("finished") if wrote_nothing => self.chat.push(Item::Note(
                        "Gaia finished without saying anything. Ask again, or try another model."
                            .to_string(),
                    )),
                    _ => {}
                }
                if let Some(facts) = facts_of(&message["did"]) {
                    self.chat.push(Item::Facts(facts));
                }
                // A question never answered cannot be answered now.
                for item in &mut self.chat {
                    if let Item::Step(step) = item
                        && !matches!(step.state, StepState::Done(_))
                    {
                        step.state = StepState::Done(false);
                    }
                }
            }
            _ => {}
        }
    }

    /// Put proposal `n` of the chat into the editor. Nothing is written to
    /// disk: the user saves, or does not.
    pub fn apply_proposal(&mut self, n: usize) -> Vec<Action> {
        let Some(Item::Proposal(proposal)) = self.chat.get(n) else {
            return Vec::new();
        };
        if proposal.applied == Some(Ok(())) {
            return Vec::new();
        }
        let (file, change) = (proposal.file.clone(), proposal.change.clone());
        let open = self.buffers.iter().position(|b| b.path == file);

        let outcome = match (change, open) {
            (Change::Whole(content), Some(at)) => {
                self.buffers[at].text = content;
                self.current = Some(at);
                Ok(())
            }
            (Change::Whole(content), None) => {
                self.buffers.push(Buffer {
                    path: file,
                    text: content,
                    saved: String::new(),
                    mtime: None,
                    notice: Some("A new file: it is not on disk until you save it.".to_string()),
                    changed_on_disk: false,
                });
                self.current = Some(self.buffers.len() - 1);
                Ok(())
            }
            (Change::Replace { .. }, None) => {
                // Read the file first; `opened` comes back here.
                self.pending = Some(n);
                return vec![Action::Open(file)];
            }
            (Change::Replace { old, fresh }, Some(at)) => {
                let buffer = &mut self.buffers[at];
                self.current = Some(at);
                match buffer.text.matches(old.as_str()).count() {
                    1 => {
                        buffer.text = buffer.text.replacen(old.as_str(), &fresh, 1);
                        Ok(())
                    }
                    0 => Err(
                        "The lines to replace are not in the file as it is in the editor now."
                            .to_string(),
                    ),
                    _ => Err(
                        "The lines to replace are in the file more than once; it is not clear which were meant."
                            .to_string(),
                    ),
                }
            }
        };
        if let Some(Item::Proposal(proposal)) = self.chat.get_mut(n) {
            proposal.applied = Some(outcome);
        }
        Vec::new()
    }
}

/// Whether this text is nothing but a tool call written out.
///
/// That is `{"name": .., "arguments": {..}}`, with or without a code fence
/// around it. Models do not always write it as valid JSON, so it is
/// recognised by its shape.
pub fn is_a_call_written_out(text: &str) -> bool {
    let mut inner = text.trim();
    if let Some(rest) = inner.strip_prefix("```") {
        inner = rest
            .trim_start_matches(|c: char| c.is_ascii_alphanumeric())
            .trim_end()
            .trim_end_matches("```")
            .trim();
    }
    inner.starts_with('{')
        && inner.ends_with('}')
        && inner.contains("\"name\"")
        && (inner.contains("\"arguments\"") || inner.contains("\"parameters\""))
}

/// The folder a path is in.
pub fn parent_of(path: &str) -> &str {
    match path.rfind('/') {
        Some(0) => "/",
        Some(at) => &path[..at],
        None => "",
    }
}

/// The IDE's own count of what a run did, in a line. A small model will
/// sometimes say "the tests pass" when no test ran; this is what happened.
fn facts_of(did: &Value) -> Option<String> {
    let count = |key: &str| did[key].as_u64().unwrap_or(0);
    let some = |n: u64, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
    let short = |text: &str, most: usize| {
        if text.chars().count() > most {
            format!("{}…", text.chars().take(most - 1).collect::<String>())
        } else {
            text.to_string()
        }
    };
    let mut parts = Vec::new();
    if count("proposed") > 0 {
        parts.push(format!(
            "{} proposed — none is in the file until you apply it and save",
            some(count("proposed"), "change", "changes")
        ));
    }
    for command in did["commands"].as_array().into_iter().flatten() {
        let how = match command["exit"].as_i64() {
            Some(0) => "exit code 0 (succeeded)".to_string(),
            Some(code) => format!("exit code {code} (failed)"),
            None => "ended by a signal".to_string(),
        };
        let line = short(command["command"].as_str().unwrap_or_default(), 60);
        parts.push(format!("ran {line}, {how}"));
    }
    for reached in did["reached"].as_array().into_iter().flatten() {
        parts.push(short(reached.as_str().unwrap_or_default(), 70));
    }
    if count("refused") > 0 {
        parts.push(format!(
            "{} you did not allow",
            some(count("refused"), "step", "steps")
        ));
    }
    if count("failed") > 0 {
        parts.push(format!("{} failed", some(count("failed"), "step", "steps")));
    }
    if parts.is_empty() {
        None
    } else {
        Some(format!("What actually happened: {}.", parts.join(" · ")))
    }
}

/// A proposal as the server sends it: `{file, old, fresh}` or `{file, whole}`.
fn proposal_of(sent: &Value) -> Option<Proposal> {
    let file = sent.get("file")?.as_str()?.to_string();
    let change = match (sent.get("whole"), sent.get("old"), sent.get("fresh")) {
        (Some(whole), _, _) => Change::Whole(whole.as_str()?.to_string()),
        (None, Some(old), Some(fresh)) => Change::Replace {
            old: old.as_str()?.to_string(),
            fresh: fresh.as_str()?.to_string(),
        },
        _ => return None,
    };
    Some(Proposal {
        file,
        change,
        applied: None,
    })
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
