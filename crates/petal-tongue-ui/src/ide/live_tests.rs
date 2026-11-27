// SPDX-License-Identifier: AGPL-3.0-or-later
//! The window's whole path against a real `gaia-ide`, without a display.
//!
//! Not run with the other tests: it needs a real server, a real model and
//! a folder it may change. Run it by name, with
//!
//! - `GAIA_IDE_LIVE_PORT`: the port of a `gaia-ide` to use
//! - `GAIA_IDE_LIVE_SHOP`: a copy of the little shop with the planted bug,
//!   inside a folder that server may open
//! - `GAIA_IDE_LIVE_MODEL` (optional): the model to ask, when not the one
//!   the window chooses by itself
//! - `GAIA_IDE_PICTURES` (optional): where to keep pictures of the window
//!
//! It plays the user through the same state changes the window's clicks
//! make, and draws the window along the way.

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::state::{Item, Server, StepState};
use super::{IdeOptions, Session, picture};

const WIDTH: f32 = 1500.0;
const HEIGHT: f32 = 950.0;

/// Take in what comes back until `done` says so; fail after `within`.
fn until(session: &mut Session, what: &str, within: Duration, done: impl Fn(&Session) -> bool) {
    let started = Instant::now();
    loop {
        session.take_in();
        if done(session) {
            return;
        }
        assert!(
            started.elapsed() < within,
            "waited {within:?} for: {what}\nserver: {:?}\nchat: {:#?}",
            session.state.server,
            session.state.chat
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn shoot(session: &mut Session, name: &str) {
    let (shot, asked) = picture::of(&mut session.state, WIDTH, HEIGHT);
    session.carry_out(asked);
    picture::keep(&shot, name);
}

fn on_disk(path: &str) -> String {
    std::fs::read_to_string(path).expect("the file is on disk")
}

#[test]
#[ignore = "needs a running gaia-ide and a model; see the top of this file"]
fn the_window_against_a_real_server() {
    let port: u16 = std::env::var("GAIA_IDE_LIVE_PORT")
        .expect("GAIA_IDE_LIVE_PORT is set")
        .parse()
        .expect("a port");
    let shop = std::env::var("GAIA_IDE_LIVE_SHOP").expect("GAIA_IDE_LIVE_SHOP is set");
    let cart = format!("{shop}/src/cart.py");
    assert!(Path::new(&cart).is_file(), "{cart} is there");
    let short = Duration::from_secs(15);

    // ── the window opens ────────────────────────────────────────────────
    let options = IdeOptions {
        port: Some(port),
        workspace: Some(shop.clone()),
        file: None,
    };
    let mut session = Session::new(options, Arc::new(|| {})).expect("the session starts");
    until(
        &mut session,
        "the server, its models, the folder",
        short,
        |s| {
            matches!(s.state.server, Server::Ready(_))
                && !s.state.via.is_empty()
                && matches!(
                    s.state.dirs.get(&s.state.workspace),
                    Some(super::state::Dir::Listed(_))
                )
        },
    );
    println!(
        "server {:?} · via {} · {} models with tools · chosen {} · primals {} · cursor {}",
        session.state.server,
        session.state.via,
        session.state.models.len(),
        session.state.model,
        session.state.can_primals,
        session.state.can_cursor
    );
    assert!(
        !session.state.model.is_empty(),
        "a model that can use tools is there"
    );

    // ── a folder is opened in the tree, a file is picked ────────────────
    let src = format!("{shop}/src");
    let asked = session.state.want_listing(&src);
    session.carry_out(asked);
    until(&mut session, "src listed", short, |s| {
        matches!(s.state.dirs.get(&src), Some(super::state::Dir::Listed(_)))
    });
    let asked = session.state.want_file(&cart);
    session.carry_out(asked);
    until(&mut session, "cart.py opened", short, |s| {
        s.state.buffer().is_some()
    });
    assert_eq!(session.state.buffers[0].text, on_disk(&cart));
    shoot(&mut session, "live-1-opened");

    // ── typing and Ctrl+S ───────────────────────────────────────────────
    session.state.buffers[0]
        .text
        .push_str("\n# seen by the window\n");
    let asked = session.state.save_current();
    session.carry_out(asked);
    until(&mut session, "saved", short, |s| {
        !s.state.buffers[0].dirty()
    });
    assert!(
        on_disk(&cart).ends_with("\n# seen by the window\n"),
        "the save reached the disk"
    );
    println!("save: on disk, mtime {:?}", session.state.buffers[0].mtime);

    // ── something else changes the file; the window's save is refused ───
    std::thread::sleep(Duration::from_millis(1100));
    let theirs = format!("{}# written by something else\n", on_disk(&cart));
    std::fs::write(&cart, &theirs).expect("the file can be written");
    session.state.buffers[0]
        .text
        .push_str("# typed after that\n");
    let asked = session.state.save_current();
    session.carry_out(asked);
    until(&mut session, "the refusal", short, |s| {
        s.state.buffers[0].notice.is_some()
    });
    assert!(session.state.buffers[0].changed_on_disk);
    assert_eq!(
        on_disk(&cart),
        theirs,
        "nothing was written over the other change"
    );
    println!("refused: {:?}", session.state.buffers[0].notice);
    shoot(&mut session, "live-2-changed-on-disk");

    // One way out: read it again.
    let asked = session.state.read_again(0);
    session.carry_out(asked);
    until(&mut session, "read again", short, |s| {
        s.state.buffers[0].notice.is_none()
    });
    assert_eq!(session.state.buffers[0].text, theirs);

    // The other: refused once more, then written over.
    std::thread::sleep(Duration::from_millis(1100));
    std::fs::write(&cart, "gone\n").expect("the file can be written");
    session.state.buffers[0].text = theirs.replace("# written by something else\n", "");
    let mine = session.state.buffers[0].text.clone();
    let asked = session.state.save_current();
    session.carry_out(asked);
    until(&mut session, "the second refusal", short, |s| {
        s.state.buffers[0].changed_on_disk
    });
    assert_eq!(on_disk(&cart), "gone\n");
    let asked = session.state.write_over(0);
    session.carry_out(asked);
    until(&mut session, "written over", short, |s| {
        !s.state.buffers[0].dirty()
    });
    assert_eq!(
        on_disk(&cart),
        mine,
        "the user's text is on disk after they said so"
    );

    // ── a file outside the folders the server may open ──────────────────
    let asked = session.state.want_file("/etc/hostname");
    session.carry_out(asked);
    until(&mut session, "the refusal to read outside", short, |s| {
        matches!(s.state.chat.last(), Some(Item::Error(_)))
    });
    println!("outside: {:?}", session.state.chat.last());
    session.state.chat.clear();

    // ── Gaia is asked; the user applies what she proposes, allows the
    //    tests to run, and allows nothing else ──────────────────────────
    if let Ok(model) = std::env::var("GAIA_IDE_LIVE_MODEL") {
        assert!(
            session.state.models.iter().any(|m| m.name == model),
            "{model} is offered"
        );
        session.state.model = model;
    }
    println!("asking {}", session.state.model);
    session.state.input = "The cart test is failing. Find out why and fix it.".to_string();
    let asked = session.state.ask();
    session.carry_out(asked);
    let started = Instant::now();
    let mut mid = false;
    while session.state.running {
        session.take_in();
        let mut clicks = Vec::new();
        for (n, item) in session.state.chat.iter().enumerate() {
            match item {
                Item::Step(step) if step.state == StepState::Asking => {
                    let harmless = step.summary.starts_with("python3 tests/")
                        || step.summary.starts_with("python3 -m unittest");
                    clicks.push((Some((step.id, harmless)), n));
                }
                Item::Proposal(proposal) if proposal.applied.is_none() => clicks.push((None, n)),
                _ => {}
            }
        }
        for (answer, n) in clicks {
            if !mid {
                mid = true;
                shoot(&mut session, "live-3-in-the-middle");
            }
            let asked = match answer {
                Some((id, allow)) => session.state.answer(id, allow, false),
                None => session.state.apply_proposal(n),
            };
            session.carry_out(asked);
        }
        assert!(
            started.elapsed() < Duration::from_secs(300),
            "Gaia took over five minutes"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    until(
        &mut session,
        "anything still on its way",
        Duration::from_secs(2),
        |_| true,
    );
    shoot(&mut session, "live-4-after");

    println!("── the conversation, {:.0?} ──", started.elapsed());
    for item in &session.state.chat {
        match item {
            Item::User(text) => println!("USER   {text}"),
            Item::Gaia(text) => println!("GAIA   {}", text.trim().replace('\n', "\n       ")),
            Item::Step(step) => println!(
                "STEP   [{:?}] {} {}",
                step.state,
                step.name,
                step.summary.chars().take(90).collect::<String>()
            ),
            Item::Proposal(proposal) => {
                println!("CHANGE {} → {:?}", proposal.file, proposal.applied);
            }
            Item::Note(text) => println!("NOTE   {text}"),
            Item::Facts(text) => println!("FACTS  {text}"),
            Item::Error(text) => println!("ERROR  {text}"),
        }
    }

    // What must hold whatever the model did: the run ended; nothing Gaia
    // proposed is on disk, because the user has not saved.
    assert!(!session.state.running);
    assert_eq!(
        on_disk(&cart),
        mine,
        "a proposal alone never reaches the disk"
    );
    let fixed_in_editor = session
        .state
        .buffers
        .iter()
        .find(|b| b.path == cart)
        .is_some_and(|b| b.dirty() && b.text.contains("qty"));
    println!("the fix is in the editor, unsaved: {fixed_in_editor}");

    // The user saves; now it is on disk.
    if fixed_in_editor {
        let n = session
            .state
            .buffers
            .iter()
            .position(|b| b.path == cart)
            .expect("open");
        session.carry_out(vec![super::state::Action::Save(n)]);
        until(&mut session, "the fix saved", short, |s| {
            !s.state.buffers[n].dirty()
        });
        assert_eq!(on_disk(&cart), session.state.buffers[n].text);
        shoot(&mut session, "live-5-saved");
        println!("saved: the fix is on disk");
    }
}
