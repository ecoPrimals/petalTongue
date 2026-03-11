// SPDX-License-Identifier: AGPL-3.0-only
//! Grammar of Graphics expression types.
//!
//! A `GrammarExpr` declaratively describes a visualization:
//! data source, variable mappings, scale types, geometry, coordinate system,
//! faceting, and aesthetics. The grammar compiler transforms this into a `SceneGraph`.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Binding of a variable to a data field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableBinding {
    /// Variable name (e.g. "x", "y", "color").
    pub name: String,
    /// Data field name.
    pub field: String,
    /// Role of the variable.
    pub role: VariableRole,
}

/// Role of a variable in the grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableRole {
    X,
    Y,
    Z,
    Color,
    Size,
    Shape,
    Label,
    Detail,
}

/// Binding of a scale to a variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleBinding {
    /// Variable name.
    pub variable: String,
    /// Scale type.
    pub scale_type: ScaleType,
}

/// Scale type for variable mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleType {
    Linear,
    Log,
    Temporal,
    Categorical,
    Ordinal,
    Sqrt,
}

/// Geometry type (geom) for the visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeometryType {
    Point,
    Line,
    Bar,
    Area,
    Ribbon,
    Tile,
    Arc,
    ErrorBar,
    Mesh3D,
    Sphere,
    Cylinder,
    Text,
}

/// Coordinate system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinateSystem {
    Cartesian,
    Polar,
    Geographic,
    Perspective3D,
}

/// Facet layout for small multiples.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FacetLayout {
    /// Wrap facets into rows with a fixed column count.
    Wrap { columns: usize },
    /// Grid with row and column variable names.
    Grid { rows: String, cols: String },
}

/// Faceting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Facet {
    /// Variable to facet by.
    pub variable: String,
    /// Layout of the facets.
    pub layout: FacetLayout,
}

/// Aesthetic mapping (field name to aesthetic channel).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Aesthetic {
    Fill(String),
    Stroke(String),
    Size(String),
    Alpha(String),
    Shape(String),
    Label(String),
}

/// Grammar of Graphics expression.
///
/// Declaratively describes a visualization that the grammar compiler
/// transforms into a `SceneGraph`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarExpr {
    /// Data source identifier.
    pub data_source: String,
    /// Variable bindings (x, y, color, etc.).
    pub variables: Vec<VariableBinding>,
    /// Geometry type.
    pub geometry: GeometryType,
    /// Scale bindings.
    pub scales: Vec<ScaleBinding>,
    /// Coordinate system.
    pub coordinate: CoordinateSystem,
    /// Optional faceting.
    pub facets: Option<Facet>,
    /// Aesthetic mappings.
    pub aesthetics: Vec<Aesthetic>,
    /// Optional title.
    pub title: Option<String>,
    /// Optional domain hint.
    pub domain: Option<String>,
}

impl GrammarExpr {
    /// Create a minimal grammar expression.
    pub fn new(data_source: impl Into<String>, geometry: GeometryType) -> Self {
        Self {
            data_source: data_source.into(),
            variables: Vec::new(),
            geometry,
            scales: Vec::new(),
            coordinate: CoordinateSystem::Cartesian,
            facets: None,
            aesthetics: Vec::new(),
            title: None,
            domain: None,
        }
    }

    /// Add x variable binding.
    #[must_use]
    pub fn with_x(mut self, field: impl Into<String>) -> Self {
        self.variables.push(VariableBinding {
            name: "x".to_string(),
            field: field.into(),
            role: VariableRole::X,
        });
        self
    }

    /// Add y variable binding.
    #[must_use]
    pub fn with_y(mut self, field: impl Into<String>) -> Self {
        self.variables.push(VariableBinding {
            name: "y".to_string(),
            field: field.into(),
            role: VariableRole::Y,
        });
        self
    }

    /// Add color aesthetic binding.
    #[must_use]
    pub fn with_color(mut self, field: impl Into<String>) -> Self {
        self.aesthetics.push(Aesthetic::Fill(field.into()));
        self
    }

    /// Add size aesthetic binding.
    #[must_use]
    pub fn with_size(mut self, field: impl Into<String>) -> Self {
        self.aesthetics.push(Aesthetic::Size(field.into()));
        self
    }

    /// Add scale binding.
    #[must_use]
    pub fn with_scale(mut self, variable: impl Into<String>, scale_type: ScaleType) -> Self {
        self.scales.push(ScaleBinding {
            variable: variable.into(),
            scale_type,
        });
        self
    }

    /// Add faceting.
    #[must_use]
    pub fn with_facet(mut self, variable: impl Into<String>, layout: FacetLayout) -> Self {
        self.facets = Some(Facet {
            variable: variable.into(),
            layout,
        });
        self
    }

    /// Set title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set domain hint.
    #[must_use]
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Count of bound variables (data dimensions).
    #[must_use]
    pub const fn data_dimensions(&self) -> usize {
        self.variables.len()
    }

    /// Count of unique values implied by color aesthetic.
    /// Returns 0 if no color aesthetic is set.
    #[must_use]
    pub fn color_category_count(&self) -> usize {
        self.color_category_count_with_data(None)
    }

    /// Count unique string values in the category aesthetic field from data.
    /// When `data` is `Some`, extracts the Fill aesthetic field and counts
    /// unique string values in that column. When `data` is `None`, returns 0.
    #[must_use]
    pub fn color_category_count_with_data(&self, data: Option<&[serde_json::Value]>) -> usize {
        let field = self.aesthetics.iter().find_map(|a| {
            if let Aesthetic::Fill(f) = a {
                Some(f.as_str())
            } else {
                None
            }
        });
        let Some(field) = field else {
            return 0;
        };
        let Some(data) = data else {
            return 0;
        };
        let mut unique: HashSet<String> = HashSet::new();
        for row in data {
            let serde_json::Value::Object(obj) = row else {
                continue;
            };
            let val = match obj.get(field) {
                Some(serde_json::Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => continue,
            };
            unique.insert(val);
        }
        unique.len()
    }

    /// Whether faceting is configured.
    #[must_use]
    pub const fn has_facets(&self) -> bool {
        self.facets.is_some()
    }

    /// Whether the coordinate system is 3D.
    #[must_use]
    pub const fn uses_3d_coord(&self) -> bool {
        matches!(self.coordinate, CoordinateSystem::Perspective3D)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_expr_builder_chain() {
        let expr = GrammarExpr::new("table1", GeometryType::Point)
            .with_x("a")
            .with_y("b")
            .with_color("category")
            .with_scale("x", ScaleType::Linear)
            .with_facet("region", FacetLayout::Wrap { columns: 2 })
            .with_title("My Plot")
            .with_domain("experiment-1");
        assert_eq!(expr.data_source, "table1");
        assert_eq!(expr.geometry, GeometryType::Point);
        assert_eq!(expr.variables.len(), 2);
        assert_eq!(expr.aesthetics.len(), 1);
        assert_eq!(expr.scales.len(), 1);
        assert!(expr.facets.is_some());
        assert_eq!(expr.title.as_deref(), Some("My Plot"));
        assert_eq!(expr.domain.as_deref(), Some("experiment-1"));
    }

    #[test]
    fn data_dimensions_counts_correctly() {
        let expr = GrammarExpr::new("data", GeometryType::Line)
            .with_x("t")
            .with_y("value");
        assert_eq!(expr.data_dimensions(), 2);
    }

    #[test]
    fn has_facets_returns_true_when_set() {
        let expr = GrammarExpr::new("data", GeometryType::Bar)
            .with_x("x")
            .with_y("y")
            .with_facet("group", FacetLayout::Wrap { columns: 3 });
        assert!(expr.has_facets());
    }

    #[test]
    fn has_facets_returns_false_when_not_set() {
        let expr = GrammarExpr::new("data", GeometryType::Bar)
            .with_x("x")
            .with_y("y");
        assert!(!expr.has_facets());
    }

    #[test]
    fn uses_3d_coord_for_perspective3d() {
        let mut expr = GrammarExpr::new("data", GeometryType::Sphere);
        expr.coordinate = CoordinateSystem::Perspective3D;
        assert!(expr.uses_3d_coord());
    }

    #[test]
    fn uses_3d_coord_false_for_cartesian() {
        let expr = GrammarExpr::new("data", GeometryType::Point)
            .with_x("x")
            .with_y("y");
        assert!(!expr.uses_3d_coord());
    }

    #[test]
    fn serialization_roundtrip() {
        let expr = GrammarExpr::new("mydata", GeometryType::Line)
            .with_x("time")
            .with_y("value")
            .with_color("series");
        let json = serde_json::to_string(&expr).unwrap();
        let decoded: GrammarExpr = serde_json::from_str(&json).unwrap();
        assert_eq!(expr.data_source, decoded.data_source);
        assert_eq!(expr.geometry, decoded.geometry);
        assert_eq!(expr.variables.len(), decoded.variables.len());
    }

    #[test]
    fn color_category_count_with_data_counts_unique_values() {
        let expr = GrammarExpr::new("data", GeometryType::Point)
            .with_x("x")
            .with_y("y")
            .with_color("category");
        let data = vec![
            serde_json::json!({"x": 1, "y": 2, "category": "A"}),
            serde_json::json!({"x": 2, "y": 3, "category": "B"}),
            serde_json::json!({"x": 3, "y": 4, "category": "A"}),
        ];
        let data: Vec<serde_json::Value> = data;
        assert_eq!(expr.color_category_count_with_data(Some(&data)), 2);
    }

    #[test]
    fn color_category_count_without_fill_returns_zero() {
        let expr = GrammarExpr::new("data", GeometryType::Point)
            .with_x("x")
            .with_y("y");
        assert_eq!(expr.color_category_count(), 0);
    }
}
