//! Cell model: addressing, value types, and grid storage.
//!
//! This module provides the core data types for the spreadsheet:
//! - [`CellAddr`] for A1-style cell addressing
//! - [`CellRange`] for rectangular cell ranges
//! - [`CellValue`] for cell content (text, numbers, formulas, etc.)
//! - [`Grid`] for the underlying 2D cell storage

use std::fmt;
use std::str::FromStr;

/// A cell address in the spreadsheet grid.
///
/// Uses 0-indexed (col, row) internally. Supports A1-style parsing and display,
/// where column A is 0, B is 1, ..., Z is 25, AA is 26, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellAddr {
    pub col: usize,
    pub row: usize,
}

impl FromStr for CellAddr {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(());
        }

        let mut chars = s.chars().peekable();

        // Parse column letters
        let mut col = 0;
        let mut has_letter = false;

        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_alphabetic() {
                let ch = chars.next().unwrap().to_ascii_uppercase();
                if has_letter {
                    // Second letter - convert first letter from 0-based to 1-based for calculation
                    col = (col + 1) * 26 + (ch as u8 - b'A') as usize;
                } else {
                    // First letter
                    col = (ch as u8 - b'A') as usize;
                    has_letter = true;
                }
            } else {
                break;
            }
        }

        if !has_letter {
            return Err(());
        }

        // Parse row number
        let row_str: String = chars.collect();
        if row_str.is_empty() {
            return Err(());
        }

        let row = row_str.parse::<usize>().map_err(|_| ())?;
        if row == 0 {
            return Err(());
        }

        Ok(CellAddr {
            col,
            row: row - 1, // Convert to 0-indexed
        })
    }
}

impl fmt::Display for CellAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let col_str = if self.col < 26 {
            ((self.col as u8 + b'A') as char).to_string()
        } else {
            let first = ((self.col / 26) as u8 - 1 + b'A') as char;
            let second = ((self.col % 26) as u8 + b'A') as char;
            format!("{}{}", first, second)
        };
        write!(f, "{}{}", col_str, self.row + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRange {
    pub start: CellAddr,
    pub end: CellAddr,
}

impl IntoIterator for CellRange {
    type Item = CellAddr;
    type IntoIter = CellRangeIter;

    fn into_iter(self) -> Self::IntoIter {
        let start_col = self.start.col;
        let start_row = self.start.row;
        CellRangeIter {
            range: self,
            current_col: start_col,
            current_row: start_row,
        }
    }
}

pub struct CellRangeIter {
    range: CellRange,
    current_col: usize,
    current_row: usize,
}

impl Iterator for CellRangeIter {
    type Item = CellAddr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row > self.range.end.row {
            return None;
        }

        let addr = CellAddr {
            col: self.current_col,
            row: self.current_row,
        };

        // Move to next position (row-major order)
        if self.current_col < self.range.end.col {
            self.current_col += 1;
        } else {
            self.current_col = self.range.start.col;
            self.current_row += 1;
        }

        Some(addr)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellError {
    DivByZero,
    ValueError,
    CycleError,
    RefError,
    ParseError,
}

impl fmt::Display for CellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            CellError::DivByZero => "#DIV/0!",
            CellError::ValueError => "#VALUE!",
            CellError::CycleError => "#CYCLE!",
            CellError::RefError => "#REF!",
            CellError::ParseError => "#PARSE!",
        };
        write!(f, "{}", code)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Empty,
    Text(String),
    Number(f64),
    Boolean(bool),
    Error(CellError),
    Formula {
        expr: String,
        cached: Box<CellValue>,
    },
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellValue::Empty => write!(f, ""),
            CellValue::Text(s) => write!(f, "{}", s),
            CellValue::Number(n) => write!(f, "{}", n),
            CellValue::Boolean(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            CellValue::Error(e) => write!(f, "{}", e),
            CellValue::Formula { cached, .. } => write!(f, "{}", cached),
        }
    }
}

pub struct Grid {
    cells: Vec<Vec<CellValue>>,
    empty: CellValue,
}

impl Grid {
    pub fn new(cols: usize, rows: usize) -> Self {
        let cells = vec![vec![CellValue::Empty; cols]; rows];
        Grid {
            cells,
            empty: CellValue::Empty,
        }
    }

    pub fn get(&self, addr: CellAddr) -> &CellValue {
        if addr.row >= self.cells.len()
            || (addr.row < self.cells.len() && addr.col >= self.cells[addr.row].len())
        {
            &self.empty
        } else {
            &self.cells[addr.row][addr.col]
        }
    }

    pub fn set(&mut self, addr: CellAddr, value: CellValue) {
        // Grow rows if needed
        while addr.row >= self.cells.len() {
            self.cells.push(Vec::new());
        }

        // Calculate current column count
        let current_cols = if self.cells.is_empty() {
            0
        } else {
            self.cells.iter().map(|row| row.len()).max().unwrap_or(0)
        };

        // Grow columns if needed
        if addr.col >= current_cols {
            let new_cols = addr.col + 1;
            for row in &mut self.cells {
                row.resize(new_cols, CellValue::Empty);
            }
        }

        self.cells[addr.row][addr.col] = value;
    }

    pub fn row_count(&self) -> usize {
        self.cells.len()
    }

    pub fn col_count(&self) -> usize {
        if self.cells.is_empty() {
            0
        } else {
            self.cells.iter().map(|row| row.len()).max().unwrap_or(0)
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }
}

impl crate::formula::Grid for Grid {
    fn get(&self, addr: CellAddr) -> &CellValue {
        self.get(addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celladdr_parsing() {
        assert_eq!(
            "A1".parse::<CellAddr>().unwrap(),
            CellAddr { col: 0, row: 0 }
        );
        assert_eq!(
            "B3".parse::<CellAddr>().unwrap(),
            CellAddr { col: 1, row: 2 }
        );
        assert_eq!(
            "AA1".parse::<CellAddr>().unwrap(),
            CellAddr { col: 26, row: 0 }
        );
        assert_eq!(
            "Z1".parse::<CellAddr>().unwrap(),
            CellAddr { col: 25, row: 0 }
        );

        // Invalid cases
        assert!("123".parse::<CellAddr>().is_err());
        assert!("".parse::<CellAddr>().is_err());
        assert!("1A".parse::<CellAddr>().is_err());
    }

    #[test]
    fn test_celladdr_display_roundtrip() {
        let cases = vec![
            CellAddr { col: 0, row: 0 },   // A1
            CellAddr { col: 1, row: 2 },   // B3
            CellAddr { col: 26, row: 0 },  // AA1
            CellAddr { col: 25, row: 0 },  // Z1
            CellAddr { col: 51, row: 99 }, // AZ100
        ];

        for addr in cases {
            let display = addr.to_string();
            let parsed = display.parse::<CellAddr>().unwrap();
            assert_eq!(addr, parsed, "Failed roundtrip for {}", display);
        }
    }

    #[test]
    fn test_cellrange_iteration() {
        let range = CellRange {
            start: CellAddr { col: 0, row: 0 }, // A1
            end: CellAddr { col: 1, row: 1 },   // B2
        };

        let cells: Vec<CellAddr> = range.into_iter().collect();
        assert_eq!(
            cells,
            vec![
                CellAddr { col: 0, row: 0 }, // A1
                CellAddr { col: 1, row: 0 }, // B1
                CellAddr { col: 0, row: 1 }, // A2
                CellAddr { col: 1, row: 1 }, // B2
            ]
        );
    }

    #[test]
    fn test_cellrange_single_cell() {
        let range = CellRange {
            start: CellAddr { col: 0, row: 0 }, // A1
            end: CellAddr { col: 0, row: 0 },   // A1
        };

        let cells: Vec<CellAddr> = range.into_iter().collect();
        assert_eq!(cells, vec![CellAddr { col: 0, row: 0 }]);
    }

    #[test]
    fn test_grid_new() {
        let grid = Grid::new(3, 2);
        assert_eq!(grid.row_count(), 2);
        assert_eq!(grid.col_count(), 3);
    }

    #[test]
    fn test_grid_get_set() {
        let mut grid = Grid::new(2, 2);

        let addr = CellAddr { col: 0, row: 0 };
        assert_eq!(grid.get(addr), &CellValue::Empty);

        grid.set(addr, CellValue::Number(42.0));
        assert_eq!(grid.get(addr), &CellValue::Number(42.0));
    }

    #[test]
    fn test_grid_dynamic_growth() {
        let mut grid = Grid::new(1, 1);

        let addr = CellAddr { col: 5, row: 3 };
        grid.set(addr, CellValue::Text("test".to_string()));

        assert_eq!(grid.row_count(), 4);
        assert_eq!(grid.col_count(), 6);
        assert_eq!(grid.get(addr), &CellValue::Text("test".to_string()));
    }

    #[test]
    fn test_grid_out_of_bounds_get() {
        let grid = Grid::new(2, 2);
        let addr = CellAddr { col: 10, row: 10 };
        assert_eq!(grid.get(addr), &CellValue::Empty);
    }

    #[test]
    fn test_grid_clear() {
        let mut grid = Grid::new(2, 2);
        grid.set(CellAddr { col: 0, row: 0 }, CellValue::Number(42.0));

        grid.clear();
        assert_eq!(grid.row_count(), 0);
        assert_eq!(grid.col_count(), 0);
    }

    #[test]
    fn test_cellvalue_display() {
        assert_eq!(CellValue::Empty.to_string(), "");
        assert_eq!(CellValue::Text("hello".to_string()).to_string(), "hello");
        assert_eq!(CellValue::Number(42.5).to_string(), "42.5");
        assert_eq!(CellValue::Boolean(true).to_string(), "TRUE");
        assert_eq!(CellValue::Boolean(false).to_string(), "FALSE");
        assert_eq!(
            CellValue::Error(CellError::DivByZero).to_string(),
            "#DIV/0!"
        );

        let formula = CellValue::Formula {
            expr: "=A1+B1".to_string(),
            cached: Box::new(CellValue::Number(10.0)),
        };
        assert_eq!(formula.to_string(), "10");
    }
}
