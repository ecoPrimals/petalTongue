// SPDX-License-Identifier: AGPL-3.0-only
//! Graph validation types: severity, issues, and results.

/// Validation error severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    /// Error: Graph cannot be executed
    Error,
    /// Warning: Graph may have issues
    Warning,
    /// Info: Suggestion for improvement
    Info,
}

/// Validation result for a graph
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// Severity level
    pub severity: ValidationSeverity,
    /// Node ID if issue is node-specific
    pub node_id: Option<String>,
    /// Edge index if issue is edge-specific
    pub edge_index: Option<usize>,
    /// Human-readable description
    pub message: String,
    /// Optional suggestion for fixing
    pub suggestion: Option<String>,
}

impl ValidationIssue {
    /// Create an error
    #[must_use]
    pub fn error(message: String) -> Self {
        Self {
            severity: ValidationSeverity::Error,
            node_id: None,
            edge_index: None,
            message,
            suggestion: None,
        }
    }

    /// Create an error for a specific node
    #[must_use]
    pub fn node_error(node_id: String, message: String) -> Self {
        Self {
            severity: ValidationSeverity::Error,
            node_id: Some(node_id),
            edge_index: None,
            message,
            suggestion: None,
        }
    }

    /// Create a warning
    #[must_use]
    pub fn warning(message: String) -> Self {
        Self {
            severity: ValidationSeverity::Warning,
            node_id: None,
            edge_index: None,
            message,
            suggestion: None,
        }
    }

    /// Create a warning for a specific node
    #[must_use]
    pub fn node_warning(node_id: String, message: String) -> Self {
        Self {
            severity: ValidationSeverity::Warning,
            node_id: Some(node_id),
            edge_index: None,
            message,
            suggestion: None,
        }
    }

    /// Add a suggestion
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }
}

/// Graph validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// All validation issues
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    /// Create a new validation result
    #[must_use]
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    /// Add an issue
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    /// Check if there are any errors
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Error)
    }

    /// Check if there are any warnings
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == ValidationSeverity::Warning)
    }

    /// Check if validation passed (no errors)
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.has_errors()
    }

    /// Get all errors
    #[must_use]
    pub fn errors(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Error)
            .collect()
    }

    /// Get all warnings
    #[must_use]
    pub fn warnings(&self) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Warning)
            .collect()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}
