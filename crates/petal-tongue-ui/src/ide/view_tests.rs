// SPDX-License-Identifier: AGPL-3.0-or-later
//! The window drawn without a display.
//!
//! Each test builds a state, draws it for a few frames into a picture, and
//! checks what can be checked without eyes. With `GAIA_IDE_PICTURES=<folder>`
//! the pictures are kept there to be looked at.

use serde_json::json;

use super::super::picture;
use super::super::state::{Action, Entry, Event, Model, NotSaved, State};
use super::*;

const WIDTH: f32 = 1500.0;
const HEIGHT: f32 = 950.0;

const CART: &str = r#"from dataclasses import dataclass


@dataclass
class Line:
    name: str
    price: float
    count: int = 1


class Cart:
    """What someone means to buy."""

    def __init__(self):
        self.lines = []

    def add(self, name, price, count=1):
        self.lines.append(Line(name, price, count))

    def total(self):
        # every line counts as many times as it was asked for
        return sum(line.price for line in self.lines)
"#;

fn connected() -> State {
    let mut state = State::new(None, None);
    state.apply(Event::Connected {
        version: "4.57.0".to_string(),
        roots: vec![
            "/home/me/shop".to_string(),
            "/home/me/Development".to_string(),
        ],
    });
    state.apply(Event::Models(Ok((
        vec![
            Model {
                name: "gaia:latest".to_string(),
                tools: true,
            },
            Model {
                name: "qwen2.5-coder:14b".to_string(),
                tools: true,
            },
        ],
        "squirrel".to_string(),
    ))));
    state.apply(Event::Reach {
        primals: true,
        cursor: true,
    });
    let at = |name: &str, is_dir| Entry {
        name: name.to_string(),
        path: format!("/home/me/shop/{name}"),
        is_dir,
    };
    state.apply(Event::Listed(
        "/home/me/shop".to_string(),
        Ok(vec![
            at("src", true),
            at("tests", true),
            at("README.md", false),
            at("pyproject.toml", false),
        ]),
    ));
    state
}

fn working() -> State {
    let mut state = connected();
    state.apply(Event::Opened(
        "/home/me/shop/src/cart.py".to_string(),
        Ok((CART.to_string(), Some(100))),
    ));
    state.input = "the total is wrong when I buy two of something. find it and fix it".to_string();
    state.ask();
    for message in [
        json!({"t": "delta", "text": "I will read the cart first.\n"}),
        json!({"t": "tool", "id": 1, "name": "read_file", "summary": "read src/cart.py", "ask": false}),
        json!({"t": "result", "id": 1, "ok": true, "text": CART}),
        json!({"t": "turn"}),
        json!({"t": "delta", "text": "`total` adds each price once and forgets `count`:\n\n```python\nreturn sum(line.price for line in self.lines)\n```\n\nIt should multiply by the count."}),
        json!({"t": "tool", "id": 2, "name": "propose_edit", "summary": "change src/cart.py", "ask": false}),
        json!({"t": "result", "id": 2, "ok": true, "text": "proposed", "proposal": {
            "file": "/home/me/shop/src/cart.py",
            "old": "        return sum(line.price for line in self.lines)\n",
            "fresh": "        return sum(line.price * line.count for line in self.lines)\n",
        }}),
        json!({"t": "turn"}),
        json!({"t": "tool", "id": 3, "name": "run_command", "summary": "python -m pytest -q", "ask": true}),
    ] {
        state.apply(Event::Agent(message));
    }
    state
}

fn drawn(state: &mut State, name: &str) -> Vec<Action> {
    let (picture, asked) = picture::of(state, WIDTH, HEIGHT);
    assert_eq!((picture.width, picture.height), (1500, 950));
    assert!(
        picture.colours() > 8,
        "{name}: the picture is blank ({} colours)",
        picture.colours()
    );
    picture::keep(&picture, name);
    asked
}

#[test]
fn a_window_still_looking_for_its_server_draws() {
    let mut state = State::new(None, None);
    assert_eq!(drawn(&mut state, "1-looking"), vec![]);
}

#[test]
fn a_window_without_its_server_says_why_and_offers_to_look_again() {
    let mut state = State::new(None, None);
    state.apply(Event::Offline(
        "the IDE's server has left no session token at /run/user/1000/biomeos/gaia-ide.token \
         (No such file or directory (os error 2)). Is `gaia-ide` running?"
            .to_string(),
    ));
    assert_eq!(drawn(&mut state, "2-no-server"), vec![]);
}

#[test]
fn a_window_with_its_server_and_no_file_draws() {
    let mut state = connected();
    assert_eq!(drawn(&mut state, "3-connected"), vec![]);
}

#[test]
fn a_window_in_the_middle_of_work_draws() {
    let mut state = working();
    assert_eq!(drawn(&mut state, "4-working"), vec![]);
    assert_eq!(state.cursor, None, "nobody clicked into the text");
}

#[test]
fn a_window_after_the_change_was_applied_draws() {
    let mut state = working();
    let proposal = state
        .chat
        .iter()
        .position(|item| matches!(item, Item::Proposal(_)))
        .expect("a change was proposed");
    state.apply_proposal(proposal);
    state.answer(3, false, false);
    for message in [
        json!({"t": "result", "id": 3, "ok": false, "text": "not allowed"}),
        json!({"t": "delta", "text": "The change is proposed. I did not run the tests."}),
        json!({"t": "done", "how": "finished", "did": {
            "looked": 1, "proposed": 1, "commands": [], "reached": [], "refused": 1, "failed": 0,
        }}),
    ] {
        state.apply(Event::Agent(message));
    }
    assert_eq!(drawn(&mut state, "5-applied"), vec![]);
}

#[test]
fn closing_a_changed_file_shows_the_question() {
    let mut state = working();
    state.buffers[0].text.push_str("\n# changed\n");
    state.want_close(0);
    assert_eq!(drawn(&mut state, "6-closing"), vec![]);
    assert_eq!(state.buffers.len(), 1);
}

#[test]
fn a_save_refused_because_the_file_changed_shows_both_ways_out() {
    let mut state = working();
    state.buffers[0].text.push_str("\n# changed\n");
    state.apply(Event::Saved(
        "/home/me/shop/src/cart.py".to_string(),
        Err(NotSaved {
            why: "the file changed on disk since it was opened".to_string(),
            changed_on_disk: true,
        }),
    ));
    assert_eq!(drawn(&mut state, "8-changed-on-disk"), vec![]);
}

#[test]
fn a_folder_opened_in_the_tree_is_asked_for() {
    // Nothing is known of the workspace's own folder: drawing asks for it.
    let mut state = State::new(None, None);
    state.apply(Event::Connected {
        version: "1".to_string(),
        roots: vec!["/w".to_string()],
    });
    state.dirs.clear();
    let asked = drawn(&mut state, "7-tree-asks");
    assert_eq!(
        asked,
        vec![Action::List("/w".to_string())],
        "asked once, not once a frame"
    );
}

#[test]
fn ctrl_s_saves_the_file_shown_when_it_has_changes() {
    let key = |frame: usize| {
        if frame == 2 {
            vec![egui::Event::Key {
                key: egui::Key::S,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::COMMAND,
            }]
        } else {
            Vec::new()
        }
    };
    let mut state = working();
    let (_, asked) = picture::of_with(&mut state, WIDTH, HEIGHT, key);
    assert_eq!(asked, vec![], "nothing to save: nothing is asked");

    state.buffers[0].text.push_str("# more\n");
    let (_, asked) = picture::of_with(&mut state, WIDTH, HEIGHT, key);
    assert_eq!(asked, vec![Action::Save(0)]);
}

#[test]
fn every_sign_the_window_uses_is_in_its_fonts() {
    let ctx = egui::Context::default();
    style(&ctx);
    let _ = ctx.run(egui::RawInput::default(), |_| {});
    let signs = "•×✔✖…—·$?";
    let mut missing = Vec::new();
    for style in [
        egui::TextStyle::Body,
        egui::TextStyle::Monospace,
        egui::TextStyle::Small,
    ] {
        let font = style.resolve(&ctx.style());
        for sign in signs.chars() {
            if !ctx.fonts(|fonts| fonts.has_glyph(&font, sign)) {
                missing.push(format!("{sign:?} in {style:?}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "these would show as empty boxes: {missing:?}"
    );
}

#[test]
fn a_long_file_is_drawn_in_good_time() {
    let mut state = connected();
    let mut text = String::new();
    for n in 0..5000 {
        text.push_str(&format!(
            "def function_{n}(a, b):\n    return a + b  # line {n}\n"
        ));
    }
    state.apply(Event::Opened(
        "/home/me/shop/long.py".to_string(),
        Ok((text, Some(1))),
    ));

    let ctx = egui::Context::default();
    style(&ctx);
    let frame = |ctx: &egui::Context, state: &mut State, n: usize| {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(WIDTH, HEIGHT),
            )),
            time: Some(n as f64 / 60.0),
            ..Default::default()
        };
        let started = std::time::Instant::now();
        let output = ctx.run(input, |ctx| {
            draw(ctx, state);
        });
        let _ = ctx.tessellate(output.shapes, output.pixels_per_point);
        started.elapsed()
    };
    let first = frame(&ctx, &mut state, 0);
    let later: Vec<_> = (1..6).map(|n| frame(&ctx, &mut state, n)).collect();
    let slowest = later.iter().max().expect("five frames");
    // A key is pressed: the text changes, so all of it is coloured again.
    state.buffers[0].text.insert(0, '#');
    let typed = frame(&ctx, &mut state, 6);
    println!(
        "10,000 lines: first frame {first:?}, a frame at rest at most {slowest:?}, \
         a frame after a key {typed:?}"
    );
    // Tests are not built for speed; this only catches a frame that
    // takes seconds.
    assert!(slowest.as_millis() < 1500, "a frame took {slowest:?}");
}

#[test]
fn prose_and_code_are_told_apart() {
    assert_eq!(
        fenced("Look:\n```python\nx = 1\n```\nDone."),
        vec![
            (false, "Look:\n".to_string()),
            (true, "x = 1\n".to_string()),
            (false, "Done.".to_string()),
        ]
    );
    // Gaia is still writing: the fence is not closed yet.
    assert_eq!(
        fenced("Look:\n```\nx = "),
        vec![(false, "Look:\n".to_string()), (true, "x = ".to_string())]
    );
    assert_eq!(
        fenced("no code here"),
        vec![(false, "no code here".to_string())]
    );
    assert_eq!(fenced(""), vec![]);
}

#[test]
fn the_top_right_says_how_gaia_is_reached() {
    let mut state = connected();
    assert_eq!(brain_pill(&state).0, "brain: squirrel · 2 models");
    state.via = "engine".to_string();
    assert_eq!(brain_pill(&state).0, "engine · 2 models");
    state.apply(Event::Offline("gone".to_string()));
    assert_eq!(brain_pill(&state).0, "no server");
}

#[test]
fn a_result_is_cut_at_a_whole_character() {
    assert_eq!(clipped("héllo", 2), ("hé", true));
    assert_eq!(clipped("héllo", 5), ("héllo", false));
    assert_eq!(clipped("", 5), ("", false));
}

#[test]
fn a_folder_is_named_by_its_last_part() {
    assert_eq!(short_name("/home/me/shop"), "shop");
    assert_eq!(short_name("/home/me/shop/"), "shop");
    assert_eq!(short_name("/"), "/");
}

#[test]
fn the_indent_every_line_shares_is_found() {
    assert_eq!(
        shared_indent(["    a", "      b", "", "    c"].into_iter()),
        4
    );
    assert_eq!(shared_indent(["a", "    b"].into_iter()), 0);
    assert_eq!(shared_indent(["", "  "].into_iter()), 0);
    assert_eq!(shared_indent(std::iter::empty()), 0);
}
