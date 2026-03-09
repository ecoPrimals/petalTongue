# Tufte Constraint System

**Version**: 1.0.0  
**Date**: March 8, 2026  
**Status**: Design Phase  
**Priority**: Medium (Integrated into Grammar Compiler)  
**Depends On**: `GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md`

---

## Purpose

Encode Edward Tufte's principles of data visualization as **machine-checkable
constraints** that the grammar compiler evaluates on every render. Violations
produce warnings or auto-corrections, not hard errors -- the human has final say,
but petalTongue tells them when a visualization is misleading, wasteful, or
inaccessible.

This is uniquely possible in a type-safe grammar system. No existing visualization
library treats Tufte's principles as invariants. petalTongue should be the first.

---

## Background: Tufte's Core Principles

Edward Tufte's *The Visual Display of Quantitative Information* (1983, 2001)
and subsequent works establish principles that are not aesthetic preferences but
information-theoretic constraints on how humans decode visual representations.

The principles that are machine-checkable:

| Principle | What It Means | Why It Matters |
|-----------|--------------|----------------|
| **Data-Ink Ratio** | Maximize proportion of ink carrying data | Reduces cognitive load |
| **Lie Factor** | Visual effect size must equal data effect size | Prevents deception |
| **Chartjunk** | Remove non-data decorations | Reduces noise |
| **Small Multiples** | Repeat structure, vary one dimension | Enables comparison |
| **Data Density** | Maximize information per unit area | Respects attention |
| **Micro/Macro** | Readable at multiple scales | Supports different tasks |
| **Sparklines** | Dense, word-sized graphics | Embeds data in context |
| **Smallest Effective Difference** | Use minimum visual distinction | Avoids distortion |

---

## Constraint Architecture

### Constraint Trait

```rust
pub trait TufteConstraint: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&self, plan: &RenderPlan, expr: &GrammarExpr) -> ConstraintResult;
    fn severity(&self) -> ConstraintSeverity;
    fn auto_correctable(&self) -> bool;
    fn auto_correct(&self, plan: &mut RenderPlan) -> bool;
}

pub enum ConstraintSeverity {
    Info,
    Warning,
    Error,
}

pub struct ConstraintResult {
    pub passed: bool,
    pub score: f64,
    pub message: String,
    pub details: Option<ConstraintDetails>,
}
```

### Constraint Set

The grammar compiler runs all constraints after producing the initial
`RenderPlan` and before handing it to modality compilers:

```rust
pub struct TufteConstraints {
    pub enabled: bool,
    pub constraints: Vec<Box<dyn TufteConstraint>>,
    pub auto_correct: bool,
    pub min_score: f64,
}

impl Default for TufteConstraints {
    fn default() -> Self {
        Self {
            enabled: true,
            constraints: all_default_constraints(),
            auto_correct: true,
            min_score: 0.0, // warn but don't block
        }
    }
}
```

### Constraints Report

Included in every `RenderPlan`:

```rust
pub struct ConstraintsReport {
    pub overall_score: f64,
    pub results: Vec<(String, ConstraintResult)>,
    pub corrections_applied: Vec<String>,
}
```

The overall score (0.0 to 1.0) is displayed in the grammar builder UI and
available via `visualization.validate` JSON-RPC. Other primals can use it to
assess whether their grammar expression produces a good visualization before
showing it to a human.

---

## Constraint Implementations

### 1. Data-Ink Ratio

**Principle**: Of all the visual elements rendered, what fraction carries data?
Elements that don't carry data (grid lines, background fills, decorative borders,
3D perspective on 2D data) are "non-data ink."

**Measurement**:

```rust
pub struct DataInkRatio;

impl TufteConstraint for DataInkRatio {
    fn evaluate(&self, plan: &RenderPlan, _expr: &GrammarExpr) -> ConstraintResult {
        let mut data_primitives = 0usize;
        let mut total_primitives = 0usize;

        for panel in &plan.panels {
            for prim in &panel.primitives {
                total_primitives += 1;
                if prim.carries_data() {
                    data_primitives += 1;
                }
            }
            // Grid lines, axis lines, tick marks are non-data
            total_primitives += panel.grid_line_count();
            total_primitives += panel.axis_decoration_count();
        }

        // Legends, titles are structural (not counted against ratio)
        let ratio = if total_primitives > 0 {
            data_primitives as f64 / total_primitives as f64
        } else {
            1.0
        };

        ConstraintResult {
            passed: ratio >= 0.5,
            score: ratio,
            message: format!("Data-ink ratio: {:.0}% ({data_primitives}/{total_primitives})", ratio * 100.0),
            details: None,
        }
    }

    fn auto_correct(&self, plan: &mut RenderPlan) -> bool {
        // Remove grid lines if ratio is below threshold
        // Reduce axis decorations to minimum
        // Remove background fills
        let mut corrected = false;
        for panel in &mut plan.panels {
            if panel.grid_lines == GridLines::Both {
                panel.grid_lines = GridLines::MajorOnly;
                corrected = true;
            }
        }
        corrected
    }
}
```

**Thresholds**:
- >= 0.75: Excellent (Tufte-approved)
- 0.50 - 0.75: Acceptable
- < 0.50: Warning ("More decoration than data")
- < 0.25: Error ("Visualization is mostly noise")

### 2. Lie Factor

**Principle**: The size of an effect shown in the graphic should equal the size
of the effect in the data. `Lie Factor = (visual effect size) / (data effect size)`.
A lie factor of 1.0 is truthful. Greater than 1.0 exaggerates. Less than 1.0
diminishes.

**Common violations**:
- Truncated Y axis (bar chart starting at 50 instead of 0)
- Area/radius confusion (doubling radius quadruples area)
- 3D perspective on 2D data (foreshortening distorts size)
- Non-zero baseline for area charts

**Measurement**:

```rust
pub struct LieFactor;

impl TufteConstraint for LieFactor {
    fn evaluate(&self, plan: &RenderPlan, expr: &GrammarExpr) -> ConstraintResult {
        let mut max_lie = 1.0f64;
        let mut violations = Vec::new();

        for (i, panel) in plan.panels.iter().enumerate() {
            // Check Y axis: does it start at 0 for bar/area charts?
            if expr.has_bar_or_area_geom() {
                if let Some(y_min) = panel.y_scale_domain_min() {
                    if y_min > 0.0 {
                        let data_range = panel.y_scale_domain_max().unwrap_or(100.0) - 0.0;
                        let visual_range = panel.y_scale_domain_max().unwrap_or(100.0) - y_min;
                        let lie = data_range / visual_range;
                        max_lie = max_lie.max(lie);
                        violations.push(format!(
                            "Panel {i}: Y axis starts at {y_min}, not 0 (lie factor {lie:.2}x)"
                        ));
                    }
                }
            }

            // Check size aesthetic: is it area-proportional?
            if expr.has_size_aesthetic() && !expr.size_is_area_proportional() {
                max_lie = max_lie.max(2.0);
                violations.push(format!(
                    "Panel {i}: Size mapped to radius, not area (up to 4x distortion)"
                ));
            }
        }

        let passed = max_lie <= 1.05; // 5% tolerance
        ConstraintResult {
            passed,
            score: 1.0 / max_lie, // 1.0 is perfect, lower is worse
            message: if passed {
                "Lie factor: 1.0 (truthful)".to_string()
            } else {
                format!("Lie factor: {max_lie:.2}x — {}", violations.join("; "))
            },
            details: None,
        }
    }

    fn auto_correct(&self, plan: &mut RenderPlan) -> bool {
        let mut corrected = false;
        for panel in &mut plan.panels {
            // Extend Y axis to include 0 for bar/area charts
            if panel.needs_zero_baseline() {
                panel.extend_y_to_zero();
                corrected = true;
            }
        }
        // Convert radius-proportional size to area-proportional
        // (sqrt transform on the size scale)
        corrected
    }
}
```

**Thresholds**:
- 0.95 - 1.05: Truthful
- 1.05 - 2.0: Warning ("Moderate exaggeration")
- > 2.0: Error ("Misleading visualization")

### 3. Chartjunk Detection

**Principle**: Decorative elements that don't serve the data are chartjunk.
Includes: 3D effects on 2D data, gradient fills, drop shadows, unnecessary
icons, logos, watermarks, moire patterns.

**Measurement**:

```rust
pub struct ChartjunkDetection;

impl TufteConstraint for ChartjunkDetection {
    fn evaluate(&self, plan: &RenderPlan, expr: &GrammarExpr) -> ConstraintResult {
        let mut junk_count = 0u32;
        let mut issues = Vec::new();

        // 3D perspective on inherently 2D data
        if expr.data_dimensions() <= 2 && expr.uses_3d_coord() {
            junk_count += 1;
            issues.push("3D coordinates for 2D data (unnecessary perspective)");
        }

        // Gradient fills that don't encode data
        if plan.has_decorative_gradients() {
            junk_count += 1;
            issues.push("Decorative gradients (not mapped to data)");
        }

        // Excessive grid lines
        if plan.grid_line_density() > 10 {
            junk_count += 1;
            issues.push("Excessive grid lines (>10 per axis)");
        }

        // Dual Y axes (almost always misleading)
        if plan.has_dual_y_axes() {
            junk_count += 1;
            issues.push("Dual Y axes (use facets instead)");
        }

        let score = 1.0 - (junk_count as f64 * 0.25).min(1.0);
        ConstraintResult {
            passed: junk_count == 0,
            score,
            message: if junk_count == 0 {
                "No chartjunk detected".to_string()
            } else {
                format!("{junk_count} chartjunk issue(s): {}", issues.join(", "))
            },
            details: None,
        }
    }
}
```

### 4. Small Multiples Preference

**Principle**: When comparing across a categorical variable, small multiples
(faceting) are almost always better than overlaying, dual axes, or animation.

**Trigger**: Grammar expression has > 3 categories in a color aesthetic.

```rust
pub struct SmallMultiplesPreference;

impl TufteConstraint for SmallMultiplesPreference {
    fn evaluate(&self, _plan: &RenderPlan, expr: &GrammarExpr) -> ConstraintResult {
        let color_categories = expr.color_category_count();
        let has_facets = expr.facets.is_some();

        if color_categories > 3 && !has_facets {
            ConstraintResult {
                passed: false,
                score: 0.5,
                message: format!(
                    "{color_categories} color categories without faceting — \
                     consider small multiples (facet by the categorical variable)"
                ),
                details: None,
            }
        } else {
            ConstraintResult {
                passed: true,
                score: 1.0,
                message: if has_facets {
                    "Using small multiples (good)".to_string()
                } else {
                    "Category count manageable without faceting".to_string()
                },
                details: None,
            }
        }
    }
}
```

### 5. Data Density

**Principle**: Maximize the information content per unit of display area.
Sparse visualizations waste the human's limited visual bandwidth.

**Measurement**: `data points per 100x100 pixel region`. Empty regions
indicate the visualization could be denser (smaller, or showing more data).

```rust
pub struct DataDensity;

impl TufteConstraint for DataDensity {
    fn evaluate(&self, plan: &RenderPlan, _expr: &GrammarExpr) -> ConstraintResult {
        let total_data_points: usize = plan.panels.iter()
            .map(|p| p.data_primitive_count())
            .sum();

        let total_area: f64 = plan.panels.iter()
            .map(|p| p.bounds.width() * p.bounds.height())
            .sum();

        let density = if total_area > 0.0 {
            total_data_points as f64 / (total_area / 10_000.0)
        } else {
            0.0
        };

        let score = (density / 100.0).min(1.0); // 100 points per 100x100 is good
        ConstraintResult {
            passed: density >= 1.0, // at least 1 point per 100x100 region
            score,
            message: format!("Data density: {density:.1} points per 10Kpx"),
            details: None,
        }
    }
}
```

### 6. Color Accessibility

**Principle**: Color should never be the sole channel for critical information.
Color-blind users (8% of males) cannot distinguish red/green.

This extends Tufte into accessibility territory, which aligns with petalTongue's
UUI mission: if a visualization only works for sighted users with full color
vision, it's not universal.

```rust
pub struct ColorAccessibility;

impl TufteConstraint for ColorAccessibility {
    fn evaluate(&self, _plan: &RenderPlan, expr: &GrammarExpr) -> ConstraintResult {
        let mut issues = Vec::new();

        // Color as sole distinguishing aesthetic
        if expr.uses_color_for_categories() && !expr.has_redundant_channel() {
            issues.push("Color is the sole channel for categories — add shape or pattern");
        }

        // Red/green palette
        if expr.uses_red_green_palette() {
            issues.push("Red/green color scheme — use colorblind-safe palette");
        }

        // Too many colors
        if expr.color_category_count() > 7 {
            issues.push("More than 7 colors — human color discrimination limit");
        }

        let passed = issues.is_empty();
        ConstraintResult {
            passed,
            score: if passed { 1.0 } else { 0.5 },
            message: if passed {
                "Color usage is accessible".to_string()
            } else {
                format!("Accessibility: {}", issues.join("; "))
            },
            details: None,
        }
    }

    fn auto_correct(&self, plan: &mut RenderPlan) -> bool {
        // Replace red/green with colorblind-safe palette (blue/orange)
        // Add shape variation as redundant channel
        plan.apply_colorblind_safe_palette()
    }
}
```

### 7. Smallest Effective Difference

**Principle**: Visual distinctions should be as small as possible while still
being perceptible. Thick borders, bold colors, and large markers are noisy
when thin lines, muted tones, and small dots would suffice.

```rust
pub struct SmallestEffectiveDifference;

impl TufteConstraint for SmallestEffectiveDifference {
    fn evaluate(&self, plan: &RenderPlan, _expr: &GrammarExpr) -> ConstraintResult {
        let mut oversized = 0u32;

        for panel in &plan.panels {
            // Line widths > 2px for data lines (not axes)
            if panel.max_data_line_width() > 2.0 {
                oversized += 1;
            }
            // Point sizes > 10px for standard scatterplots
            if panel.max_point_size() > 10.0 && panel.data_primitive_count() > 20 {
                oversized += 1;
            }
            // Grid lines more prominent than data lines
            if panel.grid_line_opacity() > panel.data_line_opacity() {
                oversized += 1;
            }
        }

        ConstraintResult {
            passed: oversized == 0,
            score: 1.0 - (oversized as f64 * 0.2).min(1.0),
            message: if oversized == 0 {
                "Visual weight appropriate".to_string()
            } else {
                format!("{oversized} element(s) visually heavier than necessary")
            },
            details: None,
        }
    }
}
```

---

## Multi-Modal Constraint Extensions

Tufte's principles were formulated for print graphics. petalTongue extends them
to other modalities:

### Audio Constraints

| Constraint | Principle | Measurement |
|------------|-----------|-------------|
| **Pitch Range** | Don't use frequencies most humans can't hear | 100Hz - 4000Hz |
| **Temporal Density** | Don't play sounds faster than perception allows | >= 100ms per datum |
| **Volume Normalization** | Equal loudness for equal data magnitude | LUFS normalization |
| **Redundant Encoding** | Pitch alone is hard to decode; add rhythm or timbre | >= 2 audio channels |

### TUI Constraints

| Constraint | Principle | Measurement |
|------------|-----------|-------------|
| **Character Efficiency** | Maximize data per character cell | Braille > block > ASCII |
| **Terminal Width** | Don't exceed terminal width | Query $COLUMNS |
| **Color Depth** | Don't assume 24-bit color | Query $COLORTERM |
| **Scrollback** | Dense single-screen views over scrolling | Height <= $LINES |

---

## Integration with Grammar Compiler

The constraint system runs as the second-to-last phase of grammar compilation,
after geometry rendering and before modality dispatch:

```
Scales → Statistics → Geometry → TUFTE CONSTRAINTS → Modality Compiler
```

### Configuration

Users can configure constraint behavior via environment variable or config:

```toml
[visualization.constraints]
enabled = true
auto_correct = true
min_overall_score = 0.0    # 0.0 = warn only, 0.5 = block bad visualizations
colorblind_safe = true
```

### JSON-RPC Validation Endpoint

Other primals can validate grammar expressions before committing to a render:

```json
{
  "jsonrpc": "2.0",
  "method": "visualization.validate",
  "params": {
    "grammar": { ... },
    "constraints": { "strict": true }
  },
  "id": 1
}
```

Response:

```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": true,
    "overall_score": 0.87,
    "constraints": [
      {"name": "data_ink_ratio", "passed": true, "score": 0.82},
      {"name": "lie_factor", "passed": true, "score": 1.0},
      {"name": "chartjunk", "passed": true, "score": 1.0},
      {"name": "color_accessibility", "passed": false, "score": 0.5,
       "message": "Color is sole channel — add shape variation",
       "auto_corrected": true}
    ]
  },
  "id": 1
}
```

---

## Human Dignity Considerations

The Tufte constraint system directly serves human dignity:

1. **Truthful representation**: The lie factor constraint ensures data is not
   distorted. Misleading visualizations violate a user's right to accurate
   information about their own systems.

2. **Accessibility**: The color accessibility constraint ensures blind, color-blind,
   and low-vision users can access the same information. A UUI that only works
   for fully sighted users is not universal.

3. **Cognitive respect**: The data-ink ratio and chartjunk constraints respect the
   human's limited attention. Every unnecessary decoration is a tax on cognition.

4. **Transparency**: The constraints report is always available. The human can see
   the score, understand the warnings, and override them. petalTongue advises but
   never censors.

---

## References

- Tufte, E.R. (2001). *The Visual Display of Quantitative Information*. 2nd edition.
- Tufte, E.R. (1990). *Envisioning Information*.
- Tufte, E.R. (1997). *Visual Explanations*.
- Tufte, E.R. (2006). *Beautiful Evidence*.
- Few, S. (2012). *Show Me the Numbers*. 2nd edition.
- Ware, C. (2012). *Information Visualization: Perception for Design*. 3rd edition.
- Brewer, C. (2003). ColorBrewer: Color advice for cartography.

---

**Status**: Ready for implementation (integrated into grammar compiler)  
**Blocking**: `GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md` Phase 1  
**First Milestone**: Data-ink ratio + lie factor on existing topology view
