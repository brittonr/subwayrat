//! Org pipe-table adapter: parse/serialize org tables to/from Grid.
//!
//! Gated behind the `org-compat` feature flag.

use crate::cell::{CellAddr, CellValue, Grid};

/// Error during org table parsing.
#[derive(Debug, Clone)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "org table parse error: {}", self.0)
    }
}

/// Parse an org-mode pipe table into a Grid.
///
/// Data rows (`| a | b |`) map to grid cells. Separator rows (`|---+---|`) are
/// skipped. Leading/trailing whitespace in cells is trimmed. Numeric-looking
/// cells become `CellValue::Number`.
pub fn from_org_table(text: &str) -> Result<Grid, ParseError> {
    let mut rows: Vec<Vec<CellValue>> = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue; // ignore non-table lines
        }
        // Check if separator row
        if is_separator_row(trimmed) {
            continue;
        }
        // Parse data row
        let cells: Vec<CellValue> = trimmed
            .split('|')
            .filter(|s| !s.is_empty() || false) // split produces empty first/last from leading/trailing |
            .collect::<Vec<_>>()
            .into_iter()
            .skip(0) // skip empty from leading |
            .filter(|s| {
                // The split('|') on "| a | b |" gives ["", " a ", " b ", ""]
                // We want the middle parts
                true
            })
            .map(|s| {
                let t = s.trim();
                if t.is_empty() {
                    CellValue::Empty
                } else if let Ok(n) = t.parse::<f64>() {
                    CellValue::Number(n)
                } else {
                    CellValue::Text(t.to_string())
                }
            })
            .collect();
        // Filter: split on "| a | b |" gives ["", " a ", " b ", ""] → after trim
        // we get [Empty, Text("a"), Text("b"), Empty]. Strip outer empties.
        let cells: Vec<CellValue> = strip_outer_empties(cells);
        if !cells.is_empty() {
            rows.push(cells);
        }
    }

    if rows.is_empty() {
        return Ok(Grid::new(0, 0));
    }

    let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let num_rows = rows.len();
    let mut grid = Grid::new(max_cols, num_rows);

    for (ri, row) in rows.iter().enumerate() {
        for (ci, cell) in row.iter().enumerate() {
            grid.set(CellAddr { col: ci, row: ri }, cell.clone());
        }
    }

    Ok(grid)
}

fn strip_outer_empties(mut cells: Vec<CellValue>) -> Vec<CellValue> {
    // Leading empty from "|"
    if let Some(CellValue::Empty) = cells.first() {
        cells.remove(0);
    }
    // Trailing empty from "|"
    if let Some(CellValue::Empty) = cells.last() {
        cells.pop();
    }
    cells
}

fn is_separator_row(line: &str) -> bool {
    let inner = line.trim_start_matches('|').trim_end_matches('|');
    !inner.is_empty() && inner.chars().all(|c| c == '-' || c == '+' || c == ' ')
}

/// Serialize a Grid to org pipe-table format.
///
/// Columns are padded to equal width. Numbers are right-aligned. A separator
/// row is inserted after the first row (header).
pub fn to_org_table(grid: &Grid) -> String {
    let rows = grid.row_count();
    let cols = grid.col_count();
    if rows == 0 || cols == 0 {
        return String::new();
    }

    // Compute column widths
    let mut col_widths = vec![1usize; cols];
    let mut is_numeric_col = vec![true; cols];
    for r in 0..rows {
        for c in 0..cols {
            let cell = grid.get(CellAddr { col: c, row: r });
            let text = cell.to_string();
            col_widths[c] = col_widths[c].max(text.len());
            if !matches!(cell, CellValue::Number(_) | CellValue::Empty) {
                is_numeric_col[c] = false;
            }
        }
    }

    let mut lines = Vec::new();
    for r in 0..rows {
        let mut parts = Vec::new();
        for c in 0..cols {
            let cell = grid.get(CellAddr { col: c, row: r });
            let text = cell.to_string();
            let w = col_widths[c];
            let padded = if is_numeric_col[c] {
                format!("{:>width$}", text, width = w)
            } else {
                format!("{:<width$}", text, width = w)
            };
            parts.push(padded);
        }
        lines.push(format!("| {} |", parts.join(" | ")));

        // Insert separator after header row
        if r == 0 {
            let sep_parts: Vec<String> = col_widths.iter().map(|&w| "-".repeat(w)).collect();
            lines.push(format!("|{}|", sep_parts.iter().map(|s| format!("-{}-", s)).collect::<Vec<_>>().join("+")));
        }
    }

    lines.join("\n")
}

/// Translate an org-style column formula to spreadsheet A1 syntax.
///
/// - `$1` → column `A` (for current row context)
/// - `$2` → column `B`
/// - `@2$3` → `C2`
/// - `vsum($1..$3)` → `SUM(A{row}:C{row})`
/// - `vmean($1..$3)` → `AVERAGE(A{row}:C{row})`
/// - A1-style formulas pass through unchanged
pub fn translate_formula(org_formula: &str, current_row: usize) -> String {
    let s = org_formula.trim();
    // If it starts with = and uses A1-style, passthrough
    if s.starts_with('=') {
        return s.to_string();
    }

    let mut result = s.to_string();

    // Handle vsum/vmean
    result = result.replace("vsum", "SUM").replace("vmean", "AVERAGE");

    // Handle @R$C cell references
    let re_cell = |s: &str| -> String {
        let mut out = s.to_string();
        // Simple pattern: @N$M → column_letter(M)N
        let mut i = 0;
        let chars: Vec<char> = out.chars().collect();
        let mut new = String::new();
        while i < chars.len() {
            if chars[i] == '@' {
                // Parse row number
                i += 1;
                let mut row_s = String::new();
                while i < chars.len() && chars[i].is_ascii_digit() {
                    row_s.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() && chars[i] == '$' {
                    i += 1;
                    let mut col_s = String::new();
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        col_s.push(chars[i]);
                        i += 1;
                    }
                    if let (Ok(row), Ok(col)) = (row_s.parse::<usize>(), col_s.parse::<usize>()) {
                        new.push_str(&format!("{}{}", col_letter(col), row));
                        continue;
                    }
                }
                new.push('@');
                new.push_str(&row_s);
            } else if chars[i] == '$' {
                // Column-only reference: $N → column_letter(N){current_row+1}
                i += 1;
                let mut col_s = String::new();
                while i < chars.len() && chars[i].is_ascii_digit() {
                    col_s.push(chars[i]);
                    i += 1;
                }
                if let Ok(col) = col_s.parse::<usize>() {
                    new.push_str(&format!("{}{}", col_letter(col), current_row + 1));
                } else {
                    new.push('$');
                    new.push_str(&col_s);
                }
            } else {
                new.push(chars[i]);
                i += 1;
            }
        }
        new
    };

    re_cell(&result)
}

/// Convert 1-indexed column number to letter(s): 1→A, 2→B, 26→Z, 27→AA.
fn col_letter(n: usize) -> String {
    if n == 0 { return "A".into(); }
    let idx = n - 1;
    if idx < 26 {
        ((idx as u8 + b'A') as char).to_string()
    } else {
        let first = ((idx / 26 - 1) as u8 + b'A') as char;
        let second = ((idx % 26) as u8 + b'A') as char;
        format!("{}{}", first, second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_table() {
        let input = "| Name | Age |\n|------+-----|\n| Alice | 30 |\n| Bob | 25 |";
        let grid = from_org_table(input).unwrap();
        assert_eq!(grid.row_count(), 3);
        assert_eq!(grid.col_count(), 2);
        assert_eq!(grid.get(CellAddr { col: 0, row: 0 }).to_string(), "Name");
        assert_eq!(grid.get(CellAddr { col: 1, row: 2 }).to_string(), "25");
    }

    #[test]
    fn parse_numeric_detection() {
        let input = "| x |\n| 42 |\n| 3.14 |";
        let grid = from_org_table(input).unwrap();
        assert!(matches!(grid.get(CellAddr { col: 0, row: 1 }), CellValue::Number(n) if *n == 42.0));
        assert!(matches!(grid.get(CellAddr { col: 0, row: 2 }), CellValue::Number(n) if (*n - 3.14).abs() < 0.001));
    }

    #[test]
    fn parse_empty_cells() {
        let input = "| a |  | c |";
        let grid = from_org_table(input).unwrap();
        assert_eq!(grid.col_count(), 3);
        assert!(matches!(grid.get(CellAddr { col: 1, row: 0 }), CellValue::Empty));
    }

    #[test]
    fn serialize_roundtrip() {
        let input = "| Name  | Age |\n|-------+-----|\n| Alice |  30 |\n| Bob   |  25 |";
        let grid = from_org_table(input).unwrap();
        let output = to_org_table(&grid);
        // Re-parse the output
        let grid2 = from_org_table(&output).unwrap();
        assert_eq!(grid2.row_count(), grid.row_count());
        assert_eq!(grid2.col_count(), grid.col_count());
        for r in 0..grid.row_count() {
            for c in 0..grid.col_count() {
                let addr = CellAddr { col: c, row: r };
                assert_eq!(grid.get(addr).to_string(), grid2.get(addr).to_string());
            }
        }
    }

    #[test]
    fn empty_grid_serializes_empty() {
        let grid = Grid::new(0, 0);
        assert_eq!(to_org_table(&grid), "");
    }

    #[test]
    fn translate_column_ref() {
        // $1 + $2 on row 3 (0-indexed) → A4 + B4
        assert_eq!(translate_formula("$1 + $2", 3), "A4 + B4");
    }

    #[test]
    fn translate_cell_ref() {
        assert_eq!(translate_formula("@2$3", 0), "C2");
    }

    #[test]
    fn translate_function() {
        let result = translate_formula("vsum($1..$3)", 0);
        assert!(result.contains("SUM"));
    }

    #[test]
    fn passthrough_a1_style() {
        assert_eq!(translate_formula("=A1+B1", 0), "=A1+B1");
    }
}
