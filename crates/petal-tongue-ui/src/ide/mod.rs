// SPDX-License-Identifier: AGPL-3.0-or-later
//! The Gaia IDE in a window of its own.
//!
//! `gaia-ide` (in biomeOS) is the part that reads files, runs tools and
//! reaches Gaia; until now its only face was a page in a browser. This is
//! a second face: a native window, drawn here, that talks to the same
//! server on this device with the same session token. The window frame,
//! its buttons and its place on the screen are the desktop's own.
//!
//! - [`state`]: what the window knows and how it changes (no drawing, no
//!   network; this is where the tests are)
//! - [`view`]: the state on the screen
//! - [`link`]: the line to the server, on background threads
//! - [`http`]: the few plain HTTP requests the link makes

pub mod http;
pub mod link;
pub mod state;
pub mod view;

#[cfg(test)]
mod live_tests;
#[cfg(test)]
mod picture;

use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};

use link::{Link, Place};
use state::{Action, Event, State};

/// The name the desktop knows the window by. It matches the launcher
/// (`gaia-ide.desktop`), so the desktop shows the launcher's name and icon
/// for it.
pub const APP_ID: &str = "gaia-ide";

/// The window's title.
pub const TITLE: &str = "Gaia IDE";

/// How the window was asked for.
#[derive(Debug, Clone, Default)]
pub struct IdeOptions {
    /// The port the IDE's server listens on; `None` for the usual one.
    pub port: Option<u16>,
    /// The folder to work in; `None` for the first the server may open.
    pub workspace: Option<String>,
    /// A file to open at the start.
    pub file: Option<String>,
}

/// One sitting at the IDE: what is known, and the line to the server.
/// The window draws it; it can also be driven with no window at all.
pub struct Session {
    /// What is known.
    pub state: State,
    link: Link,
    events: Receiver<Event>,
}

impl Session {
    /// A sitting that starts by looking for the server. `wake` is called
    /// whenever something came back.
    pub fn new(options: IdeOptions, wake: link::Wake) -> Result<Self, String> {
        let (sender, events) = channel();
        let place = Place::on_this_device(options.port.unwrap_or(link::DEFAULT_PORT));
        let link = Link::new(place, sender, wake)?;
        link.connect();
        Ok(Self {
            state: State::new(options.workspace, options.file),
            link,
            events,
        })
    }

    /// Take in everything that came back since the last time. Returns
    /// whether anything had.
    pub fn take_in(&mut self) -> bool {
        let mut any = false;
        while let Ok(event) = self.events.try_recv() {
            any = true;
            let next = self.state.apply(event);
            self.carry_out(next);
        }
        any
    }

    /// Hand each action to the link.
    pub fn carry_out(&mut self, actions: Vec<Action>) {
        for action in actions {
            match action {
                Action::Connect => {
                    self.state.server = state::Server::Connecting;
                    self.link.connect();
                }
                Action::List(path) => self.link.list(path),
                Action::Open(path) => self.link.open(path),
                Action::Save(n) => {
                    if let Some(buffer) = self.state.buffers.get(n) {
                        self.link
                            .save(buffer.path.clone(), buffer.text.clone(), buffer.mtime);
                    }
                }
                Action::SaveOver(n) => {
                    if let Some(buffer) = self.state.buffers.get(n) {
                        self.link
                            .save(buffer.path.clone(), buffer.text.clone(), None);
                    }
                }
                Action::Models => self.link.models(),
                Action::Reach => self.link.reach(),
                Action::Agent(message) => self.link.agent(&message),
            }
        }
    }
}

/// Under these names the window keeps, from one opening to the next, the
/// folder last worked in and the model last chosen.
const KEPT_WORKSPACE: &str = "gaia-ide.workspace";
const KEPT_MODEL: &str = "gaia-ide.model";

/// Fill in from what was kept last time what this opening did not say.
///
/// A folder or a file named on the command line wins over the kept folder.
/// Returns the options to open with and the model to start on, if any.
/// The browser page keeps the same two things, so the window is no step
/// back from it.
pub fn remembered(
    mut options: IdeOptions,
    kept: Option<&dyn eframe::Storage>,
) -> (IdeOptions, Option<String>) {
    let kept_under = |key: &str| {
        kept.and_then(|storage| storage.get_string(key))
            .filter(|value| !value.trim().is_empty())
    };
    if options.workspace.is_none() && options.file.is_none() {
        options.workspace = kept_under(KEPT_WORKSPACE);
    }
    (options, kept_under(KEPT_MODEL))
}

/// Keep the folder and the model for the next opening.
pub fn remember(state: &State, keep: &mut dyn eframe::Storage) {
    keep.set_string(KEPT_WORKSPACE, state.workspace.clone());
    keep.set_string(KEPT_MODEL, state.model.clone());
}

/// The window.
pub struct IdeApp {
    session: Session,
}

impl IdeApp {
    /// A window that starts by looking for the server, in the folder and
    /// with the model it had when it was last closed.
    pub fn new(options: IdeOptions, cc: &eframe::CreationContext<'_>) -> Result<Self, String> {
        view::style(&cc.egui_ctx);
        let wake = {
            let ctx = cc.egui_ctx.clone();
            Arc::new(move || ctx.request_repaint())
        };
        let (options, model) = remembered(options, cc.storage);
        let mut session = Session::new(options, wake)?;
        // It stays chosen only if the server still offers it (`State::apply`).
        if let Some(model) = model {
            session.state.model = model;
        }
        Ok(Self { session })
    }
}

impl eframe::App for IdeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.session.take_in();
        let asked = view::draw(ctx, &mut self.session.state);
        self.session.carry_out(asked);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        remember(&self.session.state, storage);
    }
}

/// The window as the desktop is asked for it: its title, its size, and the
/// name that ties it to the launcher. The frame is left to the desktop.
pub fn viewport() -> egui::ViewportBuilder {
    egui::ViewportBuilder::default()
        .with_title(TITLE)
        .with_app_id(APP_ID)
        .with_inner_size([1500.0, 950.0])
        .with_min_inner_size([900.0, 560.0])
}

/// Open the window and stay until it is closed. Must be called on the main
/// thread. `native` are the options for the event loop, with [`viewport`]
/// as their viewport.
pub fn run(options: IdeOptions, native: eframe::NativeOptions) -> Result<(), String> {
    eframe::run_native(
        APP_ID,
        native,
        Box::new(move |cc| Ok(Box::new(IdeApp::new(options, cc)?))),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// What eframe keeps on disk between openings, held in memory.
    #[derive(Default)]
    struct Kept(HashMap<String, String>);

    impl eframe::Storage for Kept {
        fn get_string(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
        fn set_string(&mut self, key: &str, value: String) {
            self.0.insert(key.to_string(), value);
        }
        fn flush(&mut self) {}
    }

    fn closed_in(workspace: &str, model: &str) -> Kept {
        let mut state = State::new(Some(workspace.to_string()), None);
        state.model = model.to_string();
        let mut kept = Kept::default();
        remember(&state, &mut kept);
        kept
    }

    #[test]
    fn the_window_opens_where_it_was_closed() {
        let kept = closed_in("/work/shop", "qwen2.5-coder:14b");
        let (options, model) = remembered(IdeOptions::default(), Some(&kept));
        assert_eq!(options.workspace.as_deref(), Some("/work/shop"));
        assert_eq!(model.as_deref(), Some("qwen2.5-coder:14b"));
    }

    #[test]
    fn a_folder_or_a_file_named_now_wins_over_the_one_kept() {
        let kept = closed_in("/work/shop", "gaia:latest");
        let named = IdeOptions {
            workspace: Some("/work/other".to_string()),
            ..Default::default()
        };
        let (options, _) = remembered(named, Some(&kept));
        assert_eq!(options.workspace.as_deref(), Some("/work/other"));

        let named = IdeOptions {
            file: Some("/elsewhere/a.rs".to_string()),
            ..Default::default()
        };
        let (options, _) = remembered(named, Some(&kept));
        assert_eq!(
            options.workspace, None,
            "the file's own place, not the kept folder"
        );
    }

    #[test]
    fn a_first_opening_has_nothing_kept() {
        let (options, model) = remembered(IdeOptions::default(), None);
        assert_eq!((options.workspace, model), (None, None));

        // Closed before the server ever answered: nothing worth keeping.
        let kept = closed_in("", "");
        let (options, model) = remembered(IdeOptions::default(), Some(&kept));
        assert_eq!((options.workspace, model), (None, None));
    }

    #[test]
    fn a_kept_model_the_server_no_longer_offers_gives_way() {
        let mut state = State::new(None, None);
        state.model = "gone:1b".to_string();
        state.apply(Event::Models(Ok((
            vec![state::Model {
                name: "gaia:latest".to_string(),
                tools: true,
            }],
            "squirrel".to_string(),
        ))));
        assert_eq!(state.model, "gaia:latest");
    }
}
