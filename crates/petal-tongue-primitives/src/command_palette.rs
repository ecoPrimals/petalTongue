// SPDX-License-Identifier: AGPL-3.0-only
//! # Command Palette Primitive
//!
//! Universal command/action system with fuzzy search and keybindings.
//!
//! ## Philosophy
//!
//! - **Universal Access**: Single interface for all commands
//! - **Fuzzy Search**: Fast, forgiving command discovery
//! - **Keybindings**: Optional keyboard shortcuts
//! - **Categories**: Organize commands by category
//! - **Generic Actions**: Commands execute ANY action type
//! - **Safe**: 100% safe Rust, no unsafe code
//!
//! ## Example
//!
//! ```rust,ignore
//! use petal_tongue_primitives::command_palette::{Command, CommandPalette};
//!
//! let mut palette = CommandPalette::new()
//!     .with_command(Command::new(
//!         "file.open",
//!         "Open File",
//!         "Files",
//!         || { /* action */ },
//!     ).with_keybinding("Ctrl+O"))
//!     .with_command(Command::new(
//!         "file.save",
//!         "Save File",
//!         "Files",
//!         || { /* action */ },
//!     ).with_keybinding("Ctrl+S"));
//!
//! // Search for commands
//! let results = palette.search("save");
//! ```

use serde::{Deserialize, Serialize};

/// Command ID (unique identifier)
pub type CommandId = String;

/// Command category
pub type Category = String;

/// Keybinding (e.g., "Ctrl+S", "Alt+F4")
pub type Keybinding = String;

/// A command in the palette
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Command<T> {
    /// Unique command ID
    pub id: CommandId,

    /// Display name
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// Category (for grouping)
    pub category: Category,

    /// Optional keybinding
    pub keybinding: Option<Keybinding>,

    /// Command action/data
    pub action: T,

    /// Whether command is enabled
    pub enabled: bool,

    /// Icon (optional)
    pub icon: Option<String>,
}

impl<T> Command<T> {
    /// Create a new command
    pub fn new(
        id: impl Into<CommandId>,
        name: impl Into<String>,
        category: impl Into<Category>,
        action: T,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            category: category.into(),
            keybinding: None,
            action,
            enabled: true,
            icon: None,
        }
    }

    /// Set description
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set keybinding
    #[must_use]
    pub fn with_keybinding(mut self, keybinding: impl Into<Keybinding>) -> Self {
        self.keybinding = Some(keybinding.into());
        self
    }

    /// Set icon
    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set enabled state
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Search result with relevance score
#[derive(Debug, Clone)]
pub struct SearchResult<T> {
    /// The command
    pub command: Command<T>,

    /// Relevance score (0.0 - 1.0, higher is better)
    pub score: f32,
}

/// Command palette
///
/// Manages commands and provides search functionality.
pub struct CommandPalette<T> {
    /// All commands
    commands: Vec<Command<T>>,

    /// Current search query
    search_query: String,

    /// Selected result index
    selected_index: usize,
}

impl<T: Clone> CommandPalette<T> {
    /// Create a new command palette
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            search_query: String::new(),
            selected_index: 0,
        }
    }

    /// Add a command (builder pattern)
    #[must_use]
    pub fn with_command(mut self, command: Command<T>) -> Self {
        self.commands.push(command);
        self
    }

    /// Add multiple commands (builder pattern)
    #[must_use]
    pub fn with_commands(mut self, commands: Vec<Command<T>>) -> Self {
        self.commands.extend(commands);
        self
    }

    /// Add a command
    pub fn add_command(&mut self, command: Command<T>) {
        self.commands.push(command);
    }

    /// Get all commands
    #[must_use]
    pub fn commands(&self) -> &[Command<T>] {
        &self.commands
    }

    /// Get command by ID
    #[must_use]
    pub fn get_command(&self, id: &str) -> Option<&Command<T>> {
        self.commands.iter().find(|c| c.id == id)
    }

    /// Remove command by ID
    pub fn remove_command(&mut self, id: &str) -> Option<Command<T>> {
        if let Some(pos) = self.commands.iter().position(|c| c.id == id) {
            Some(self.commands.remove(pos))
        } else {
            None
        }
    }

    /// Get commands by category
    #[must_use]
    pub fn commands_by_category(&self, category: &str) -> Vec<&Command<T>> {
        self.commands
            .iter()
            .filter(|c| c.category == category)
            .collect()
    }

    /// Get all categories
    #[must_use]
    pub fn categories(&self) -> Vec<Category> {
        let mut cats: Vec<_> = self.commands.iter().map(|c| c.category.clone()).collect();
        cats.sort();
        cats.dedup();
        cats
    }

    /// Search commands (fuzzy matching)
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<SearchResult<T>> {
        if query.is_empty() {
            // Return all enabled commands
            return self
                .commands
                .iter()
                .filter(|c| c.enabled)
                .map(|c| SearchResult {
                    command: c.clone(),
                    score: 1.0,
                })
                .collect();
        }

        let query_lower = query.to_lowercase();
        let mut results: Vec<SearchResult<T>> = self
            .commands
            .iter()
            .filter(|c| c.enabled)
            .filter_map(|c| {
                let score = self.calculate_match_score(&query_lower, c);
                if score > 0.0 {
                    Some(SearchResult {
                        command: c.clone(),
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        results
    }

    /// Calculate match score for a command
    fn calculate_match_score(&self, query: &str, command: &Command<T>) -> f32 {
        let name_lower = command.name.to_lowercase();
        let id_lower = command.id.to_lowercase();

        // Exact match (highest score)
        if name_lower == query || id_lower == query {
            return 1.0;
        }

        // Starts with query (high score)
        if name_lower.starts_with(query) {
            return 0.9;
        }

        if id_lower.starts_with(query) {
            return 0.85;
        }

        // Contains query (medium score)
        if name_lower.contains(query) {
            return 0.7;
        }

        if id_lower.contains(query) {
            return 0.65;
        }

        // Check description
        if let Some(ref desc) = command.description {
            let desc_lower = desc.to_lowercase();
            if desc_lower.contains(query) {
                return 0.5;
            }
        }

        // Fuzzy match (basic - could be improved)
        let fuzzy_score = self.fuzzy_match(query, &name_lower);
        if fuzzy_score > 0.3 {
            return fuzzy_score;
        }

        0.0 // No match
    }

    /// Basic fuzzy matching (character sequence)
    fn fuzzy_match(&self, query: &str, text: &str) -> f32 {
        let mut query_chars = query.chars();
        let mut current_char = query_chars.next();
        let mut matches = 0;

        if current_char.is_none() {
            return 0.0;
        }

        for c in text.chars() {
            if Some(c) == current_char {
                matches += 1;
                current_char = query_chars.next();
                if current_char.is_none() {
                    break;
                }
            }
        }

        if current_char.is_none() {
            // All query chars matched
            matches as f32 / text.len() as f32
        } else {
            0.0
        }
    }

    /// Set search query
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.search_query = query.into();
        self.selected_index = 0;
    }

    /// Get current search query
    #[must_use]
    pub fn query(&self) -> &str {
        &self.search_query
    }

    /// Get search results for current query
    #[must_use]
    pub fn results(&self) -> Vec<SearchResult<T>> {
        self.search(&self.search_query)
    }

    /// Get selected index
    #[must_use]
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Select next result
    pub fn select_next(&mut self) {
        let results_count = self.results().len();
        if results_count > 0 {
            self.selected_index = (self.selected_index + 1) % results_count;
        }
    }

    /// Select previous result
    pub fn select_previous(&mut self) {
        let results_count = self.results().len();
        if results_count > 0 {
            if self.selected_index == 0 {
                self.selected_index = results_count - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Get selected command (if any)
    #[must_use]
    pub fn selected_command(&self) -> Option<Command<T>> {
        self.results()
            .get(self.selected_index)
            .map(|r| r.command.clone())
    }

    /// Clear search
    pub fn clear(&mut self) {
        self.search_query.clear();
        self.selected_index = 0;
    }
}

impl<T: Clone> Default for CommandPalette<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_commands() -> Vec<Command<String>> {
        vec![
            Command::new("file.open", "Open File", "File", "open".to_string())
                .with_keybinding("Ctrl+O"),
            Command::new("file.save", "Save File", "File", "save".to_string())
                .with_keybinding("Ctrl+S"),
            Command::new("file.close", "Close File", "File", "close".to_string())
                .with_keybinding("Ctrl+W"),
            Command::new("edit.copy", "Copy", "Edit", "copy".to_string()).with_keybinding("Ctrl+C"),
            Command::new("edit.paste", "Paste", "Edit", "paste".to_string())
                .with_keybinding("Ctrl+V"),
            Command::new("view.zoom_in", "Zoom In", "View", "zoom_in".to_string())
                .with_keybinding("Ctrl++"),
        ]
    }

    #[test]
    fn test_command_creation() {
        let cmd = Command::new("test", "Test Command", "Testing", "action".to_string())
            .with_description("A test command")
            .with_keybinding("Ctrl+T")
            .with_icon("🧪");

        assert_eq!(cmd.id, "test");
        assert_eq!(cmd.name, "Test Command");
        assert_eq!(cmd.category, "Testing");
        assert_eq!(cmd.description, Some("A test command".to_string()));
        assert_eq!(cmd.keybinding, Some("Ctrl+T".to_string()));
        assert_eq!(cmd.icon, Some("🧪".to_string()));
        assert!(cmd.enabled);
    }

    #[test]
    fn test_palette_creation() {
        let palette: CommandPalette<String> = CommandPalette::new();
        assert_eq!(palette.commands().len(), 0);
    }

    #[test]
    fn test_add_commands() {
        let palette = CommandPalette::new().with_commands(sample_commands());

        assert_eq!(palette.commands().len(), 6);
    }

    #[test]
    fn test_get_command() {
        let palette = CommandPalette::new().with_commands(sample_commands());

        assert!(palette.get_command("file.open").is_some());
        assert!(palette.get_command("nonexistent").is_none());
    }

    #[test]
    fn test_remove_command() {
        let mut palette = CommandPalette::new().with_commands(sample_commands());

        assert_eq!(palette.commands().len(), 6);

        let removed = palette.remove_command("file.open");
        assert!(removed.is_some());
        assert_eq!(palette.commands().len(), 5);
        assert!(palette.get_command("file.open").is_none());
    }

    #[test]
    fn test_categories() {
        let palette = CommandPalette::new().with_commands(sample_commands());

        let categories = palette.categories();
        assert_eq!(categories, vec!["Edit", "File", "View"]);
    }

    #[test]
    fn test_commands_by_category() {
        let palette = CommandPalette::new().with_commands(sample_commands());

        let file_commands = palette.commands_by_category("File");
        assert_eq!(file_commands.len(), 3);
    }

    #[test]
    fn test_search_empty_query() {
        let palette = CommandPalette::new().with_commands(sample_commands());

        let results = palette.search("");
        assert_eq!(results.len(), 6); // All commands
    }

    #[test]
    fn test_search_exact_match() {
        let palette = CommandPalette::new().with_commands(sample_commands());

        let results = palette.search("Save File");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].command.id, "file.save");
        assert_eq!(results[0].score, 1.0);
    }

    #[test]
    fn test_search_partial_match() {
        let palette = CommandPalette::new().with_commands(sample_commands());

        let results = palette.search("file");
        assert!(results.len() >= 3); // At least the 3 File commands
    }

    #[test]
    fn test_search_fuzzy() {
        let palette = CommandPalette::new().with_commands(sample_commands());

        let results = palette.search("sav");
        assert!(!results.is_empty());
        // Should find "Save File"
        assert!(results.iter().any(|r| r.command.id == "file.save"));
    }

    #[test]
    fn test_search_scoring() {
        let palette = CommandPalette::new().with_commands(sample_commands());

        let results = palette.search("copy");
        assert!(!results.is_empty());

        // Exact/contains match should score higher than fuzzy
        let top_result = &results[0];
        assert_eq!(top_result.command.id, "edit.copy");
        assert!(top_result.score > 0.5);
    }

    #[test]
    fn test_selection_navigation() {
        let mut palette = CommandPalette::new().with_commands(sample_commands());

        palette.set_query("file");

        assert_eq!(palette.selected_index(), 0);

        palette.select_next();
        assert_eq!(palette.selected_index(), 1);

        palette.select_previous();
        assert_eq!(palette.selected_index(), 0);
    }

    #[test]
    fn test_selected_command() {
        let mut palette = CommandPalette::new().with_commands(sample_commands());

        palette.set_query("save");

        let selected = palette.selected_command();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "file.save");
    }

    #[test]
    fn test_clear() {
        let mut palette = CommandPalette::new().with_commands(sample_commands());

        palette.set_query("test");
        assert_eq!(palette.query(), "test");

        palette.clear();
        assert_eq!(palette.query(), "");
        assert_eq!(palette.selected_index(), 0);
    }

    #[test]
    fn test_disabled_commands() {
        let mut commands = sample_commands();
        commands[0] = commands[0].clone().enabled(false);

        let palette = CommandPalette::new().with_commands(commands);

        let results = palette.search("open");
        // Disabled command should not appear
        assert!(!results.iter().any(|r| r.command.id == "file.open"));
    }
}
