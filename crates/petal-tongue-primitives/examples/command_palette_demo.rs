//! Command Palette Demo
//!
//! Demonstrates the command palette with fuzzy search.
//!
//! Run with: cargo run --example command_palette_demo

use petal_tongue_primitives::command_palette::{Command, CommandPalette};

fn main() {
    println!("🌸 petalTongue Command Palette Demo\n");

    // Create a command palette with sample commands
    let mut palette = create_sample_palette();

    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  All Commands ({} total)", palette.commands().len());
    println!("═══════════════════════════════════════════════════════════════════════════");
    print_commands(palette.commands());
    println!();

    // Example 1: Search for "file" commands
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Search: 'file'");
    println!("═══════════════════════════════════════════════════════════════════════════");
    let results = palette.search("file");
    print_results(&results);
    println!();

    // Example 2: Search for "copy"
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Search: 'copy'");
    println!("═══════════════════════════════════════════════════════════════════════════");
    let results = palette.search("copy");
    print_results(&results);
    println!();

    // Example 3: Fuzzy search
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Search: 'sav' (fuzzy)");
    println!("═══════════════════════════════════════════════════════════════════════════");
    let results = palette.search("sav");
    print_results(&results);
    println!();

    // Example 4: Search by category
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Commands by Category");
    println!("═══════════════════════════════════════════════════════════════════════════");
    for category in palette.categories() {
        let commands = palette.commands_by_category(&category);
        println!("\n{}:", category);
        for cmd in commands {
            print_command(cmd);
        }
    }
    println!();

    // Example 5: Navigation
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Navigation Example");
    println!("═══════════════════════════════════════════════════════════════════════════");
    palette.set_query("file");

    println!("Query: '{}'\n", palette.query());

    for i in 0..3 {
        if let Some(cmd) = palette.selected_command() {
            println!(
                "  [{}] Selected: {} (score: {:.2})",
                i,
                cmd.name,
                palette
                    .results()
                    .get(palette.selected_index())
                    .map(|r| r.score)
                    .unwrap_or(0.0)
            );
        }
        palette.select_next();
    }
    println!();

    // Example 6: Keybindings
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Commands with Keybindings");
    println!("═══════════════════════════════════════════════════════════════════════════");
    for cmd in palette.commands() {
        if let Some(ref kb) = cmd.keybinding {
            println!("  {:20} → {}", cmd.name, kb);
        }
    }
    println!();

    println!("✅ Demo complete!");
}

fn create_sample_palette() -> CommandPalette<String> {
    CommandPalette::new()
        // File commands
        .with_command(
            Command::new("file.new", "New File", "File", "new_file".to_string())
                .with_description("Create a new file")
                .with_keybinding("Ctrl+N")
                .with_icon("📄"),
        )
        .with_command(
            Command::new("file.open", "Open File", "File", "open_file".to_string())
                .with_description("Open an existing file")
                .with_keybinding("Ctrl+O")
                .with_icon("📂"),
        )
        .with_command(
            Command::new("file.save", "Save File", "File", "save_file".to_string())
                .with_description("Save the current file")
                .with_keybinding("Ctrl+S")
                .with_icon("💾"),
        )
        .with_command(
            Command::new("file.save_as", "Save As...", "File", "save_as".to_string())
                .with_description("Save file with a new name")
                .with_keybinding("Ctrl+Shift+S")
                .with_icon("💾"),
        )
        .with_command(
            Command::new("file.close", "Close File", "File", "close_file".to_string())
                .with_description("Close the current file")
                .with_keybinding("Ctrl+W")
                .with_icon("❌"),
        )
        // Edit commands
        .with_command(
            Command::new("edit.undo", "Undo", "Edit", "undo".to_string())
                .with_description("Undo last action")
                .with_keybinding("Ctrl+Z")
                .with_icon("↶"),
        )
        .with_command(
            Command::new("edit.redo", "Redo", "Edit", "redo".to_string())
                .with_description("Redo last undone action")
                .with_keybinding("Ctrl+Y")
                .with_icon("↷"),
        )
        .with_command(
            Command::new("edit.copy", "Copy", "Edit", "copy".to_string())
                .with_description("Copy selection to clipboard")
                .with_keybinding("Ctrl+C")
                .with_icon("📋"),
        )
        .with_command(
            Command::new("edit.paste", "Paste", "Edit", "paste".to_string())
                .with_description("Paste from clipboard")
                .with_keybinding("Ctrl+V")
                .with_icon("📋"),
        )
        .with_command(
            Command::new("edit.find", "Find", "Edit", "find".to_string())
                .with_description("Find text in file")
                .with_keybinding("Ctrl+F")
                .with_icon("🔍"),
        )
        // View commands
        .with_command(
            Command::new("view.zoom_in", "Zoom In", "View", "zoom_in".to_string())
                .with_description("Increase zoom level")
                .with_keybinding("Ctrl++")
                .with_icon("🔍"),
        )
        .with_command(
            Command::new("view.zoom_out", "Zoom Out", "View", "zoom_out".to_string())
                .with_description("Decrease zoom level")
                .with_keybinding("Ctrl+-")
                .with_icon("🔍"),
        )
        .with_command(
            Command::new(
                "view.toggle_sidebar",
                "Toggle Sidebar",
                "View",
                "toggle_sidebar".to_string(),
            )
            .with_description("Show/hide the sidebar")
            .with_keybinding("Ctrl+B")
            .with_icon("📑"),
        )
}

fn print_commands(commands: &[Command<String>]) {
    for cmd in commands {
        print_command(cmd);
    }
}

fn print_command(cmd: &Command<String>) {
    let icon = cmd.icon.as_ref().map(|s| s.as_str()).unwrap_or(" ");
    let kb = cmd
        .keybinding
        .as_ref()
        .map(|s| format!(" [{}]", s))
        .unwrap_or_default();
    println!(
        "  {} {:30} {} ({}){}",
        icon, cmd.name, cmd.id, cmd.category, kb
    );
}

fn print_results(results: &[petal_tongue_primitives::command_palette::SearchResult<String>]) {
    if results.is_empty() {
        println!("  No results found.");
        return;
    }

    println!("  Found {} result(s):\n", results.len());
    for (i, result) in results.iter().enumerate() {
        let icon = result
            .command
            .icon
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(" ");
        let kb = result
            .command
            .keybinding
            .as_ref()
            .map(|s| format!(" [{}]", s))
            .unwrap_or_default();
        println!(
            "  {}. {} {:30} (score: {:.2}){}",
            i + 1,
            icon,
            result.command.name,
            result.score,
            kb
        );
    }
}
