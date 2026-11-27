// SPDX-License-Identifier: AGPL-3.0-or-later
//! What the Gaia IDE window shows.
//!
//! One function, [`draw`], from the state to the screen. It changes the
//! state only where the user typed or clicked, and returns what the link
//! should be asked for.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::Arc;

use egui::{Align, Color32, Key, Layout, Modifiers, RichText, ScrollArea, TextEdit};

use super::state::{Action, Buffer, Change, Dir, Item, Proposal, Server, State, Step, StepState};

/// The most lines of a proposed change shown in the chat. The whole of it
/// is seen in the editor once applied.
const DIFF_LINES: usize = 24;
/// The most characters of a tool's result shown in the chat.
const RESULT_CHARS: usize = 4000;

/// Draw the window. Returns what to ask the server for.
pub fn draw(ctx: &egui::Context, state: &mut State) -> Vec<Action> {
    let mut actions = Vec::new();

    if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::S)) {
        actions.extend(state.save_current());
    }

    top_bar(ctx, state, &mut actions);
    status_bar(ctx, state);

    if matches!(state.server, Server::Ready(_)) {
        egui::SidePanel::left("ide_tree")
            .default_width(250.0)
            .width_range(140.0..=600.0)
            .show(ctx, |ui| tree_panel(ui, state, &mut actions));
        egui::SidePanel::right("ide_chat")
            .default_width(430.0)
            .width_range(300.0..=900.0)
            .show(ctx, |ui| chat_panel(ui, state, &mut actions));
        egui::CentralPanel::default().show(ctx, |ui| editor_panel(ui, state, &mut actions));
    } else {
        egui::CentralPanel::default().show(ctx, |ui| no_server(ui, state, &mut actions));
    }
    actions
}

/// The sizes of the window's text, in both the dark and the light theme.
/// egui's own sizes are made for small tool panels; an editor is read for
/// hours.
pub fn style(ctx: &egui::Context) {
    use egui::{FontFamily, FontId, TextStyle};
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(22.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(14.5, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(14.5, FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(12.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(13.5, FontFamily::Monospace),
            ),
        ]
        .into();
    });
}

/// Text in the fixed-width font, a little smaller than the editor's: for
/// commands, results and the lines of a change, where columns matter.
fn fixed(text: impl Into<String>) -> RichText {
    RichText::new(text).font(egui::FontId::monospace(12.0))
}

fn green(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(110, 200, 130)
    } else {
        Color32::from_rgb(20, 120, 50)
    }
}

fn top_bar(ctx: &egui::Context, state: &mut State, actions: &mut Vec<Action>) {
    egui::TopBottomPanel::top("ide_top").show(ctx, |ui| {
        ui.add_space(3.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Gaia IDE").strong());
            ui.separator();

            if matches!(state.server, Server::Ready(_)) {
                ui.label("folder");
                let typed = ui.add(
                    TextEdit::singleline(&mut state.workspace_typed)
                        .desired_width(360.0)
                        .hint_text("a folder to work in"),
                );
                if typed.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    let wanted = state.workspace_typed.clone();
                    actions.extend(state.set_workspace(&wanted));
                }
                if state.roots.len() > 1 {
                    let mut picked = None;
                    egui::ComboBox::from_id_salt("ide_roots")
                        .selected_text("folders")
                        .show_ui(ui, |ui| {
                            for root in &state.roots {
                                if ui
                                    .selectable_label(*root == state.workspace, root)
                                    .clicked()
                                {
                                    picked = Some(root.clone());
                                }
                            }
                        });
                    if let Some(root) = picked {
                        actions.extend(state.set_workspace(&root));
                    }
                }
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let (text, about) = brain_pill(state);
                ui.label(RichText::new(text).small()).on_hover_text(about);
            });
        });
        ui.add_space(3.0);
    });
}

/// The words at the top right: how Gaia is reached, and what that means.
pub fn brain_pill(state: &State) -> (String, &'static str) {
    match &state.server {
        Server::Connecting => (
            "looking for the IDE's server…".to_string(),
            "The window is asking the IDE's server on this device what it has.",
        ),
        Server::Offline(_) => (
            "no server".to_string(),
            "The IDE's server on this device did not answer.",
        ),
        Server::Ready(_) if state.via == "squirrel" => (
            format!("brain: squirrel · {} models", state.models.len()),
            "Gaia is asked through squirrel, which chooses where each model runs. \
             The models counted are the ones that can use tools.",
        ),
        Server::Ready(_) if state.via.is_empty() => (
            "asking for the models…".to_string(),
            "The server is there; its list of models has not come yet.",
        ),
        Server::Ready(_) => (
            format!("engine · {} models", state.models.len()),
            "Gaia is asked at the model engine on this device directly, not through squirrel. \
             The models counted are the ones that can use tools.",
        ),
    }
}

fn status_bar(ctx: &egui::Context, state: &State) {
    egui::TopBottomPanel::bottom("ide_status").show(ctx, |ui| {
        ui.horizontal(|ui| match state.buffer() {
            Some(buffer) => {
                ui.label(RichText::new(&buffer.path).small());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let (text, colour) = if buffer.mtime.is_none() && !buffer.dirty() {
                        ("empty, not on disk", ui.visuals().weak_text_color())
                    } else if buffer.dirty() {
                        ("changed — Ctrl+S saves", ui.visuals().warn_fg_color)
                    } else {
                        ("saved", ui.visuals().weak_text_color())
                    };
                    ui.label(RichText::new(text).small().color(colour));
                    if let Some((line, column)) = state.cursor {
                        ui.label(
                            RichText::new(format!("Ln {line}, Col {column}"))
                                .small()
                                .weak(),
                        );
                    }
                });
            }
            None => {
                ui.label(RichText::new("no file open").small().weak());
            }
        });
    });
}

fn no_server(ui: &mut egui::Ui, state: &State, actions: &mut Vec<Action>) {
    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        ui.set_max_width(640.0);
        match &state.server {
            Server::Offline(why) => {
                ui.heading("The IDE's server is not answering");
                ui.add_space(8.0);
                ui.label(why);
                ui.add_space(8.0);
                ui.label(
                    RichText::new(
                        "This window is the face; `gaia-ide` is the part that reads files and \
                         reaches Gaia. Start it, then look again.",
                    )
                    .weak(),
                );
                ui.add_space(12.0);
                if ui.button("Look again").clicked() {
                    actions.push(Action::Connect);
                }
            }
            _ => {
                ui.heading("Looking for the IDE's server…");
            }
        }
    });
}

// ───────────────────────────── the file tree ─────────────────────────────

fn tree_panel(ui: &mut egui::Ui, state: &mut State, actions: &mut Vec<Action>) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(short_name(&state.workspace)).strong())
            .on_hover_text(&state.workspace);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .small_button("reread")
                .on_hover_text("Read the folders again from disk")
                .clicked()
            {
                actions.extend(state.open_workspace());
            }
        });
    });
    ui.separator();

    let mut wants = Vec::new();
    let current = state.buffer().map(|b| b.path.clone());
    ScrollArea::both()
        .id_salt("ide_tree_scroll")
        .auto_shrink(false)
        .show(ui, |ui| {
            tree(
                ui,
                &state.dirs,
                &state.workspace,
                current.as_deref(),
                &mut wants,
            );
        });
    for want in wants {
        match want {
            Want::List(path) => actions.extend(state.want_listing(&path)),
            Want::File(path) => actions.extend(state.want_file(&path)),
        }
    }
}

enum Want {
    List(String),
    File(String),
}

fn tree(
    ui: &mut egui::Ui,
    dirs: &BTreeMap<String, Dir>,
    path: &str,
    current: Option<&str>,
    wants: &mut Vec<Want>,
) {
    match dirs.get(path) {
        None => {
            wants.push(Want::List(path.to_string()));
            ui.weak("reading…");
        }
        Some(Dir::Loading) => {
            ui.weak("reading…");
        }
        Some(Dir::Failed(why)) => {
            ui.colored_label(ui.visuals().error_fg_color, why);
        }
        Some(Dir::Listed(entries)) if entries.is_empty() => {
            ui.weak("(empty)");
        }
        Some(Dir::Listed(entries)) => {
            for entry in entries {
                if entry.is_dir {
                    egui::CollapsingHeader::new(&entry.name)
                        .id_salt(&entry.path)
                        .show(ui, |ui| tree(ui, dirs, &entry.path, current, wants));
                } else if ui
                    .selectable_label(current == Some(entry.path.as_str()), &entry.name)
                    .clicked()
                {
                    wants.push(Want::File(entry.path.clone()));
                }
            }
        }
    }
}

fn short_name(path: &str) -> &str {
    match path.trim_end_matches('/').rsplit('/').next() {
        Some(name) if !name.is_empty() => name,
        _ => path,
    }
}

// ─────────────────────────────── the editor ──────────────────────────────

fn editor_panel(ui: &mut egui::Ui, state: &mut State, actions: &mut Vec<Action>) {
    if state.buffers.is_empty() {
        welcome(ui);
        return;
    }

    // The open files, one tab each.
    let mut show = None;
    let mut close = None;
    ui.horizontal_wrapped(|ui| {
        for (n, buffer) in state.buffers.iter().enumerate() {
            let name = if buffer.dirty() {
                format!("• {}", buffer.name())
            } else {
                buffer.name().to_string()
            };
            if ui
                .selectable_label(state.current == Some(n), name)
                .on_hover_text(&buffer.path)
                .clicked()
            {
                show = Some(n);
            }
            if ui
                .small_button("×")
                .on_hover_text("Close this file")
                .clicked()
            {
                close = Some(n);
            }
            ui.add_space(6.0);
        }
    });
    if let Some(n) = show {
        state.current = Some(n);
    }
    if let Some(n) = close {
        state.want_close(n);
    }
    ui.separator();

    // Closing a file with changes is asked about, never done silently.
    if let Some(n) = state.closing {
        let name = state.buffers.get(n).map(|b| b.name().to_string());
        if let Some(name) = name {
            let mut save_first = false;
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    format!("{name} has changes that are not saved."),
                );
                if ui.button("Save").clicked() {
                    save_first = true;
                }
                if ui.button("Close without saving").clicked() {
                    state.close(n);
                }
                if ui.button("Keep it open").clicked() {
                    state.closing = None;
                }
            });
            if save_first {
                state.closing = None;
                actions.push(Action::Save(n));
            }
            ui.separator();
        } else {
            state.closing = None;
        }
    }

    let Some(n) = state.current.filter(|n| *n < state.buffers.len()) else {
        return;
    };
    if let Some(notice) = state.buffers[n].notice.clone() {
        let changed_on_disk = state.buffers[n].changed_on_disk;
        ui.horizontal_wrapped(|ui| {
            ui.colored_label(ui.visuals().warn_fg_color, notice);
            if changed_on_disk {
                if ui
                    .button("read it again from disk")
                    .on_hover_text(
                        "Shows the file as it is on disk now. What you typed here is dropped.",
                    )
                    .clicked()
                {
                    actions.extend(state.read_again(n));
                }
                if ui
                    .button("save mine over it")
                    .on_hover_text(
                        "Writes what is in the editor. What the other program wrote is lost.",
                    )
                    .clicked()
                {
                    actions.extend(state.write_over(n));
                }
            }
        });
        ui.separator();
    }
    state.cursor = code(ui, &mut state.buffers[n]);
}

/// The text of a file, with line numbers and colours. Returns where the
/// cursor is: line and column, from 1.
fn code(ui: &mut egui::Ui, buffer: &mut Buffer) -> Option<(usize, usize)> {
    let language = buffer.language();
    let theme = egui_extras::syntax_highlighting::CodeTheme::from_style(ui.style());
    let mut layouter = |ui: &egui::Ui, text: &str, _wrap_width: f32| {
        let mut job = egui_extras::syntax_highlighting::highlight(
            ui.ctx(),
            ui.style(),
            &theme,
            text,
            language,
        );
        // Code is not wrapped: a long line is scrolled to.
        job.wrap.max_width = f32::INFINITY;
        ui.fonts(|fonts| fonts.layout_job(job))
    };

    let lines = buffer.text.bytes().filter(|b| *b == b'\n').count() + 1;
    let numbers = line_numbers(ui.ctx(), lines);
    let row = ui.text_style_height(&egui::TextStyle::Monospace);
    let rows = ((ui.available_height() / row) as usize).max(4);
    let visible = ui.available_width();

    let mut cursor = None;
    ScrollArea::both()
        .id_salt(("ide_code", &buffer.path))
        .auto_shrink(false)
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let gutter = ui
                    .vertical(|ui| {
                        // The editor's text starts two points down.
                        ui.add_space(2.0);
                        ui.add(
                            egui::Label::new(RichText::new(&*numbers).monospace().weak())
                                .selectable(false)
                                .extend(),
                        );
                    })
                    .response
                    .rect
                    .width();
                let shown = TextEdit::multiline(&mut buffer.text)
                    .id_salt(("ide_text", &buffer.path))
                    .code_editor()
                    .frame(false)
                    .desired_width((visible - gutter - 24.0).max(100.0))
                    .desired_rows(rows)
                    .layouter(&mut layouter)
                    .show(ui);
                cursor = shown.cursor_range.map(|range| {
                    let at = range.primary.pcursor;
                    (at.paragraph + 1, at.offset + 1)
                });
            });
        });
    cursor
}

/// `1\n2\n3…`, right-aligned, kept from frame to frame while the number
/// of lines stays the same.
fn line_numbers(ctx: &egui::Context, lines: usize) -> Arc<str> {
    let id = egui::Id::new("ide_line_numbers");
    if let Some((kept_for, kept)) = ctx.data(|d| d.get_temp::<(usize, Arc<str>)>(id))
        && kept_for == lines
    {
        return kept;
    }
    let width = lines.to_string().len();
    let mut numbers = String::with_capacity(lines * (width + 1));
    for n in 1..=lines {
        let _ = writeln!(numbers, "{n:>width$}");
    }
    numbers.pop();
    let numbers: Arc<str> = numbers.into();
    ctx.data_mut(|d| d.insert_temp(id, (lines, Arc::clone(&numbers))));
    numbers
}

fn welcome(ui: &mut egui::Ui) {
    ui.add_space(30.0);
    ui.vertical_centered(|ui| {
        ui.set_max_width(620.0);
        ui.heading("Gaia IDE");
        ui.add_space(8.0);
        ui.label(
            "Your editor, in a window of its own. Nothing here comes from the internet. \
             Files stay on this device; chat goes only to the models on it, by the way \
             shown at the top right.",
        );
        ui.add_space(12.0);
        ui.label("Pick a file in the tree on the left. Ctrl+S saves it.");
        ui.add_space(4.0);
        ui.label(
            "Gaia, on the right, reads and searches this folder herself, proposes changes \
             you put into the editor with one click, and asks before running any command.",
        );
        ui.add_space(12.0);
        ui.label(
            RichText::new(
                "This window is younger than the IDE's page in the browser. The terminal, \
                 the list of changes since the last commit, finding a file by name, and \
                 renaming or deleting files are on the page and not here yet.",
            )
            .weak(),
        );
    });
}

// ──────────────────────────────── the chat ───────────────────────────────

/// What was clicked in the conversation; applied once it is no longer
/// being read.
enum Click {
    Answer { id: u64, allow: bool, always: bool },
    Apply(usize),
}

fn chat_panel(ui: &mut egui::Ui, state: &mut State, actions: &mut Vec<Action>) {
    egui::TopBottomPanel::top("ide_chat_top")
        .show_separator_line(true)
        .show_inside(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Gaia").strong());
                if state.models.is_empty() {
                    ui.weak("no model that can use tools");
                } else {
                    egui::ComboBox::from_id_salt("ide_model")
                        .selected_text(&state.model)
                        .width(210.0)
                        .show_ui(ui, |ui| {
                            for model in &state.models {
                                ui.selectable_value(
                                    &mut state.model,
                                    model.name.clone(),
                                    &model.name,
                                );
                            }
                        });
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add_enabled(
                            !state.running && !state.chat.is_empty(),
                            egui::Button::new("clear").small(),
                        )
                        .on_hover_text("Forget this conversation and start a new one")
                        .clicked()
                    {
                        actions.extend(state.clear_chat());
                    }
                });
            });
            ui.add_space(4.0);
        });

    egui::TopBottomPanel::bottom("ide_chat_bottom")
        .show_separator_line(true)
        .show_inside(ui, |ui| chat_input(ui, state, actions));

    let mut clicks = Vec::new();
    egui::CentralPanel::default().show_inside(ui, |ui| {
        ScrollArea::vertical()
            .id_salt("ide_chat_scroll")
            .auto_shrink(false)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                if state.chat.is_empty() {
                    ui.weak(
                        "Ask Gaia to find, explain or change something in this folder. \
                         She reads and searches it herself. A change she proposes is \
                         not in the file until you apply it and save. She asks before \
                         running any command.",
                    );
                }
                for (n, item) in state.chat.iter().enumerate() {
                    chat_item(ui, n, item, &mut clicks);
                    ui.add_space(6.0);
                }
                if state.running {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.weak("Gaia is working…");
                    });
                }
            });
    });
    for click in clicks {
        match click {
            Click::Answer { id, allow, always } => {
                actions.extend(state.answer(id, allow, always));
            }
            Click::Apply(n) => actions.extend(state.apply_proposal(n)),
        }
    }
}

fn chat_input(ui: &mut egui::Ui, state: &mut State, actions: &mut Vec<Action>) {
    ui.add_space(6.0);
    let input = egui::Id::new("ide_chat_input");
    let focused = ui.memory(|m| m.has_focus(input));
    let by_key = focused && ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Enter));
    ui.add(
        TextEdit::multiline(&mut state.input)
            .id(input)
            .desired_rows(3)
            .desired_width(f32::INFINITY)
            .hint_text("Ask Gaia to find, explain or change something… (Ctrl+Enter)"),
    );
    ui.horizontal_wrapped(|ui| {
        if state.running {
            if ui.button("stop").clicked() {
                actions.extend(state.stop());
            }
        } else {
            let can_ask = !state.input.trim().is_empty() && !state.model.is_empty();
            if ui.add_enabled(can_ask, egui::Button::new("ask")).clicked() || (by_key && can_ask) {
                actions.extend(state.ask());
            }
        }
        ui.checkbox(&mut state.share_file, "open file")
            .on_hover_text("Tell Gaia which file is open in the editor.");
        if state.can_primals {
            ui.checkbox(&mut state.use_primals, "primals")
                .on_hover_text(
                    "Let Gaia list the primals running on this device and call their methods. \
                 She asks before every single call. Leave it off for ordinary code work: \
                 a small model works better with fewer tools.",
                );
        }
        if state.can_cursor {
            ui.checkbox(&mut state.use_cursor, "cursor (cloud)")
                .on_hover_text(
                    "Let Gaia ask Cursor's cloud agent. That sends the question, and whatever \
                 its agent reads in this folder, off this device. She asks before every \
                 single question. Off again each time the window opens.",
                );
        }
    });
    ui.add_space(4.0);
}

fn chat_item(ui: &mut egui::Ui, n: usize, item: &Item, clicks: &mut Vec<Click>) {
    match item {
        Item::User(text) => {
            egui::Frame::none()
                .fill(ui.visuals().faint_bg_color)
                .rounding(4.0)
                .inner_margin(6.0)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(text);
                });
        }
        Item::Gaia(text) => prose(ui, text),
        Item::Step(step) => step_row(ui, step, clicks),
        Item::Proposal(proposal) => proposal_box(ui, n, proposal, clicks),
        Item::Note(text) => {
            ui.label(RichText::new(text).small().weak());
        }
        Item::Facts(text) => {
            ui.label(RichText::new(text).small());
        }
        Item::Error(text) => {
            ui.colored_label(ui.visuals().error_fg_color, text);
        }
    }
}

/// What Gaia wrote: prose as prose, fenced code as code.
fn prose(ui: &mut egui::Ui, text: &str) {
    for (is_code, part) in fenced(text) {
        if part.trim().is_empty() {
            continue;
        }
        if is_code {
            egui::Frame::none()
                .fill(ui.visuals().code_bg_color)
                .rounding(3.0)
                .inner_margin(5.0)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(part.trim_end()).monospace());
                });
        } else {
            ui.label(part.trim());
        }
    }
}

/// Split text at its ``` fences: `(is it code, the part)`. A fence left
/// open (Gaia is still writing) counts as code to the end.
pub fn fenced(text: &str) -> Vec<(bool, String)> {
    let mut parts = Vec::new();
    let mut part = String::new();
    let mut in_code = false;
    for line in text.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            parts.push((in_code, std::mem::take(&mut part)));
            in_code = !in_code;
        } else {
            part.push_str(line);
        }
    }
    parts.push((in_code, part));
    parts.retain(|(_, part)| !part.is_empty());
    parts
}

fn step_row(ui: &mut egui::Ui, step: &Step, clicks: &mut Vec<Click>) {
    let is_command = step.name == "run_command";
    let (mark, colour) = match step.state {
        StepState::Running => ("…", ui.visuals().weak_text_color()),
        StepState::Asking => ("?", ui.visuals().warn_fg_color),
        StepState::Done(true) => ("✔", green(ui)),
        StepState::Done(false) => ("✖", ui.visuals().error_fg_color),
    };
    ui.horizontal_top(|ui| {
        ui.label(RichText::new(mark).color(colour).monospace());
        let what = if is_command {
            format!("$ {}", step.summary)
        } else {
            step.summary.clone()
        };
        ui.add(egui::Label::new(fixed(what)).wrap());
    });

    if step.state == StepState::Asking {
        if let Some(warn) = &step.warn {
            ui.colored_label(ui.visuals().warn_fg_color, warn);
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(match step.name.as_str() {
                "run_command" => "Gaia wants to run this command.",
                "primal_call" => "Gaia wants to call this method on this primal.",
                "ask_cursor" => "Gaia wants to ask Cursor's cloud agent.",
                _ => "Gaia wants to do this.",
            });
            let (yes, no) = if is_command {
                ("run it", "don't run")
            } else if step.once {
                ("allow this once", "don't allow")
            } else {
                ("allow", "don't allow")
            };
            let id = step.id;
            if ui.button(yes).clicked() {
                clicks.push(Click::Answer {
                    id,
                    allow: true,
                    always: false,
                });
            }
            if ui.button(no).clicked() {
                clicks.push(Click::Answer {
                    id,
                    allow: false,
                    always: false,
                });
            }
            if step.once {
                ui.weak("Asked every time.");
            } else if ui
                .button("run every command in this chat")
                .on_hover_text(
                    "Gaia will run any command she chooses without asking, until you \
                     press clear or close the window.",
                )
                .clicked()
            {
                clicks.push(Click::Answer {
                    id,
                    allow: true,
                    always: true,
                });
            }
        });
    }

    if let Some(result) = step.result.as_deref().filter(|r| !r.trim().is_empty()) {
        egui::CollapsingHeader::new(RichText::new("what came back").small().weak())
            .id_salt(("ide_step", step.id))
            .show(ui, |ui| {
                let (shown, cut) = clipped(result, RESULT_CHARS);
                ui.label(fixed(shown));
                if cut {
                    ui.weak("… (the rest is not shown here; Gaia got all of it)");
                }
            });
    }
}

/// How many spaces every line that is not blank starts with.
fn shared_indent<'a>(lines: impl Iterator<Item = &'a str>) -> usize {
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start_matches(' ').len())
        .min()
        .unwrap_or(0)
}

/// The first `most` characters of `text`, and whether anything was left off.
fn clipped(text: &str, most: usize) -> (&str, bool) {
    match text.char_indices().nth(most) {
        Some((at, _)) => (&text[..at], true),
        None => (text, false),
    }
}

fn proposal_box(ui: &mut egui::Ui, n: usize, proposal: &Proposal, clicks: &mut Vec<Click>) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        let (title, removed, added) = match &proposal.change {
            Change::Replace { old, fresh } => ("change to", old.as_str(), fresh.as_str()),
            Change::Whole(content) => ("new file", "", content.as_str()),
        };
        ui.label(RichText::new(format!("{title} {}", short_name(&proposal.file))).strong())
            .on_hover_text(&proposal.file);

        let red = ui.visuals().error_fg_color;
        let green = green(ui);
        // The panel is narrow: what every line is indented by is left off.
        let indent = shared_indent(removed.lines().chain(added.lines()));
        let mut shown = 0;
        let mut left_off = 0;
        for (sign, colour, text) in [("-", red, removed), ("+", green, added)] {
            for line in text.lines() {
                if shown < DIFF_LINES {
                    let line = line.get(indent..).unwrap_or("");
                    ui.label(fixed(format!("{sign} {line}")).color(colour));
                    shown += 1;
                } else {
                    left_off += 1;
                }
            }
        }
        if left_off > 0 {
            ui.weak(format!(
                "… {left_off} more lines; all of it shows in the editor once applied"
            ));
        }

        match &proposal.applied {
            None => {
                if ui
                    .button("apply to the editor")
                    .on_hover_text("Puts the change into the editor. Nothing is written to disk until you save.")
                    .clicked()
                {
                    clicks.push(Click::Apply(n));
                }
            }
            Some(Ok(())) => {
                ui.label(
                    RichText::new("In the editor. Not on disk until you save (Ctrl+S).")
                        .small()
                        .color(green),
                );
            }
            Some(Err(why)) => {
                ui.colored_label(red, why);
                if ui.button("try again").clicked() {
                    clicks.push(Click::Apply(n));
                }
            }
        }
    });
}

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
