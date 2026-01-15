//! Tree Primitive Demo
//!
//! Demonstrates the tree primitive with a file system example.
//!
//! Run with: cargo run --example tree_demo

use petal_tongue_primitives::common::Icon;
use petal_tongue_primitives::tree::TreeNode;

fn main() {
    println!("🌸 petalTongue Tree Primitive Demo\n");

    // Build a file system tree (generic - works with String)
    let project = TreeNode::new("my_project/".to_string())
        .with_icon(Icon::Emoji("📁".to_string()))
        .expanded(true)
        .with_child(
            TreeNode::new("src/".to_string())
                .with_icon(Icon::Emoji("📁".to_string()))
                .expanded(true)
                .with_child(
                    TreeNode::new("main.rs".to_string()).with_icon(Icon::Emoji("📄".to_string())),
                )
                .with_child(
                    TreeNode::new("lib.rs".to_string()).with_icon(Icon::Emoji("📄".to_string())),
                )
                .with_child(
                    TreeNode::new("utils/".to_string())
                        .with_icon(Icon::Emoji("📁".to_string()))
                        .with_child(
                            TreeNode::new("helpers.rs".to_string())
                                .with_icon(Icon::Emoji("📄".to_string())),
                        ),
                ),
        )
        .with_child(
            TreeNode::new("tests/".to_string())
                .with_icon(Icon::Emoji("📁".to_string()))
                .with_child(
                    TreeNode::new("integration_tests.rs".to_string())
                        .with_icon(Icon::Emoji("🧪".to_string())),
                ),
        )
        .with_child(
            TreeNode::new("Cargo.toml".to_string()).with_icon(Icon::Emoji("⚙️".to_string())),
        )
        .with_child(
            TreeNode::new("README.md".to_string()).with_icon(Icon::Emoji("📖".to_string())),
        );

    // Display the tree (uses Display trait)
    println!("File System Tree:");
    println!("{}", project);
    println!();

    // Demonstrate tree statistics
    println!("Tree Statistics:");
    println!("  Total nodes: {}", project.count_nodes());
    println!("  Max depth:   {}", project.depth());
    println!();

    // Demonstrate finding
    println!("Finding nodes:");
    if let Some(found) = project.find(|name| name.contains("main.rs")) {
        println!("  Found: {}", found.data());
    }
    println!();

    // Demonstrate filtering (find all Rust files)
    println!("Filtering for .rs files:");
    if let Some(filtered) = project.filter(|name| name.ends_with(".rs")) {
        println!("{}", filtered);
    }

    // Demonstrate visiting
    println!("All files (depth-first):");
    let mut count = 0;
    project.visit(&mut |name| {
        count += 1;
        println!("  {}. {}", count, name);
    });
    println!();

    // Demonstrate mapping (transform to uppercase)
    println!("Transformed tree (uppercase):");
    let uppercase_tree = project.clone().map(|name| name.to_uppercase());
    println!("{}", uppercase_tree);

    println!("✅ Demo complete!");
}
