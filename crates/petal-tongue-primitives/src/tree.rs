// SPDX-License-Identifier: AGPL-3.0-only
//! # Tree Primitive
//!
//! Render hierarchical data in a tree structure.
//!
//! ## Philosophy
//!
//! - **Generic**: Works with ANY data type via `TreeNode<T>`
//! - **Zero Hardcoding**: No assumptions about data structure
//! - **Capability-Based**: Renderer adapts to available modalities
//! - **Performance**: Lazy rendering, virtualization for large trees
//!
//! ## Use Cases
//!
//! - File browsers
//! - Category navigation
//! - Org charts
//! - Menu systems
//! - Primal topology
//! - API hierarchies
//!
//! ## Example
//!
//! ```rust
//! use petal_tongue_primitives::tree::TreeNode;
//!
//! // Build a file system tree
//! let root = TreeNode::new("project")
//!     .with_child(TreeNode::new("src")
//!         .with_child(TreeNode::new("main.rs"))
//!         .with_child(TreeNode::new("lib.rs"))
//!     )
//!     .with_child(TreeNode::new("Cargo.toml"));
//!
//! // Tree is now ready to render
//! assert_eq!(root.data(), &"project");
//! assert_eq!(root.children().len(), 2);
//! ```

use crate::common::{Color, Icon};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A node in a tree structure
///
/// Generic over data type `T` - works with ANY data.
/// Zero hardcoding - data structure is completely generic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreeNode<T> {
    /// The data contained in this node
    data: T,

    /// Child nodes
    children: Vec<TreeNode<T>>,

    /// Whether this node is expanded (showing children)
    expanded: bool,

    /// Optional icon for this node
    icon: Option<Icon>,

    /// Optional color for this node
    color: Option<Color>,

    /// Whether this node is selectable
    selectable: bool,

    /// Whether this node is visible (for filtering)
    visible: bool,
}

impl<T> TreeNode<T> {
    /// Create a new tree node with data
    ///
    /// # Example
    ///
    /// ```rust
    /// use petal_tongue_primitives::tree::TreeNode;
    ///
    /// let node = TreeNode::new("my_file.rs");
    /// assert_eq!(node.data(), &"my_file.rs");
    /// ```
    pub fn new(data: T) -> Self {
        Self {
            data,
            children: Vec::new(),
            expanded: false,
            icon: None,
            color: None,
            selectable: true,
            visible: true,
        }
    }

    /// Add a child node (builder pattern)
    ///
    /// # Example
    ///
    /// ```rust
    /// use petal_tongue_primitives::tree::TreeNode;
    ///
    /// let parent = TreeNode::new("parent")
    ///     .with_child(TreeNode::new("child1"))
    ///     .with_child(TreeNode::new("child2"));
    ///
    /// assert_eq!(parent.children().len(), 2);
    /// ```
    #[must_use]
    pub fn with_child(mut self, child: TreeNode<T>) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children (builder pattern)
    #[must_use]
    pub fn with_children(mut self, children: Vec<TreeNode<T>>) -> Self {
        self.children.extend(children);
        self
    }

    /// Set icon (builder pattern)
    #[must_use]
    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set color (builder pattern)
    #[must_use]
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set expanded state (builder pattern)
    #[must_use]
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Set selectable (builder pattern)
    #[must_use]
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Get reference to node data
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Get mutable reference to node data
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Get reference to children
    pub fn children(&self) -> &[TreeNode<T>] {
        &self.children
    }

    /// Get mutable reference to children
    pub fn children_mut(&mut self) -> &mut Vec<TreeNode<T>> {
        &mut self.children
    }

    /// Check if node is expanded
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Expand this node
    pub fn expand(&mut self) {
        self.expanded = true;
    }

    /// Collapse this node
    pub fn collapse(&mut self) {
        self.expanded = false;
    }

    /// Toggle expansion
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }

    /// Get icon
    pub fn icon(&self) -> Option<&Icon> {
        self.icon.as_ref()
    }

    /// Get color
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    /// Check if node is selectable
    pub fn is_selectable(&self) -> bool {
        self.selectable
    }

    /// Check if node is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set visibility (for filtering)
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Check if node has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Get depth of tree (recursive)
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(Self::depth).max().unwrap_or(0)
        }
    }

    /// Count total nodes in tree (including this one)
    pub fn count_nodes(&self) -> usize {
        1 + self.children.iter().map(Self::count_nodes).sum::<usize>()
    }

    /// Find first node matching predicate (depth-first search)
    pub fn find<F>(&self, predicate: F) -> Option<&TreeNode<T>>
    where
        F: Fn(&T) -> bool + Copy,
    {
        if predicate(&self.data) {
            return Some(self);
        }

        for child in &self.children {
            if let Some(found) = child.find(predicate) {
                return Some(found);
            }
        }

        None
    }

    /// Find mutable reference to first node matching predicate
    pub fn find_mut<F>(&mut self, predicate: F) -> Option<&mut TreeNode<T>>
    where
        F: Fn(&T) -> bool + Copy,
    {
        if predicate(&self.data) {
            return Some(self);
        }

        for child in &mut self.children {
            if let Some(found) = child.find_mut(predicate) {
                return Some(found);
            }
        }

        None
    }

    /// Filter tree by predicate (creates new tree with matching nodes)
    pub fn filter<F>(&self, predicate: F) -> Option<TreeNode<T>>
    where
        F: Fn(&T) -> bool + Copy,
        T: Clone,
    {
        let matches = predicate(&self.data);
        let filtered_children: Vec<_> = self
            .children
            .iter()
            .filter_map(|child| child.filter(predicate))
            .collect();

        if matches || !filtered_children.is_empty() {
            let mut node = self.clone();
            node.children = filtered_children;
            node.visible = matches;
            Some(node)
        } else {
            None
        }
    }

    /// Map tree to new type (preserves structure)
    pub fn map<U, F>(self, f: F) -> TreeNode<U>
    where
        F: Fn(T) -> U + Copy,
    {
        TreeNode {
            data: f(self.data),
            children: self.children.into_iter().map(|c| c.map(f)).collect(),
            expanded: self.expanded,
            icon: self.icon,
            color: self.color,
            selectable: self.selectable,
            visible: self.visible,
        }
    }

    /// Visit all nodes (depth-first, pre-order)
    pub fn visit<F>(&self, visitor: &mut F)
    where
        F: FnMut(&T),
    {
        visitor(&self.data);
        for child in &self.children {
            child.visit(visitor);
        }
    }

    /// Visit all nodes with mutable access
    pub fn visit_mut<F>(&mut self, visitor: &mut F)
    where
        F: FnMut(&mut T),
    {
        visitor(&mut self.data);
        for child in &mut self.children {
            child.visit_mut(visitor);
        }
    }
}

impl<T: fmt::Display> fmt::Display for TreeNode<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn write_tree<T: fmt::Display>(
            f: &mut fmt::Formatter<'_>,
            node: &TreeNode<T>,
            prefix: &str,
            is_last: bool,
        ) -> fmt::Result {
            // Draw the branch
            write!(f, "{prefix}")?;
            write!(f, "{}", if is_last { "└── " } else { "├── " })?;

            // Draw icon if present
            if let Some(icon) = &node.icon {
                match icon {
                    Icon::Emoji(emoji) => write!(f, "{emoji} ")?,
                    Icon::NerdFont(icon) => write!(f, "{icon} ")?,
                    Icon::Custom(icon) => write!(f, "{icon} ")?,
                    Icon::None => {}
                }
            }

            // Draw the node data
            writeln!(f, "{}", node.data)?;

            // Draw children if expanded
            if node.expanded && !node.children.is_empty() {
                let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

                for (i, child) in node.children.iter().enumerate() {
                    let is_last_child = i == node.children.len() - 1;
                    write_tree(f, child, &child_prefix, is_last_child)?;
                }
            }

            Ok(())
        }

        write_tree(f, self, "", true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_creation() {
        let node = TreeNode::new("test");
        assert_eq!(node.data(), &"test");
        assert_eq!(node.children().len(), 0);
        assert!(!node.is_expanded());
        assert!(node.is_selectable());
        assert!(node.is_visible());
    }

    #[test]
    fn test_tree_with_children() {
        let tree = TreeNode::new("root")
            .with_child(TreeNode::new("child1"))
            .with_child(TreeNode::new("child2"));

        assert_eq!(tree.children().len(), 2);
        assert_eq!(tree.children()[0].data(), &"child1");
        assert_eq!(tree.children()[1].data(), &"child2");
    }

    #[test]
    fn test_tree_depth() {
        let tree = TreeNode::new("root")
            .with_child(TreeNode::new("child1").with_child(TreeNode::new("grandchild1")))
            .with_child(TreeNode::new("child2"));

        assert_eq!(tree.depth(), 3);
    }

    #[test]
    fn test_tree_count_nodes() {
        let tree = TreeNode::new("root")
            .with_child(TreeNode::new("child1").with_child(TreeNode::new("grandchild1")))
            .with_child(TreeNode::new("child2"));

        assert_eq!(tree.count_nodes(), 4);
    }

    #[test]
    fn test_tree_find() {
        let tree = TreeNode::new("root")
            .with_child(TreeNode::new("folder").with_child(TreeNode::new("file.rs")));

        let found = tree.find(|data| data == &"file.rs");
        assert!(found.is_some());
        assert_eq!(found.unwrap().data(), &"file.rs");

        let not_found = tree.find(|data| data == &"missing");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_tree_filter() {
        let tree = TreeNode::new("root")
            .with_child(TreeNode::new("file1.rs"))
            .with_child(TreeNode::new("file2.txt"))
            .with_child(TreeNode::new("file3.rs"));

        let filtered = tree.filter(|data| data.ends_with(".rs"));
        assert!(filtered.is_some());

        let filtered_tree = filtered.unwrap();
        assert_eq!(filtered_tree.children().len(), 2);
    }

    #[test]
    fn test_tree_map() {
        let tree = TreeNode::new(1)
            .with_child(TreeNode::new(2))
            .with_child(TreeNode::new(3));

        let mapped = tree.map(|n| n * 2);
        assert_eq!(mapped.data(), &2);
        assert_eq!(mapped.children()[0].data(), &4);
        assert_eq!(mapped.children()[1].data(), &6);
    }

    #[test]
    fn test_tree_visit() {
        let tree = TreeNode::new(1)
            .with_child(TreeNode::new(2))
            .with_child(TreeNode::new(3));

        let mut sum = 0;
        tree.visit(&mut |n| sum += n);
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_tree_display() {
        let tree = TreeNode::new("root")
            .expanded(true)
            .with_child(TreeNode::new("child1"))
            .with_child(TreeNode::new("child2"));

        let output = format!("{tree}");
        assert!(output.contains("root"));
        assert!(output.contains("child1"));
        assert!(output.contains("child2"));
    }

    #[test]
    fn test_tree_with_icons() {
        let tree = TreeNode::new("folder")
            .with_icon(Icon::Emoji("📁".to_string()))
            .with_child(TreeNode::new("file.rs").with_icon(Icon::Emoji("📄".to_string())));

        assert!(tree.icon().is_some());
        assert!(tree.children()[0].icon().is_some());
    }

    #[test]
    fn test_tree_with_colors() {
        let tree = TreeNode::new("important").with_color(Color::RED);

        assert_eq!(tree.color(), Some(Color::RED));
    }
}
