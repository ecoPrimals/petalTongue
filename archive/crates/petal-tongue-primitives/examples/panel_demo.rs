// SPDX-License-Identifier: AGPL-3.0-only
//! Panel Layout Demo
//!
//! Demonstrates the panel layout system with splits and tabs.
//!
//! Run with: cargo run --example panel_demo

use petal_tongue_primitives::panel::{Direction, Panel, PanelContent, PanelLayout};

fn main() {
    println!("🌸 petalTongue Panel Layout Demo\n");

    // Example 1: Simple horizontal split
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Example 1: Horizontal Split (Editor | File Tree)");
    println!("═══════════════════════════════════════════════════════════════════════════");

    let simple_split = Panel::split(
        Direction::Horizontal,
        0.7, // 70% for editor
        Panel::leaf(
            "editor",
            "Editor",
            "fn main() {\n    println!(\"Hello!\");\n}",
        ),
        Panel::leaf("tree", "File Tree", "src/\n  main.rs\n  lib.rs\nCargo.toml"),
    );

    print_panel_structure(&simple_split, 0);
    println!();

    // Example 2: Nested splits (IDE-like layout)
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Example 2: Nested Splits (IDE Layout)");
    println!("═══════════════════════════════════════════════════════════════════════════");

    let ide_layout = Panel::split(
        Direction::Horizontal,
        0.2, // 20% for sidebar
        Panel::leaf("sidebar", "Sidebar", "Explorer\nSearch\nGit"),
        Panel::split(
            Direction::Vertical,
            0.7, // 70% for editor area
            Panel::leaf("editor", "main.rs", "fn main() {...}"),
            Panel::split(
                Direction::Horizontal,
                0.5,
                Panel::leaf("terminal", "Terminal", "$ cargo run"),
                Panel::leaf("output", "Output", "Compiling..."),
            ),
        ),
    );

    print_panel_structure(&ide_layout, 0);
    println!();

    // Example 3: Tab group
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Example 3: Tab Group");
    println!("═══════════════════════════════════════════════════════════════════════════");

    let tabs = Panel::tabs(
        vec![
            PanelContent::new("tab1", "main.rs", "fn main() {...}"),
            PanelContent::new("tab2", "lib.rs", "pub fn hello() {...}"),
            PanelContent::new("tab3", "tests.rs", "#[test]\nfn test() {...}"),
        ],
        0, // Active tab
    );

    print_panel_structure(&tabs, 0);
    println!();

    // Example 4: Complex layout with tabs
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Example 4: Complex Layout (Tabs + Splits)");
    println!("═══════════════════════════════════════════════════════════════════════════");

    let complex = Panel::split(
        Direction::Horizontal,
        0.75,
        Panel::tabs(
            vec![
                PanelContent::new("file1", "main.rs", "Code..."),
                PanelContent::new("file2", "lib.rs", "Code..."),
            ],
            0,
        ),
        Panel::split(
            Direction::Vertical,
            0.5,
            Panel::leaf("tree", "Files", "Project tree..."),
            Panel::leaf("outline", "Outline", "Symbols..."),
        ),
    );

    print_panel_structure(&complex, 0);
    println!();

    // Example 5: Panel operations
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Example 5: Panel Operations");
    println!("═══════════================================================================");

    let mut layout = PanelLayout::new(ide_layout.clone());

    println!("Total panels: {}", layout.count_panels());
    println!("Panel IDs: {:?}", layout.panel_ids());
    println!();

    // Focus a panel
    layout.focus_panel("editor");
    println!("Focused panel: {:?}", layout.focused_id());
    println!();

    // Find a specific panel
    if let Some(panel) = layout.find_panel("terminal") {
        println!("Found panel '{}': {}", panel.id, panel.title);
        println!(
            "Content preview: {}",
            panel.content.chars().take(20).collect::<String>()
        );
    }
    println!();

    // Example 6: Panel mapping (transform content)
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  Example 6: Panel Mapping (Transform Content)");
    println!("═══════════════════════════════════════════════════════════════════════════");

    let numeric_layout = Panel::split(
        Direction::Horizontal,
        0.5,
        Panel::leaf("left", "Left", 100),
        Panel::leaf("right", "Right", 200),
    );

    // Map numbers to strings
    let string_layout = numeric_layout.map(&|n: i32| format!("Value: {n}"));

    if let Some(left) = string_layout.find_panel("left") {
        println!("Left panel content: {}", left.content);
    }
    if let Some(right) = string_layout.find_panel("right") {
        println!("Right panel content: {}", right.content);
    }
    println!();

    println!("✅ Demo complete!");
}

fn print_panel_structure<T: std::fmt::Display>(panel: &Panel<T>, indent: usize) {
    let prefix = "  ".repeat(indent);

    match panel {
        Panel::Leaf(content) => {
            println!("{}├─ Leaf: {} (\"{}\")", prefix, content.id, content.title);
            let preview: String = content.content.to_string().chars().take(40).collect();
            println!("{prefix}   Content: {preview}...");
        }
        Panel::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            let dir = match direction {
                Direction::Horizontal => "Horizontal",
                Direction::Vertical => "Vertical",
            };
            println!("{}├─ Split: {} (ratio: {:.1}%)", prefix, dir, ratio * 100.0);
            println!("{}   First ({:.0}%):", prefix, ratio * 100.0);
            print_panel_structure(first, indent + 2);
            println!("{}   Second ({:.0}%):", prefix, (1.0 - ratio) * 100.0);
            print_panel_structure(second, indent + 2);
        }
        Panel::Tabs {
            panels,
            active_index,
        } => {
            println!(
                "{}├─ Tabs: {} tabs (active: {})",
                prefix,
                panels.len(),
                active_index
            );
            for (i, panel) in panels.iter().enumerate() {
                let marker = if i == *active_index { "●" } else { "○" };
                println!(
                    "{}   {} Tab {}: {} (\"{}\")",
                    prefix, marker, i, panel.id, panel.title
                );
            }
        }
    }
}
