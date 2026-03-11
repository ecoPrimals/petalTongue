// SPDX-License-Identifier: AGPL-3.0-only
//! Equation rendering: LaTeX-like math notation to scene primitives.
//!
//! Converts a subset of LaTeX math notation into `Primitive` values
//! (Text, Line for fraction bars) that can be rendered through any
//! modality compiler. This provides Manim-style mathematical notation
//! without depending on external LaTeX engines.
//!
//! ## Supported Syntax
//!
//! - Fractions: `\frac{a}{b}`
//! - Superscripts: `x^{2}` or `x^2`
//! - Subscripts: `x_{i}` or `x_i`
//! - Greek letters: `\alpha`, `\beta`, `\pi`, `\sigma`, etc.
//! - Operators: `\sum`, `\prod`, `\int`, `\sqrt`
//! - Common symbols: `\infty`, `\pm`, `\cdot`, `\times`, `\leq`, `\geq`, `\neq`

use crate::primitive::{AnchorPoint, Color, Primitive, StrokeStyle};

/// Compiles a LaTeX-like math string into scene primitives.
#[derive(Debug, Clone)]
pub struct EquationCompiler {
    pub font_size: f64,
    pub color: Color,
    pub origin_x: f64,
    pub origin_y: f64,
}

impl Default for EquationCompiler {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            color: Color::WHITE,
            origin_x: 0.0,
            origin_y: 0.0,
        }
    }
}

impl EquationCompiler {
    #[must_use]
    pub const fn new(font_size: f64, color: Color) -> Self {
        Self {
            font_size,
            color,
            origin_x: 0.0,
            origin_y: 0.0,
        }
    }

    #[must_use]
    pub const fn at(mut self, x: f64, y: f64) -> Self {
        self.origin_x = x;
        self.origin_y = y;
        self
    }

    /// Compile a LaTeX-like math string to primitives.
    #[must_use]
    pub fn compile(&self, latex: &str) -> Vec<Primitive> {
        let mut primitives = Vec::new();
        let mut cursor = Cursor {
            x: self.origin_x,
            y: self.origin_y,
            font_size: self.font_size,
            color: self.color,
        };

        self.parse_expr(latex, &mut cursor, &mut primitives);
        primitives
    }

    fn parse_expr(&self, input: &str, cursor: &mut Cursor, out: &mut Vec<Primitive>) {
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                '\\' => {
                    let cmd = Self::read_command(&chars, &mut i);
                    self.emit_command(&cmd, &chars, &mut i, cursor, out);
                }
                '^' => {
                    i += 1;
                    let arg = Self::read_arg(&chars, &mut i);
                    let saved = *cursor;
                    cursor.y -= cursor.font_size * 0.4;
                    cursor.font_size *= 0.7;
                    self.parse_expr(&arg, cursor, out);
                    cursor.font_size = saved.font_size;
                    cursor.y = saved.y;
                }
                '_' => {
                    i += 1;
                    let arg = Self::read_arg(&chars, &mut i);
                    let saved = *cursor;
                    cursor.y += cursor.font_size * 0.3;
                    cursor.font_size *= 0.7;
                    self.parse_expr(&arg, cursor, out);
                    cursor.font_size = saved.font_size;
                    cursor.y = saved.y;
                }
                '{' | '}' => {
                    i += 1;
                }
                ' ' => {
                    cursor.x += cursor.font_size * 0.25;
                    i += 1;
                }
                ch => {
                    out.push(Primitive::Text {
                        x: cursor.x,
                        y: cursor.y,
                        content: ch.to_string(),
                        font_size: cursor.font_size,
                        color: cursor.color,
                        anchor: AnchorPoint::CenterLeft,
                        bold: false,
                        italic: true,
                        data_id: None,
                    });
                    cursor.x += cursor.font_size * 0.6;
                    i += 1;
                }
            }
        }
    }

    fn read_command(chars: &[char], i: &mut usize) -> String {
        *i += 1; // skip backslash
        let start = *i;
        while *i < chars.len() && chars[*i].is_ascii_alphabetic() {
            *i += 1;
        }
        chars[start..*i].iter().collect()
    }

    fn read_arg(chars: &[char], i: &mut usize) -> String {
        if *i < chars.len() && chars[*i] == '{' {
            *i += 1;
            let start = *i;
            let mut depth = 1;
            while *i < chars.len() && depth > 0 {
                if chars[*i] == '{' {
                    depth += 1;
                } else if chars[*i] == '}' {
                    depth -= 1;
                }
                if depth > 0 {
                    *i += 1;
                }
            }
            let result: String = chars[start..*i].iter().collect();
            if *i < chars.len() {
                *i += 1; // skip closing brace
            }
            result
        } else if *i < chars.len() {
            let ch = chars[*i];
            *i += 1;
            ch.to_string()
        } else {
            String::new()
        }
    }

    fn emit_command(
        &self,
        cmd: &str,
        chars: &[char],
        i: &mut usize,
        cursor: &mut Cursor,
        out: &mut Vec<Primitive>,
    ) {
        match cmd {
            "frac" => {
                let num = Self::read_arg(chars, i);
                let den = Self::read_arg(chars, i);
                let frac_x = cursor.x;
                let frac_width = cursor.font_size * 1.5;

                let mut num_cursor = Cursor {
                    x: frac_x + frac_width * 0.1,
                    y: cursor.font_size.mul_add(-0.5, cursor.y),
                    font_size: cursor.font_size * 0.8,
                    color: cursor.color,
                };
                self.parse_expr(&num, &mut num_cursor, out);

                out.push(Primitive::Line {
                    points: vec![[frac_x, cursor.y], [frac_x + frac_width, cursor.y]],
                    stroke: StrokeStyle {
                        color: cursor.color,
                        width: 1.0,
                        ..StrokeStyle::default()
                    },
                    closed: false,
                    data_id: None,
                });

                let mut den_cursor = Cursor {
                    x: frac_x + frac_width * 0.1,
                    y: cursor.font_size.mul_add(0.6, cursor.y),
                    font_size: cursor.font_size * 0.8,
                    color: cursor.color,
                };
                self.parse_expr(&den, &mut den_cursor, out);

                cursor.x = cursor.font_size.mul_add(0.2, frac_x + frac_width);
            }
            "sqrt" => {
                let arg = Self::read_arg(chars, i);
                let start_x = cursor.x;
                let symbol = "\u{221A}";
                out.push(Primitive::Text {
                    x: cursor.x,
                    y: cursor.y,
                    content: symbol.to_string(),
                    font_size: cursor.font_size * 1.2,
                    color: cursor.color,
                    anchor: AnchorPoint::CenterLeft,
                    bold: false,
                    italic: false,
                    data_id: None,
                });
                cursor.x += cursor.font_size * 0.8;
                self.parse_expr(&arg, cursor, out);

                out.push(Primitive::Line {
                    points: vec![
                        [
                            cursor.font_size.mul_add(0.7, start_x),
                            cursor.font_size.mul_add(-0.5, cursor.y),
                        ],
                        [cursor.x, cursor.font_size.mul_add(-0.5, cursor.y)],
                    ],
                    stroke: StrokeStyle {
                        color: cursor.color,
                        width: 1.0,
                        ..StrokeStyle::default()
                    },
                    closed: false,
                    data_id: None,
                });
                cursor.x += cursor.font_size * 0.2;
            }
            _ => {
                let symbol = latex_symbol(cmd);
                out.push(Primitive::Text {
                    x: cursor.x,
                    y: cursor.y,
                    content: symbol.to_string(),
                    font_size: cursor.font_size,
                    color: cursor.color,
                    anchor: AnchorPoint::CenterLeft,
                    bold: false,
                    italic: false,
                    data_id: None,
                });
                cursor.x += cursor.font_size * char_width_factor(symbol);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Cursor {
    x: f64,
    y: f64,
    font_size: f64,
    color: Color,
}

fn char_width_factor(s: &str) -> f64 {
    match s.len() {
        0 => 0.0,
        1 => 0.6,
        _ => s.chars().count() as f64 * 0.5,
    }
}

/// Map a LaTeX command name to its Unicode symbol.
fn latex_symbol(cmd: &str) -> &'static str {
    match cmd {
        "alpha" => "\u{03B1}",
        "beta" => "\u{03B2}",
        "gamma" => "\u{03B3}",
        "delta" => "\u{03B4}",
        "epsilon" => "\u{03B5}",
        "zeta" => "\u{03B6}",
        "eta" => "\u{03B7}",
        "theta" => "\u{03B8}",
        "iota" => "\u{03B9}",
        "kappa" => "\u{03BA}",
        "lambda" => "\u{03BB}",
        "mu" => "\u{03BC}",
        "nu" => "\u{03BD}",
        "xi" => "\u{03BE}",
        "pi" => "\u{03C0}",
        "rho" => "\u{03C1}",
        "sigma" => "\u{03C3}",
        "tau" => "\u{03C4}",
        "upsilon" => "\u{03C5}",
        "phi" => "\u{03C6}",
        "chi" => "\u{03C7}",
        "psi" => "\u{03C8}",
        "omega" => "\u{03C9}",
        "Gamma" => "\u{0393}",
        "Delta" => "\u{0394}",
        "Theta" => "\u{0398}",
        "Lambda" => "\u{039B}",
        "Pi" => "\u{03A0}",
        "Sigma" => "\u{03A3}",
        "Phi" => "\u{03A6}",
        "Psi" => "\u{03A8}",
        "Omega" => "\u{03A9}",
        "sum" => "\u{2211}",
        "prod" => "\u{220F}",
        "int" => "\u{222B}",
        "infty" => "\u{221E}",
        "pm" => "\u{00B1}",
        "cdot" => "\u{00B7}",
        "times" => "\u{00D7}",
        "leq" => "\u{2264}",
        "geq" => "\u{2265}",
        "neq" => "\u{2260}",
        "approx" => "\u{2248}",
        "rightarrow" => "\u{2192}",
        "leftarrow" => "\u{2190}",
        "partial" => "\u{2202}",
        "nabla" => "\u{2207}",
        "forall" => "\u{2200}",
        "exists" => "\u{2203}",
        "in" => "\u{2208}",
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_simple_variable() {
        let compiler = EquationCompiler::default();
        let prims = compiler.compile("x");
        assert_eq!(prims.len(), 1);
        assert!(matches!(&prims[0], Primitive::Text { content, .. } if content == "x"));
    }

    #[test]
    fn compile_greek_letter() {
        let compiler = EquationCompiler::default();
        let prims = compiler.compile("\\alpha");
        assert_eq!(prims.len(), 1);
        assert!(matches!(&prims[0], Primitive::Text { content, .. } if content == "\u{03B1}"));
    }

    #[test]
    fn compile_superscript() {
        let compiler = EquationCompiler::default();
        let prims = compiler.compile("x^2");
        assert_eq!(prims.len(), 2);
        if let (
            Primitive::Text { y: y1, .. },
            Primitive::Text {
                y: y2, font_size, ..
            },
        ) = (&prims[0], &prims[1])
        {
            assert!(y2 < y1, "superscript should be above base");
            assert!(*font_size < 16.0, "superscript should be smaller");
        }
    }

    #[test]
    fn compile_subscript() {
        let compiler = EquationCompiler::default();
        let prims = compiler.compile("x_i");
        assert_eq!(prims.len(), 2);
        if let (Primitive::Text { y: y1, .. }, Primitive::Text { y: y2, .. }) =
            (&prims[0], &prims[1])
        {
            assert!(y2 > y1, "subscript should be below base");
        }
    }

    #[test]
    fn compile_fraction() {
        let compiler = EquationCompiler::default();
        let prims = compiler.compile("\\frac{a}{b}");
        let text_count = prims
            .iter()
            .filter(|p| matches!(p, Primitive::Text { .. }))
            .count();
        let line_count = prims
            .iter()
            .filter(|p| matches!(p, Primitive::Line { .. }))
            .count();
        assert!(
            text_count >= 2,
            "should have numerator and denominator text"
        );
        assert_eq!(line_count, 1, "should have one fraction bar");
    }

    #[test]
    fn compile_sqrt() {
        let compiler = EquationCompiler::default();
        let prims = compiler.compile("\\sqrt{x}");
        let has_radical = prims
            .iter()
            .any(|p| matches!(p, Primitive::Text { content, .. } if content.contains('\u{221A}')));
        assert!(has_radical, "should contain radical symbol");
    }

    #[test]
    fn compile_complex_expression() {
        let compiler = EquationCompiler::new(14.0, Color::WHITE).at(100.0, 200.0);
        let prims = compiler.compile("E = mc^2");
        assert!(prims.len() >= 4, "should produce multiple primitives");
    }

    #[test]
    fn compile_sum_with_limits() {
        let compiler = EquationCompiler::default();
        let prims = compiler.compile("\\sum_{i=0}^{n}");
        assert!(!prims.is_empty());
        let has_sum = prims
            .iter()
            .any(|p| matches!(p, Primitive::Text { content, .. } if content.contains('\u{2211}')));
        assert!(has_sum, "should contain sum symbol");
    }

    #[test]
    fn latex_symbol_coverage() {
        assert_eq!(latex_symbol("alpha"), "\u{03B1}");
        assert_eq!(latex_symbol("pi"), "\u{03C0}");
        assert_eq!(latex_symbol("sum"), "\u{2211}");
        assert_eq!(latex_symbol("int"), "\u{222B}");
        assert_eq!(latex_symbol("infty"), "\u{221E}");
        assert_eq!(latex_symbol("unknown"), "?");
    }
}
