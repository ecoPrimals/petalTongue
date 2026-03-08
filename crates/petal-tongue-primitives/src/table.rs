// SPDX-License-Identifier: AGPL-3.0-only
//! # Table Primitive
//!
//! Generic table data structure with sorting, filtering, and pagination.
//!
//! ## Philosophy
//!
//! - **Generic**: `Table<T>` works with ANY data type
//! - **Flexible Columns**: Runtime column definition via closures
//! - **Sortable**: Multi-column sorting with custom comparators
//! - **Filterable**: Predicate-based row filtering
//! - **Paginated**: Built-in pagination support
//! - **Safe**: 100% safe Rust, no unsafe code
//!
//! ## Example
//!
//! ```rust
//! use petal_tongue_primitives::table::{Table, Column};
//!
//! #[derive(Clone)]
//! struct Person {
//!     name: String,
//!     age: u32,
//!     email: String,
//! }
//!
//! let mut table = Table::new()
//!     .with_column(Column::new("Name", |p: &Person| p.name.clone()))
//!     .with_column(Column::new("Age", |p: &Person| p.age.to_string()))
//!     .with_column(Column::new("Email", |p: &Person| p.email.clone()))
//!     .with_data(vec![
//!         Person { name: "Alice".into(), age: 30, email: "alice@example.com".into() },
//!         Person { name: "Bob".into(), age: 25, email: "bob@example.com".into() },
//!     ]);
//!
//! // Sort by age
//! table.sort_by(|a, b| a.age.cmp(&b.age));
//!
//! // Filter for age > 28
//! let filtered = table.filter(|p| p.age > 28);
//! ```

use std::cmp::Ordering;

/// A column in a table
///
/// Columns are defined by a name and a function that extracts
/// a string representation from the row data.
pub struct Column<T> {
    /// Column header name
    name: String,

    /// Function to extract cell value from row data
    /// Returns String for universal rendering
    extractor: Box<dyn Fn(&T) -> String + Send + Sync>,

    /// Column width (None = auto)
    width: Option<usize>,

    /// Is column sortable?
    sortable: bool,

    /// Is column visible?
    visible: bool,
}

impl<T> Column<T> {
    /// Create a new column
    pub fn new<F>(name: impl Into<String>, extractor: F) -> Self
    where
        F: Fn(&T) -> String + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            extractor: Box::new(extractor),
            width: None,
            sortable: true,
            visible: true,
        }
    }

    /// Set column width
    #[must_use]
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set whether column is sortable
    #[must_use]
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// Set whether column is visible
    #[must_use]
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Get column name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Extract cell value from row data
    pub fn extract(&self, row: &T) -> String {
        (self.extractor)(row)
    }

    /// Get column width
    #[must_use]
    pub fn column_width(&self) -> Option<usize> {
        self.width
    }

    /// Is column sortable?
    #[must_use]
    pub fn is_sortable(&self) -> bool {
        self.sortable
    }

    /// Is column visible?
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

/// Pagination state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pagination {
    /// Current page (0-indexed)
    pub current_page: usize,

    /// Rows per page
    pub page_size: usize,
}

impl Pagination {
    /// Create new pagination
    #[must_use]
    pub fn new(page_size: usize) -> Self {
        Self {
            current_page: 0,
            page_size,
        }
    }

    /// Get total pages for given row count
    #[must_use]
    pub fn total_pages(&self, total_rows: usize) -> usize {
        if total_rows == 0 || self.page_size == 0 {
            1
        } else {
            total_rows.div_ceil(self.page_size)
        }
    }

    /// Get start index for current page
    #[must_use]
    pub fn start_index(&self) -> usize {
        self.current_page * self.page_size
    }

    /// Get end index for current page
    #[must_use]
    pub fn end_index(&self, total_rows: usize) -> usize {
        ((self.current_page + 1) * self.page_size).min(total_rows)
    }

    /// Go to next page
    pub fn next_page(&mut self, total_rows: usize) {
        let total_pages = self.total_pages(total_rows);
        if self.current_page + 1 < total_pages {
            self.current_page += 1;
        }
    }

    /// Go to previous page
    pub fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }

    /// Go to specific page
    pub fn goto_page(&mut self, page: usize, total_rows: usize) {
        let total_pages = self.total_pages(total_rows);
        self.current_page = page.min(total_pages.saturating_sub(1));
    }
}

/// Generic table data structure
///
/// Tables are generic over the row data type `T` and use column
/// definitions to extract display values.
pub struct Table<T> {
    /// Column definitions
    columns: Vec<Column<T>>,

    /// Row data
    data: Vec<T>,

    /// Pagination state (None = no pagination)
    pagination: Option<Pagination>,

    /// Currently selected row (None = no selection)
    selected_row: Option<usize>,

    /// Sort column index and direction
    sort_state: Option<(usize, SortDirection)>,
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending order
    Ascending,
    /// Descending order
    Descending,
}

impl SortDirection {
    /// Toggle direction
    #[must_use]
    pub fn toggle(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

impl<T> Table<T> {
    /// Create a new empty table
    #[must_use]
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            data: Vec::new(),
            pagination: None,
            selected_row: None,
            sort_state: None,
        }
    }

    /// Add a column (builder pattern)
    #[must_use]
    pub fn with_column(mut self, column: Column<T>) -> Self {
        self.columns.push(column);
        self
    }

    /// Add multiple columns (builder pattern)
    #[must_use]
    pub fn with_columns(mut self, columns: Vec<Column<T>>) -> Self {
        self.columns.extend(columns);
        self
    }

    /// Set table data (builder pattern)
    #[must_use]
    pub fn with_data(mut self, data: Vec<T>) -> Self {
        self.data = data;
        self
    }

    /// Enable pagination (builder pattern)
    #[must_use]
    pub fn with_pagination(mut self, page_size: usize) -> Self {
        self.pagination = Some(Pagination::new(page_size));
        self
    }

    /// Add a column
    pub fn add_column(&mut self, column: Column<T>) {
        self.columns.push(column);
    }

    /// Add a row
    pub fn add_row(&mut self, row: T) {
        self.data.push(row);
    }

    /// Get columns
    #[must_use]
    pub fn columns(&self) -> &[Column<T>] {
        &self.columns
    }

    /// Get all row data
    #[must_use]
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// Get visible rows (respects pagination)
    #[must_use]
    pub fn visible_rows(&self) -> &[T] {
        if let Some(pagination) = &self.pagination {
            let start = pagination.start_index();
            let end = pagination.end_index(self.data.len());
            &self.data[start..end]
        } else {
            &self.data
        }
    }

    /// Get row count
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.data.len()
    }

    /// Get column count
    #[must_use]
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Get pagination state
    #[must_use]
    pub fn pagination(&self) -> Option<&Pagination> {
        self.pagination.as_ref()
    }

    /// Get mutable pagination state
    pub fn pagination_mut(&mut self) -> Option<&mut Pagination> {
        self.pagination.as_mut()
    }

    /// Get selected row index
    #[must_use]
    pub fn selected_row(&self) -> Option<usize> {
        self.selected_row
    }

    /// Select a row
    pub fn select_row(&mut self, index: usize) {
        if index < self.data.len() {
            self.selected_row = Some(index);
        }
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected_row = None;
    }

    /// Get cell value at (row, col)
    #[must_use]
    pub fn get_cell(&self, row: usize, col: usize) -> Option<String> {
        if row < self.data.len() && col < self.columns.len() {
            Some(self.columns[col].extract(&self.data[row]))
        } else {
            None
        }
    }

    /// Sort by custom comparator
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.data.sort_by(compare);
    }

    /// Sort by column (using string comparison)
    pub fn sort_by_column(&mut self, col_index: usize, direction: SortDirection) {
        if col_index >= self.columns.len() {
            return;
        }

        let column = &self.columns[col_index];
        if !column.is_sortable() {
            return;
        }

        self.data.sort_by(|a, b| {
            let val_a = column.extract(a);
            let val_b = column.extract(b);

            match direction {
                SortDirection::Ascending => val_a.cmp(&val_b),
                SortDirection::Descending => val_b.cmp(&val_a),
            }
        });

        self.sort_state = Some((col_index, direction));
    }

    /// Get current sort state
    #[must_use]
    pub fn sort_state(&self) -> Option<(usize, SortDirection)> {
        self.sort_state
    }
}

impl<T: Clone> Table<T> {
    /// Filter rows by predicate (returns new table)
    pub fn filter<F>(&self, predicate: F) -> Self
    where
        F: Fn(&T) -> bool,
    {
        let filtered_data: Vec<T> = self
            .data
            .iter()
            .filter(|row| predicate(row))
            .cloned()
            .collect();

        Self {
            columns: self
                .columns
                .iter()
                .map(|col| {
                    Column {
                        name: col.name.clone(),
                        extractor: Box::new(move |_| String::new()), // Placeholder
                        width: col.width,
                        sortable: col.sortable,
                        visible: col.visible,
                    }
                })
                .collect(),
            data: filtered_data,
            pagination: self.pagination,
            selected_row: None,
            sort_state: self.sort_state,
        }
    }
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Person {
        name: String,
        age: u32,
        email: String,
    }

    fn sample_people() -> Vec<Person> {
        vec![
            Person {
                name: "Alice".into(),
                age: 30,
                email: "alice@example.com".into(),
            },
            Person {
                name: "Bob".into(),
                age: 25,
                email: "bob@example.com".into(),
            },
            Person {
                name: "Charlie".into(),
                age: 35,
                email: "charlie@example.com".into(),
            },
            Person {
                name: "Diana".into(),
                age: 28,
                email: "diana@example.com".into(),
            },
        ]
    }

    #[test]
    fn test_table_creation() {
        let table: Table<Person> = Table::new();
        assert_eq!(table.row_count(), 0);
        assert_eq!(table.column_count(), 0);
    }

    #[test]
    fn test_table_with_data() {
        let table = Table::new()
            .with_column(Column::new("Name", |p: &Person| p.name.clone()))
            .with_column(Column::new("Age", |p: &Person| p.age.to_string()))
            .with_data(sample_people());

        assert_eq!(table.row_count(), 4);
        assert_eq!(table.column_count(), 2);
    }

    #[test]
    fn test_cell_extraction() {
        let table = Table::new()
            .with_column(Column::new("Name", |p: &Person| p.name.clone()))
            .with_column(Column::new("Age", |p: &Person| p.age.to_string()))
            .with_data(sample_people());

        assert_eq!(table.get_cell(0, 0), Some("Alice".to_string()));
        assert_eq!(table.get_cell(0, 1), Some("30".to_string()));
        assert_eq!(table.get_cell(1, 0), Some("Bob".to_string()));
        assert_eq!(table.get_cell(10, 0), None); // Out of bounds
    }

    #[test]
    fn test_pagination() {
        let mut table = Table::new()
            .with_column(Column::new("Name", |p: &Person| p.name.clone()))
            .with_data(sample_people())
            .with_pagination(2);

        // Page 0: Alice, Bob
        assert_eq!(table.visible_rows().len(), 2);
        assert_eq!(table.visible_rows()[0].name, "Alice");
        assert_eq!(table.visible_rows()[1].name, "Bob");

        // Go to page 1: Charlie, Diana
        let row_count = table.row_count();
        table.pagination_mut().unwrap().next_page(row_count);
        assert_eq!(table.visible_rows().len(), 2);
        assert_eq!(table.visible_rows()[0].name, "Charlie");
        assert_eq!(table.visible_rows()[1].name, "Diana");

        // Try to go beyond last page (should stay on page 1)
        let row_count = table.row_count();
        table.pagination_mut().unwrap().next_page(row_count);
        assert_eq!(table.pagination().unwrap().current_page, 1);
    }

    #[test]
    fn test_sorting() {
        let mut table = Table::new()
            .with_column(Column::new("Name", |p: &Person| p.name.clone()))
            .with_column(Column::new("Age", |p: &Person| p.age.to_string()))
            .with_data(sample_people());

        // Sort by age ascending
        table.sort_by(|a, b| a.age.cmp(&b.age));
        assert_eq!(table.data()[0].name, "Bob"); // age 25
        assert_eq!(table.data()[1].name, "Diana"); // age 28
        assert_eq!(table.data()[2].name, "Alice"); // age 30
        assert_eq!(table.data()[3].name, "Charlie"); // age 35
    }

    #[test]
    fn test_column_sorting() {
        let mut table = Table::new()
            .with_column(Column::new("Name", |p: &Person| p.name.clone()))
            .with_column(Column::new("Age", |p: &Person| p.age.to_string()))
            .with_data(sample_people());

        // Sort by name (column 0) ascending
        table.sort_by_column(0, SortDirection::Ascending);
        assert_eq!(table.data()[0].name, "Alice");
        assert_eq!(table.data()[3].name, "Diana");

        // Sort by name descending
        table.sort_by_column(0, SortDirection::Descending);
        assert_eq!(table.data()[0].name, "Diana");
        assert_eq!(table.data()[3].name, "Alice");
    }

    #[test]
    fn test_selection() {
        let mut table = Table::new()
            .with_column(Column::new("Name", |p: &Person| p.name.clone()))
            .with_data(sample_people());

        assert_eq!(table.selected_row(), None);

        table.select_row(1);
        assert_eq!(table.selected_row(), Some(1));

        table.clear_selection();
        assert_eq!(table.selected_row(), None);
    }

    #[test]
    fn test_column_properties() {
        let col = Column::new("Test", |p: &Person| p.name.clone())
            .width(100)
            .sortable(false)
            .visible(true);

        assert_eq!(col.name(), "Test");
        assert_eq!(col.column_width(), Some(100));
        assert!(!col.is_sortable());
        assert!(col.is_visible());
    }

    #[test]
    fn test_pagination_calculations() {
        let pagination = Pagination::new(10);

        // 25 rows, 10 per page = 3 pages
        assert_eq!(pagination.total_pages(25), 3);

        // Exact multiple
        assert_eq!(pagination.total_pages(30), 3);

        // Empty
        assert_eq!(pagination.total_pages(0), 1);
    }

    #[test]
    fn test_sort_direction_toggle() {
        let asc = SortDirection::Ascending;
        assert_eq!(asc.toggle(), SortDirection::Descending);

        let desc = SortDirection::Descending;
        assert_eq!(desc.toggle(), SortDirection::Ascending);
    }
}
