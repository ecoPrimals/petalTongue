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
    pub const fn error(message: String) -> Self {
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
    pub const fn node_error(node_id: String, message: String) -> Self {
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
    pub const fn warning(message: String) -> Self {
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
    pub const fn node_warning(node_id: String, message: String) -> Self {
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
    pub const fn new() -> Self {
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

impl std::fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "Error"),
            Self::Warning => write!(f, "Warning"),
            Self::Info => write!(f, "Info"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_issue_error() {
        let issue = ValidationIssue::error("msg".to_string());
        assert_eq!(issue.severity, ValidationSeverity::Error);
        assert!(issue.node_id.is_none());
        assert_eq!(issue.message, "msg");
    }

    #[test]
    fn test_validation_issue_node_error() {
        let issue = ValidationIssue::node_error("n1".to_string(), "err".to_string());
        assert_eq!(issue.severity, ValidationSeverity::Error);
        assert_eq!(issue.node_id.as_deref(), Some("n1"));
        assert_eq!(issue.message, "err");
    }

    #[test]
    fn test_validation_issue_warning() {
        let issue = ValidationIssue::warning("warn".to_string());
        assert_eq!(issue.severity, ValidationSeverity::Warning);
    }

    #[test]
    fn test_validation_issue_with_suggestion() {
        let issue = ValidationIssue::error("e".to_string()).with_suggestion("fix".to_string());
        assert_eq!(issue.suggestion.as_deref(), Some("fix"));
    }

    #[test]
    fn test_validation_result_new() {
        let r = ValidationResult::new();
        assert!(r.issues.is_empty());
        assert!(r.is_valid());
        assert!(!r.has_errors());
        assert!(!r.has_warnings());
    }

    #[test]
    fn test_validation_result_has_errors() {
        let mut r = ValidationResult::new();
        r.add_issue(ValidationIssue::error("e".to_string()));
        assert!(r.has_errors());
        assert!(!r.is_valid());
    }

    #[test]
    fn test_validation_result_has_warnings() {
        let mut r = ValidationResult::new();
        r.add_issue(ValidationIssue::warning("w".to_string()));
        assert!(r.has_warnings());
        assert!(r.is_valid());
    }

    #[test]
    fn test_validation_result_errors_warnings() {
        let mut r = ValidationResult::new();
        r.add_issue(ValidationIssue::error("e1".to_string()));
        r.add_issue(ValidationIssue::warning("w1".to_string()));
        assert_eq!(r.errors().len(), 1);
        assert_eq!(r.warnings().len(), 1);
    }

    #[test]
    fn test_validation_issue_node_warning() {
        let issue = ValidationIssue::node_warning("n1".to_string(), "warn".to_string());
        assert_eq!(issue.severity, ValidationSeverity::Warning);
        assert_eq!(issue.node_id.as_deref(), Some("n1"));
    }

    #[test]
    fn test_validation_result_default() {
        let r = ValidationResult::default();
        assert!(r.issues.is_empty());
        assert!(r.is_valid());
    }

    #[test]
    fn test_validation_severity_display() {
        assert_eq!(ValidationSeverity::Error.to_string(), "Error");
        assert_eq!(ValidationSeverity::Warning.to_string(), "Warning");
        assert_eq!(ValidationSeverity::Info.to_string(), "Info");
    }

    #[test]
    fn test_validation_result_with_info() {
        let mut r = ValidationResult::new();
        r.add_issue(ValidationIssue {
            severity: ValidationSeverity::Info,
            node_id: None,
            edge_index: None,
            message: "info".to_string(),
            suggestion: None,
        });
        assert!(r.is_valid());
        assert!(!r.has_errors());
        assert!(!r.has_warnings());
    }

    #[test]
    fn test_validation_issue_edge_index() {
        let issue = ValidationIssue {
            severity: ValidationSeverity::Error,
            node_id: None,
            edge_index: Some(3),
            message: "edge err".to_string(),
            suggestion: None,
        };
        assert_eq!(issue.edge_index, Some(3));
    }

    #[test]
    fn test_validation_issue_node_warning_with_suggestion() {
        let issue = ValidationIssue::node_warning("n1".to_string(), "warn".to_string())
            .with_suggestion("fix it".to_string());
        assert_eq!(issue.suggestion.as_deref(), Some("fix it"));
    }

    #[test]
    fn test_validation_result_multiple_errors() {
        let mut r = ValidationResult::new();
        r.add_issue(ValidationIssue::error("e1".to_string()));
        r.add_issue(ValidationIssue::error("e2".to_string()));
        assert_eq!(r.errors().len(), 2);
        assert!(!r.is_valid());
    }
}
