// SPDX-License-Identifier: AGPL-3.0-or-later
//! What the window does, told without a display or a server.

use serde_json::json;

use super::*;

fn connected() -> State {
    let mut state = State::new(None, None);
    state.apply(Event::Connected {
        version: "4.57.0".to_string(),
        roots: vec!["/work".to_string(), "/other".to_string()],
    });
    state.apply(Event::Models(Ok((
        vec![
            Model {
                name: "tinyllama:latest".to_string(),
                tools: false,
            },
            Model {
                name: "qwen2.5-coder:14b".to_string(),
                tools: true,
            },
            Model {
                name: "gaia:latest".to_string(),
                tools: true,
            },
        ],
        "squirrel".to_string(),
    ))));
    state
}

fn with_file(path: &str, text: &str) -> State {
    let mut state = connected();
    state.apply(Event::Opened(
        path.to_string(),
        Ok((text.to_string(), Some(100))),
    ));
    state
}

fn agent(state: &mut State, message: serde_json::Value) {
    state.apply(Event::Agent(message));
}

fn entry(name: &str, is_dir: bool) -> Entry {
    Entry {
        name: name.to_string(),
        path: format!("/work/{name}"),
        is_dir,
    }
}

#[test]
fn the_first_folder_the_server_may_open_becomes_the_workspace() {
    let mut state = State::new(None, None);
    let next = state.apply(Event::Connected {
        version: "4.57.0".to_string(),
        roots: vec!["/work".to_string(), "/other".to_string()],
    });
    assert_eq!(state.server, Server::Ready("4.57.0".to_string()));
    assert_eq!(state.workspace, "/work");
    assert_eq!(
        next,
        vec![
            Action::Models,
            Action::Reach,
            Action::List("/work".to_string())
        ]
    );
    assert_eq!(state.dirs.get("/work"), Some(&Dir::Loading));
}

#[test]
fn a_folder_and_a_file_named_at_the_start_are_opened() {
    let mut state = State::new(
        Some("/other/deep".to_string()),
        Some("/other/deep/a.rs".to_string()),
    );
    let next = state.apply(Event::Connected {
        version: "1".to_string(),
        roots: vec!["/work".to_string(), "/other".to_string()],
    });
    assert_eq!(state.workspace, "/other/deep");
    assert!(next.contains(&Action::List("/other/deep".to_string())));
    assert!(next.contains(&Action::Open("/other/deep/a.rs".to_string())));
}

#[test]
fn a_folder_is_listed_folders_first_then_by_name_whatever_the_case() {
    let mut state = connected();
    state.apply(Event::Listed(
        "/work".to_string(),
        Ok(vec![
            entry("zeta.rs", false),
            entry("Makefile", false),
            entry("src", true),
            entry("alpha.rs", false),
            entry("Docs", true),
        ]),
    ));
    let Some(Dir::Listed(entries)) = state.dirs.get("/work") else {
        panic!("the folder is listed");
    };
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["Docs", "src", "alpha.rs", "Makefile", "zeta.rs"]);
}

#[test]
fn a_folder_that_cannot_be_listed_says_why() {
    let mut state = connected();
    state.apply(Event::Listed(
        "/work".to_string(),
        Err("permission denied".to_string()),
    ));
    assert_eq!(
        state.dirs.get("/work"),
        Some(&Dir::Failed("permission denied".to_string()))
    );
}

#[test]
fn a_folder_is_asked_about_once() {
    let mut state = connected();
    assert_eq!(
        state.want_listing("/work/src"),
        vec![Action::List("/work/src".to_string())]
    );
    assert_eq!(state.want_listing("/work/src"), vec![]);
}

#[test]
fn an_opened_file_is_shown_and_is_not_changed() {
    let state = with_file("/work/a.rs", "fn main() {}\n");
    let buffer = state.buffer().expect("a file is shown");
    assert_eq!(buffer.name(), "a.rs");
    assert_eq!(buffer.language(), "rs");
    assert!(!buffer.dirty());
}

#[test]
fn picking_a_file_that_is_open_keeps_what_was_typed_in_it() {
    let mut state = with_file("/work/a.rs", "one\n");
    state.buffers[0].text.push_str("two\n");
    state.apply(Event::Opened(
        "/work/b.rs".to_string(),
        Ok((String::new(), Some(1))),
    ));
    assert_eq!(state.current, Some(1));

    // Back to the first: nothing is read again, so nothing typed is lost.
    assert_eq!(state.want_file("/work/a.rs"), vec![]);
    assert_eq!(state.current, Some(0));
    assert_eq!(state.buffers[0].text, "one\ntwo\n");
}

#[test]
fn saving_is_asked_only_when_there_is_something_to_save() {
    let mut state = with_file("/work/a.rs", "one\n");
    assert_eq!(state.save_current(), vec![]);
    state.buffers[0].text.push_str("two\n");
    assert_eq!(state.save_current(), vec![Action::Save(0)]);
}

#[test]
fn a_saved_file_is_no_longer_changed_and_carries_its_new_time() {
    let mut state = with_file("/work/a.rs", "one\n");
    state.buffers[0].text.push_str("two\n");
    state.apply(Event::Saved(
        "/work/a.rs".to_string(),
        Ok(("one\ntwo\n".to_string(), Some(200))),
    ));
    let buffer = &state.buffers[0];
    assert!(!buffer.dirty());
    assert_eq!(buffer.mtime, Some(200));
}

#[test]
fn what_was_typed_while_the_file_was_being_saved_still_counts_as_changed() {
    let mut state = with_file("/work/a.rs", "one\n");
    state.buffers[0].text.push_str("two\n");
    // The save of "one\ntwo\n" is on its way; the user types on.
    state.buffers[0].text.push_str("three\n");
    state.apply(Event::Saved(
        "/work/a.rs".to_string(),
        Ok(("one\ntwo\n".to_string(), Some(200))),
    ));
    assert!(state.buffers[0].dirty());
    assert_eq!(state.buffers[0].text, "one\ntwo\nthree\n");
}

#[test]
fn a_refused_save_keeps_the_text_and_says_why() {
    let mut state = with_file("/work/a.rs", "one\n");
    state.buffers[0].text.push_str("two\n");
    state.apply(Event::Saved(
        "/work/a.rs".to_string(),
        Err(NotSaved {
            why: "permission denied".to_string(),
            changed_on_disk: false,
        }),
    ));
    let buffer = &state.buffers[0];
    assert!(buffer.dirty());
    assert_eq!(buffer.text, "one\ntwo\n");
    assert_eq!(buffer.mtime, Some(100));
    assert_eq!(
        buffer.notice.as_deref(),
        Some("Not saved: permission denied")
    );
    assert_eq!(
        state.write_over(0),
        vec![],
        "writing over is not offered for this"
    );
}

#[test]
fn a_file_changed_on_disk_is_not_written_over_until_the_user_says_which() {
    let mut state = with_file("/work/a.rs", "one\n");
    state.buffers[0].text.push_str("two\n");
    assert_eq!(state.write_over(0), vec![], "nothing was refused yet");

    state.apply(Event::Saved(
        "/work/a.rs".to_string(),
        Err(NotSaved {
            why: "the file changed on disk since it was opened".to_string(),
            changed_on_disk: true,
        }),
    ));
    assert_eq!(
        state.buffers[0].notice.as_deref(),
        Some(
            "Not saved: a.rs was changed on disk by something else after you opened it. \
             Nothing was written."
        )
    );
    assert_eq!(
        state.buffers[0].text, "one\ntwo\n",
        "what was typed is kept"
    );
    // Ctrl+S again is refused again by the server: it still carries the old time.
    assert_eq!(state.save_current(), vec![Action::Save(0)]);

    // One way out: the user's text goes over what is there.
    assert_eq!(state.write_over(0), vec![Action::SaveOver(0)]);
    state.apply(Event::Saved(
        "/work/a.rs".to_string(),
        Ok(("one\ntwo\n".to_string(), Some(300))),
    ));
    let buffer = &state.buffers[0];
    assert!(!buffer.dirty() && !buffer.changed_on_disk);
    assert_eq!((buffer.mtime, buffer.notice.as_deref()), (Some(300), None));
    assert_eq!(state.write_over(0), vec![]);
}

#[test]
fn a_file_changed_on_disk_can_be_read_again_dropping_what_was_typed() {
    let mut state = with_file("/work/a.rs", "one\n");
    state.buffers[0].text.push_str("two\n");
    state.apply(Event::Saved(
        "/work/a.rs".to_string(),
        Err(NotSaved {
            why: "the file changed on disk since it was opened".to_string(),
            changed_on_disk: true,
        }),
    ));
    assert_eq!(
        state.read_again(0),
        vec![Action::Open("/work/a.rs".to_string())]
    );
    state.apply(Event::Opened(
        "/work/a.rs".to_string(),
        Ok(("theirs\n".to_string(), Some(250))),
    ));
    assert_eq!(state.buffers.len(), 1, "the same file, not a second copy");
    let buffer = &state.buffers[0];
    assert_eq!(
        (buffer.text.as_str(), buffer.mtime),
        ("theirs\n", Some(250))
    );
    assert!(!buffer.dirty() && !buffer.changed_on_disk && buffer.notice.is_none());
}

#[test]
fn a_file_saved_into_a_listed_folder_has_the_folder_read_again() {
    let mut state = with_file("/work/a.rs", "one\n");
    state.apply(Event::Listed("/work".to_string(), Ok(vec![])));
    let next = state.apply(Event::Saved(
        "/work/a.rs".to_string(),
        Ok(("one\n".to_string(), Some(2))),
    ));
    assert_eq!(next, vec![Action::List("/work".to_string())]);
}

#[test]
fn closing_a_changed_file_is_asked_about_first() {
    let mut state = with_file("/work/a.rs", "one\n");
    state.buffers[0].text.push_str("two\n");
    state.want_close(0);
    assert_eq!(state.buffers.len(), 1, "nothing is closed yet");
    assert_eq!(state.closing, Some(0));

    state.close(0);
    assert!(state.buffers.is_empty());
    assert_eq!((state.current, state.closing), (None, None));
}

#[test]
fn closing_a_file_keeps_the_same_file_shown() {
    let mut state = connected();
    for name in ["a", "b", "c"] {
        state.apply(Event::Opened(
            format!("/work/{name}"),
            Ok((String::new(), Some(1))),
        ));
    }
    state.current = Some(2);
    state.want_close(0);
    assert_eq!(state.buffer().map(Buffer::name), Some("c"));
    state.want_close(1);
    assert_eq!(state.buffer().map(Buffer::name), Some("b"));
}

#[test]
fn only_models_that_can_use_tools_are_offered_and_gaia_is_chosen() {
    let state = connected();
    let names: Vec<&str> = state.models.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, ["qwen2.5-coder:14b", "gaia:latest"]);
    assert_eq!(state.model, "gaia:latest");
    assert_eq!(state.via, "squirrel");
}

#[test]
fn a_model_the_user_chose_stays_chosen_when_the_list_comes_again() {
    let mut state = connected();
    state.model = "qwen2.5-coder:14b".to_string();
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
        "engine".to_string(),
    ))));
    assert_eq!(state.model, "qwen2.5-coder:14b");
    assert_eq!(state.via, "engine");
}

#[test]
fn asking_sends_the_words_the_model_the_folder_and_the_open_file() {
    let mut state = with_file("/work/a.rs", "one\n");
    state.input = "  what does this do?  ".to_string();
    let next = state.ask();
    assert_eq!(
        next,
        vec![Action::Agent(json!({
            "t": "ask", "text": "what does this do?", "model": "gaia:latest",
            "workspace": "/work", "open": "/work/a.rs", "selection": null,
            "primals": false, "cursor": false,
        }))]
    );
    assert!(state.running);
    assert!(state.input.is_empty());
    assert_eq!(
        state.chat,
        vec![Item::User("what does this do?".to_string())]
    );
}

#[test]
fn the_open_file_is_not_told_when_the_user_says_not_to() {
    let mut state = with_file("/work/a.rs", "one\n");
    state.share_file = false;
    state.input = "hello".to_string();
    let Some(Action::Agent(sent)) = state.ask().pop() else {
        panic!("something is sent");
    };
    assert_eq!(sent["open"], json!(null));
}

#[test]
fn nothing_is_asked_with_no_words_or_while_gaia_is_answering() {
    let mut state = connected();
    state.input = "   ".to_string();
    assert_eq!(state.ask(), vec![]);

    state.input = "one".to_string();
    assert_eq!(state.ask().len(), 1);
    state.input = "two".to_string();
    assert_eq!(state.ask(), vec![], "one question at a time");
    assert_eq!(state.input, "two", "what was typed is kept");
}

#[test]
fn reaching_past_the_folder_is_sent_only_when_this_device_can_and_the_user_said_so() {
    let mut state = connected();
    state.use_cursor = true;
    state.use_primals = true;
    state.input = "one".to_string();
    let Some(Action::Agent(sent)) = state.ask().pop() else {
        panic!("something is sent");
    };
    assert_eq!(
        (&sent["primals"], &sent["cursor"]),
        (&json!(false), &json!(false))
    );

    let mut state = connected();
    state.apply(Event::Reach {
        primals: true,
        cursor: true,
    });
    assert!(!state.use_primals && !state.use_cursor, "both start off");
    state.use_primals = true;
    state.input = "one".to_string();
    let Some(Action::Agent(sent)) = state.ask().pop() else {
        panic!("something is sent");
    };
    assert_eq!(
        (&sent["primals"], &sent["cursor"]),
        (&json!(true), &json!(false))
    );
}

#[test]
fn what_gaia_writes_arrives_piece_by_piece_and_a_tool_call_parts_it() {
    let mut state = connected();
    state.input = "fix it".to_string();
    state.ask();
    agent(&mut state, json!({"t": "delta", "text": "Let me "}));
    agent(&mut state, json!({"t": "delta", "text": "look."}));
    agent(
        &mut state,
        json!({"t": "tool", "id": 1, "name": "read_file", "summary": "read src/cart.py", "ask": false}),
    );
    agent(
        &mut state,
        json!({"t": "result", "id": 1, "ok": true, "text": "def total(): ..."}),
    );
    agent(&mut state, json!({"t": "turn"}));
    agent(&mut state, json!({"t": "delta", "text": "Found it."}));

    assert_eq!(state.chat.len(), 4);
    assert_eq!(state.chat[1], Item::Gaia("Let me look.".to_string()));
    let Item::Step(step) = &state.chat[2] else {
        panic!("the third thing is the tool call");
    };
    assert_eq!(step.state, StepState::Done(true));
    assert_eq!(step.result.as_deref(), Some("def total(): ..."));
    assert_eq!(state.chat[3], Item::Gaia("Found it.".to_string()));
}

#[test]
fn a_command_waits_for_the_users_answer_and_is_answered_once() {
    let mut state = connected();
    state.input = "run the tests".to_string();
    state.ask();
    agent(
        &mut state,
        json!({"t": "tool", "id": 7, "name": "run_command", "summary": "pytest -q", "ask": true}),
    );
    let Some(Item::Step(step)) = state.chat.last() else {
        panic!("the command is in the chat");
    };
    assert_eq!(step.state, StepState::Asking);

    assert_eq!(
        state.answer(7, true, false),
        vec![Action::Agent(
            json!({"t": "allow", "id": 7, "always": false})
        )]
    );
    assert_eq!(
        state.answer(7, true, false),
        vec![],
        "a second click does nothing"
    );
    assert_eq!(
        state.answer(8, true, false),
        vec![],
        "nor a click on what was never asked"
    );
}

#[test]
fn a_refusal_is_sent_as_a_refusal() {
    let mut state = connected();
    agent(
        &mut state,
        json!({"t": "tool", "id": 7, "name": "run_command", "summary": "rm -rf build", "ask": true, "warn": "This deletes files."}),
    );
    let Some(Item::Step(step)) = state.chat.last() else {
        panic!("the command is in the chat");
    };
    assert_eq!(step.warn.as_deref(), Some("This deletes files."));
    assert_eq!(
        state.answer(7, false, false),
        vec![Action::Agent(json!({"t": "deny", "id": 7}))]
    );
}

#[test]
fn a_call_that_is_asked_every_time_is_never_allowed_for_the_whole_chat() {
    let mut state = connected();
    agent(
        &mut state,
        json!({"t": "tool", "id": 3, "name": "ask_cursor", "summary": "ask Cursor: …", "ask": true, "once": true}),
    );
    // Even if "always" got through somehow, it is not sent.
    assert_eq!(
        state.answer(3, true, true),
        vec![Action::Agent(
            json!({"t": "allow", "id": 3, "always": false})
        )]
    );
}

fn proposed(state: &mut State, proposal: serde_json::Value) -> usize {
    agent(
        state,
        json!({"t": "tool", "id": 2, "name": "propose_edit", "summary": "change a file", "ask": false}),
    );
    agent(
        state,
        json!({"t": "result", "id": 2, "ok": true, "text": "proposed", "proposal": proposal}),
    );
    state.chat.len() - 1
}

fn applied(state: &State, n: usize) -> Option<Result<(), String>> {
    match &state.chat[n] {
        Item::Proposal(proposal) => proposal.applied.clone(),
        other => panic!("not a proposal: {other:?}"),
    }
}

#[test]
fn a_proposed_change_goes_into_the_editor_and_not_onto_the_disk() {
    let mut state = with_file("/work/cart.py", "a = 1\nb = a * 2\nc = 3\n");
    let n = proposed(
        &mut state,
        json!({"file": "/work/cart.py", "old": "b = a * 2\n", "fresh": "b = a + 2\n"}),
    );
    assert_eq!(
        applied(&state, n),
        None,
        "nothing happens until the user applies it"
    );
    assert_eq!(state.buffers[0].text, "a = 1\nb = a * 2\nc = 3\n");

    let next = state.apply_proposal(n);
    assert_eq!(
        next,
        vec![],
        "applying asks the server for nothing: nothing is written"
    );
    assert_eq!(state.buffers[0].text, "a = 1\nb = a + 2\nc = 3\n");
    assert!(
        state.buffers[0].dirty(),
        "the file shows as changed until the user saves"
    );
    assert_eq!(applied(&state, n), Some(Ok(())));

    // Applying it again does not apply it twice.
    state.apply_proposal(n);
    assert_eq!(state.buffers[0].text, "a = 1\nb = a + 2\nc = 3\n");
}

#[test]
fn a_change_whose_lines_are_not_in_the_file_is_not_applied() {
    let mut state = with_file("/work/cart.py", "a = 1\n");
    let n = proposed(
        &mut state,
        json!({"file": "/work/cart.py", "old": "b = 2\n", "fresh": "b = 3\n"}),
    );
    state.apply_proposal(n);
    assert_eq!(state.buffers[0].text, "a = 1\n");
    assert_eq!(
        applied(&state, n),
        Some(Err(
            "The lines to replace are not in the file as it is in the editor now.".to_string()
        ))
    );
}

#[test]
fn a_change_whose_lines_are_in_the_file_twice_is_not_applied() {
    let mut state = with_file("/work/cart.py", "x = 1\nx = 1\n");
    let n = proposed(
        &mut state,
        json!({"file": "/work/cart.py", "old": "x = 1\n", "fresh": "x = 2\n"}),
    );
    state.apply_proposal(n);
    assert_eq!(state.buffers[0].text, "x = 1\nx = 1\n");
    assert!(matches!(applied(&state, n), Some(Err(why)) if why.contains("more than once")));
}

#[test]
fn a_change_to_a_file_that_is_not_open_reads_the_file_first() {
    let mut state = connected();
    let n = proposed(
        &mut state,
        json!({"file": "/work/cart.py", "old": "b = 2\n", "fresh": "b = 3\n"}),
    );
    assert_eq!(
        state.apply_proposal(n),
        vec![Action::Open("/work/cart.py".to_string())]
    );
    assert_eq!(applied(&state, n), None);

    state.apply(Event::Opened(
        "/work/cart.py".to_string(),
        Ok(("a = 1\nb = 2\n".to_string(), Some(5))),
    ));
    assert_eq!(state.buffers[0].text, "a = 1\nb = 3\n");
    assert_eq!(state.buffers[0].saved, "a = 1\nb = 2\n");
    assert_eq!(applied(&state, n), Some(Ok(())));
}

#[test]
fn a_change_to_a_file_that_cannot_be_read_says_so_on_the_change() {
    let mut state = connected();
    let n = proposed(
        &mut state,
        json!({"file": "/work/gone.py", "old": "a", "fresh": "b"}),
    );
    state.apply_proposal(n);
    state.apply(Event::Opened(
        "/work/gone.py".to_string(),
        Err("no such file".to_string()),
    ));
    assert_eq!(
        applied(&state, n),
        Some(Err("Cannot open /work/gone.py: no such file".to_string()))
    );
}

#[test]
fn a_proposed_new_file_is_in_the_editor_and_not_on_disk() {
    let mut state = connected();
    let n = proposed(
        &mut state,
        json!({"file": "/work/new.py", "whole": "print('hi')\n"}),
    );
    assert_eq!(state.apply_proposal(n), vec![]);
    let buffer = state.buffer().expect("the new file is shown");
    assert_eq!(buffer.path, "/work/new.py");
    assert_eq!(buffer.mtime, None);
    assert!(buffer.dirty());
    assert_eq!(state.save_current(), vec![Action::Save(0)]);
}

#[test]
fn the_end_of_a_run_says_what_really_happened() {
    let mut state = connected();
    state.input = "fix and test".to_string();
    state.ask();
    agent(&mut state, json!({"t": "delta", "text": "The tests pass."}));
    agent(
        &mut state,
        json!({"t": "done", "how": "finished", "did": {
            "looked": 3, "proposed": 1,
            "commands": [{"command": "pytest -q", "exit": 1}],
            "reached": [], "refused": 1, "failed": 0,
        }}),
    );
    assert!(!state.running);
    assert_eq!(
        state.chat.last(),
        Some(&Item::Facts(
            "What actually happened: 1 change proposed — none is in the file until you apply it \
             and save · ran pytest -q, exit code 1 (failed) · 1 step you did not allow."
                .to_string()
        ))
    );
}

#[test]
fn a_run_that_only_looked_has_no_line_of_facts() {
    let mut state = connected();
    state.input = "what is this?".to_string();
    state.ask();
    agent(&mut state, json!({"t": "delta", "text": "A cart."}));
    agent(
        &mut state,
        json!({"t": "done", "how": "finished", "did": {
            "looked": 2, "proposed": 0, "commands": [], "reached": [], "refused": 0, "failed": 0,
        }}),
    );
    assert_eq!(state.chat.last(), Some(&Item::Gaia("A cart.".to_string())));
}

#[test]
fn a_run_that_ends_without_a_word_says_so() {
    let mut state = connected();
    state.input = "hello".to_string();
    state.ask();
    agent(
        &mut state,
        json!({"t": "done", "how": "finished", "did": {}}),
    );
    assert!(
        matches!(state.chat.last(), Some(Item::Note(text)) if text.starts_with("Gaia finished without saying anything"))
    );
}

#[test]
fn stopping_sends_stop_and_the_end_is_noted() {
    let mut state = connected();
    assert_eq!(state.stop(), vec![], "nothing to stop");
    state.input = "hello".to_string();
    state.ask();
    assert_eq!(state.stop(), vec![Action::Agent(json!({"t": "stop"}))]);
    agent(
        &mut state,
        json!({"t": "tool", "id": 4, "name": "run_command", "summary": "sleep 99", "ask": true}),
    );
    agent(
        &mut state,
        json!({"t": "done", "how": "stopped", "did": {}}),
    );
    assert!(!state.running);
    assert!(state.chat.contains(&Item::Note("Stopped.".to_string())));
    // The question that was open can no longer be answered.
    assert_eq!(state.answer(4, true, false), vec![]);
}

#[test]
fn an_error_from_the_server_is_shown_as_it_was_said() {
    let mut state = connected();
    state.input = "hello".to_string();
    state.ask();
    agent(
        &mut state,
        json!({"t": "error", "error": "model 'nope' not found"}),
    );
    agent(&mut state, json!({"t": "done", "how": "failed", "did": {}}));
    assert_eq!(
        state.chat.last(),
        Some(&Item::Error("model 'nope' not found".to_string()))
    );
    assert!(!state.running);
}

#[test]
fn a_connection_lost_in_the_middle_is_said_and_one_lost_at_rest_is_not() {
    let mut state = connected();
    state.apply(Event::AgentClosed("the server closed it".to_string()));
    assert!(state.chat.is_empty());

    state.input = "hello".to_string();
    state.ask();
    state.apply(Event::AgentClosed("the server closed it".to_string()));
    assert!(!state.running);
    assert!(
        matches!(state.chat.last(), Some(Item::Error(text)) if text.contains("the server closed it"))
    );
}

#[test]
fn clearing_forgets_the_conversation_here_and_on_the_server() {
    let mut state = connected();
    assert_eq!(state.clear_chat(), vec![], "nothing to forget");
    state.input = "hello".to_string();
    state.ask();
    assert_eq!(state.clear_chat(), vec![], "not while Gaia is answering");
    agent(
        &mut state,
        json!({"t": "done", "how": "finished", "did": {}}),
    );
    assert_eq!(
        state.clear_chat(),
        vec![Action::Agent(json!({"t": "reset"}))]
    );
    assert!(state.chat.is_empty());
}

#[test]
fn a_server_that_is_not_there_is_remembered_with_its_reason() {
    let mut state = State::new(None, None);
    state.apply(Event::Offline(
        "nothing is listening at 127.0.0.1:3142".to_string(),
    ));
    assert_eq!(
        state.server,
        Server::Offline("nothing is listening at 127.0.0.1:3142".to_string())
    );
}

#[test]
fn choosing_another_folder_reads_it_and_forgets_the_old_tree() {
    let mut state = connected();
    state.apply(Event::Listed(
        "/work".to_string(),
        Ok(vec![entry("a.rs", false)]),
    ));
    assert_eq!(
        state.set_workspace("/other/"),
        vec![Action::List("/other".to_string())]
    );
    assert_eq!(state.workspace, "/other");
    assert!(!state.dirs.contains_key("/work"));
    assert_eq!(
        state.set_workspace("/other"),
        vec![],
        "the same folder is not read again"
    );
    assert_eq!(state.set_workspace("   "), vec![]);
}

#[test]
fn the_folder_of_a_path() {
    assert_eq!(parent_of("/work/src/a.rs"), "/work/src");
    assert_eq!(parent_of("/a.rs"), "/");
    assert_eq!(parent_of("a.rs"), "");
}

#[test]
fn files_are_coloured_by_their_ending() {
    let named = |path: &str| Buffer {
        path: path.to_string(),
        text: String::new(),
        saved: String::new(),
        mtime: None,
        notice: None,
        changed_on_disk: false,
    };
    assert_eq!(named("/w/a.py").language(), "py");
    assert_eq!(named("/w/Cargo.toml").language(), "toml");
    assert_eq!(named("/w/app.ts").language(), "cpp");
    assert_eq!(named("/w/README.md").language(), "");
    assert_eq!(named("/w/Makefile").language(), "");
}

#[test]
fn a_tool_call_the_model_wrote_out_as_text_is_not_shown_twice() {
    let mut state = connected();
    state.input = "fix it".to_string();
    state.ask();
    agent(
        &mut state,
        json!({"t": "delta", "text": "{\n  \"name\": \"read_file\",\n"}),
    );
    agent(
        &mut state,
        json!({"t": "delta", "text": "  \"arguments\": {\"path\": \"src/cart.py\"}\n}"}),
    );
    agent(
        &mut state,
        json!({"t": "tool", "id": 1, "name": "read_file", "summary": "read src/cart.py", "ask": false}),
    );
    assert_eq!(
        state.chat.len(),
        2,
        "the question and the step: {:?}",
        state.chat
    );
    assert!(matches!(&state.chat[1], Item::Step(step) if step.summary == "read src/cart.py"));
}

#[test]
fn what_gaia_says_before_a_tool_call_is_kept() {
    let mut state = connected();
    state.input = "fix it".to_string();
    state.ask();
    agent(
        &mut state,
        json!({"t": "delta", "text": "The \"name\" field is wrong; I will read the file."}),
    );
    agent(
        &mut state,
        json!({"t": "tool", "id": 1, "name": "read_file", "summary": "read src/cart.py", "ask": false}),
    );
    assert_eq!(state.chat.len(), 3);
}

#[test]
fn a_call_written_out_is_known_by_its_shape() {
    assert!(is_a_call_written_out(
        r#"{"name": "read_file", "arguments": {"path": "a"}}"#
    ));
    // Not valid JSON (single quotes), as a model really wrote it.
    assert!(is_a_call_written_out(
        "{\n \"name\": \"edit_file\",\n \"arguments\": {\"old_text\": 'a[\"b\"]'}\n}\n"
    ));
    assert!(is_a_call_written_out(
        "```json\n{\"name\": \"x\", \"parameters\": {}}\n```"
    ));
    assert!(!is_a_call_written_out(
        "Here is the call: {\"name\": \"x\", \"arguments\": {}}"
    ));
    assert!(!is_a_call_written_out("{\"name\": \"x\"}"));
    assert!(!is_a_call_written_out(""));
}
