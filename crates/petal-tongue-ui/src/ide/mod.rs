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

/// The window.
pub struct IdeApp {
    session: Session,
}

impl IdeApp {
    /// A window that starts by looking for the server.
    pub fn new(options: IdeOptions, ctx: &egui::Context) -> Result<Self, String> {
        view::style(ctx);
        let wake = {
            let ctx = ctx.clone();
            Arc::new(move || ctx.request_repaint())
        };
        Ok(Self {
            session: Session::new(options, wake)?,
        })
    }
}

impl eframe::App for IdeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.session.take_in();
        let asked = view::draw(ctx, &mut self.session.state);
        self.session.carry_out(asked);
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
        Box::new(move |cc| Ok(Box::new(IdeApp::new(options, &cc.egui_ctx)?))),
    )
    .map_err(|e| e.to_string())
}
