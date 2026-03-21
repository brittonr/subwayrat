//! Rendering: widget, state, and StatefulWidget implementation.
//!
//! Contains the primary types for using the spreadsheet:
//! - [`Spreadsheet`] - the ratatui widget (implements `StatefulWidget`)
//! - [`SpreadsheetState`] - all mutable state (grid, cursor, edit, scroll, formulas)
//! - [`SpreadsheetStyle`] - visual styling configuration
//! - [`EditState`] - inline cell editing state

use crate::cell::{CellAddr, CellRange, CellValue, Grid};
use crate::formula::{DependencyGraph, FunctionRegistry};
use crate::nav::{CursorState, ScrollState, Selection, get_selection};
use crate::edit_state::EditState;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{StatefulWidget, Widget, Block};

/// Styling configuration for the spreadsheet widget
#[derive(Debug, Clone)]
pub struct SpreadsheetStyle {
    /// Style for column/row headers
    pub header_style: Style,
    /// Style for the current cell cursor
    pub cursor_style: Style,
    /// Style for selected range highlight
    pub selection_style: Style,
    /// Default cell style
    pub cell_style: Style,
    /// Style for cell in edit mode
    pub edit_style: Style,
}

impl Default for SpreadsheetStyle {
    fn default() -> Self {
        Self {
            header_style: Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            cursor_style: Style::default()
                .bg(Color::Blue)
                .fg(Color::White),
            selection_style: Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black),
            cell_style: Style::default()
                .fg(Color::White)
                .bg(Color::Black),
            edit_style: Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black),
        }
    }
}



/// Type aliases for complex callback types
type StyleCallback = Box<dyn Fn(CellAddr, &CellValue) -> Option<Style>>;
type ValidatorCallback = Box<dyn Fn(&str) -> Result<(), String>>;

/// Parameters for cell rendering position and size
struct CellRenderParams {
    x: u16,
    y: u16,
    width: u16,
}

/// Complete spreadsheet state containing all mutable data
pub struct SpreadsheetState {
    // -- Data model --
    /// The data grid
    pub grid: Grid,
    /// Edit state for cell editing
    pub edit: EditState,
    /// Dependency graph for formula recalculation
    pub dep_graph: DependencyGraph,
    /// Function registry for formula evaluation
    pub fn_registry: FunctionRegistry,
    /// Simple undo - last changed cell and its previous value
    pub last_undo: Option<(CellAddr, CellValue)>,
    /// Per-column validation callbacks
    pub validators: std::collections::HashMap<usize, ValidatorCallback>,

    // -- Navigation --
    /// Cursor navigation state
    pub cursor: CursorState,
    /// Scroll state for viewport
    pub scroll: ScrollState,

    // -- Visual layout --
    /// Custom column widths
    pub col_widths: Vec<u16>,
    /// Default column width
    pub default_col_width: u16,
    /// Minimum column width
    pub min_col_width: u16,
    /// Number of frozen rows
    pub frozen_rows: usize,
    /// Number of frozen columns  
    pub frozen_cols: usize,
    /// Optional per-cell styling callback
    pub style_callback: Option<StyleCallback>,
}

impl SpreadsheetState {
    /// Create a new spreadsheet state
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            // -- Data model --
            grid: Grid::new(cols, rows),
            edit: EditState::new(),
            dep_graph: DependencyGraph::new(),
            fn_registry: FunctionRegistry::new(),
            last_undo: None,
            validators: std::collections::HashMap::new(),

            // -- Navigation --
            cursor: CursorState::new(),
            scroll: ScrollState::new(),

            // -- Visual layout --
            col_widths: Vec::new(),
            default_col_width: 10,
            min_col_width: 3,
            frozen_rows: 0,
            frozen_cols: 0,
            style_callback: None,
        }
    }

    /// Get the width of a specific column
    pub fn col_width(&self, col: usize) -> u16 {
        self.col_widths
            .get(col)
            .copied()
            .unwrap_or(self.default_col_width)
            .max(self.min_col_width)
    }

    /// Set the width of a specific column
    pub fn set_col_width(&mut self, col: usize, width: u16) {
        // Extend the vector if needed
        while self.col_widths.len() <= col {
            self.col_widths.push(self.default_col_width);
        }
        self.col_widths[col] = width.max(self.min_col_width);
    }

    /// Set a styling callback for per-cell styling
    pub fn set_style_callback(&mut self, f: impl Fn(CellAddr, &CellValue) -> Option<Style> + 'static) {
        self.style_callback = Some(Box::new(f));
    }

    /// Register a validation callback for a column.
    /// The callback receives the raw input string and returns Ok(()) or Err(message).
    pub fn set_column_validator(&mut self, col: usize, f: impl Fn(&str) -> Result<(), String> + 'static) {
        self.validators.insert(col, Box::new(f));
    }

    /// Validate input for a given column. Returns Ok(()) if no validator or validation passes.
    pub fn validate_input(&self, col: usize, input: &str) -> Result<(), String> {
        match self.validators.get(&col) {
            Some(validator) => validator(input),
            None => Ok(()),
        }
    }
}

/// The main spreadsheet widget
pub struct Spreadsheet<'a> {
    block: Option<Block<'a>>,
    style: SpreadsheetStyle,
}

impl<'a> Spreadsheet<'a> {
    /// Create a new spreadsheet widget
    pub fn new() -> Self {
        Self {
            block: None,
            style: SpreadsheetStyle::default(),
        }
    }

    /// Set the border block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the style
    pub fn style(mut self, style: SpreadsheetStyle) -> Self {
        self.style = style;
        self
    }
}

impl<'a> Default for Spreadsheet<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> StatefulWidget for Spreadsheet<'a> {
    type State = SpreadsheetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Apply block if present
        let area = if let Some(ref block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if area.width == 0 || area.height == 0 {
            return;
        }

        // Calculate layout dimensions
        let row_number_width = calculate_row_number_width(state.grid.row_count());
        let header_height = 1;

        // Available area for cells
        let cells_area = Rect {
            x: area.x + row_number_width,
            y: area.y + header_height,
            width: area.width.saturating_sub(row_number_width),
            height: area.height.saturating_sub(header_height),
        };

        // Update scroll state visible dimensions
        state.scroll.visible_cols = cells_area.width as usize;
        state.scroll.visible_rows = cells_area.height as usize;

        // Draw column headers
        self.render_column_headers(area, buf, state, row_number_width);

        // Draw row numbers
        self.render_row_numbers(area, buf, state, header_height);

        // Draw cells
        self.render_cells(cells_area, buf, state);
    }
}

impl<'a> Spreadsheet<'a> {
    fn render_column_headers(&self, area: Rect, buf: &mut Buffer, state: &SpreadsheetState, row_number_width: u16) {
        let header_y = area.y;
        let mut x = area.x + row_number_width;
        let max_x = area.x + area.width;

        for col in state.scroll.offset_col..state.grid.col_count() {
            let col_width = state.col_width(col);
            
            if x >= max_x {
                break;
            }

            let header_text = col_name(col);
            let available_width = (max_x - x).min(col_width);
            
            if available_width > 0 {
                let truncated = truncate_text(&header_text, available_width as usize);
                
                for (i, ch) in truncated.chars().enumerate() {
                    if x + (i as u16) < max_x {
                        buf[(x + (i as u16), header_y)]
                            .set_char(ch)
                            .set_style(self.style.header_style);
                    }
                }

                // Fill remaining space in header cell
                for i in truncated.len() as u16..available_width {
                    if x + i < max_x {
                        buf[(x + i, header_y)]
                            .set_char(' ')
                            .set_style(self.style.header_style);
                    }
                }
            }

            x += col_width;
        }

        // Fill remaining header space
        while x < max_x {
            buf[(x, header_y)]
                .set_char(' ')
                .set_style(self.style.header_style);
            x += 1;
        }
    }

    fn render_row_numbers(&self, area: Rect, buf: &mut Buffer, state: &SpreadsheetState, header_height: u16) {
        let max_y = area.y + area.height;
        let row_number_width = calculate_row_number_width(state.grid.row_count());

        for row in state.scroll.offset_row..state.grid.row_count() {
            let y = area.y + header_height + (row - state.scroll.offset_row) as u16;
            
            if y >= max_y {
                break;
            }

            let row_text = (row + 1).to_string();
            let truncated = truncate_text(&row_text, row_number_width as usize);
            
            // Right-align the row number
            let start_x = area.x + row_number_width - truncated.len() as u16;
            
            for (i, ch) in truncated.chars().enumerate() {
                let x = start_x + i as u16;
                if x < area.x + row_number_width {
                    buf[(x, y)]
                        .set_char(ch)
                        .set_style(self.style.header_style);
                }
            }

            // Fill remaining space in row number area
            for x in area.x..start_x {
                buf[(x, y)]
                    .set_char(' ')
                    .set_style(self.style.header_style);
            }
        }

        // Fill row number column for header row
        for x in area.x..area.x + row_number_width {
            buf[(x, area.y)]
                .set_char(' ')
                .set_style(self.style.header_style);
        }
    }

    fn render_cells(&self, area: Rect, buf: &mut Buffer, state: &SpreadsheetState) {
        let max_x = area.x + area.width;
        let max_y = area.y + area.height;

        for row in state.scroll.offset_row..state.grid.row_count() {
            let y = area.y + (row - state.scroll.offset_row) as u16;
            
            if y >= max_y {
                break;
            }

            let mut x = area.x;

            for col in state.scroll.offset_col..state.grid.col_count() {
                let col_width = state.col_width(col);
                
                if x >= max_x {
                    break;
                }

                let cell_addr = CellAddr { col, row };
                let cell_value = state.grid.get(cell_addr);
                let available_width = (max_x - x).min(col_width);
                
                if available_width > 0 {
                    self.render_cell(
                        buf, 
                        CellRenderParams { x, y, width: available_width }, 
                        cell_addr, 
                        cell_value, 
                        state
                    );
                }

                x += col_width;
            }

            // Fill remaining space in row
            while x < max_x {
                buf[(x, y)]
                    .set_char(' ')
                    .set_style(self.style.cell_style);
                x += 1;
            }
        }
    }

    fn render_cell(
        &self,
        buf: &mut Buffer,
        params: CellRenderParams,
        addr: CellAddr,
        value: &CellValue,
        state: &SpreadsheetState,
    ) {
        // Determine cell content and alignment
        let (content, right_align) = if state.edit.editing && addr == state.cursor.position {
            // Show edit buffer with cursor
            let mut display = state.edit.buffer.clone();
            if state.edit.cursor_pos <= display.len() {
                display.insert(state.edit.cursor_pos, '|'); // Visual cursor
            }
            (display, false)
        } else {
            format_cell_value(value)
        };

        // Determine style
        let mut style = self.style.cell_style;
        
        // Apply cursor highlighting
        if addr == state.cursor.position {
            if state.edit.editing {
                style = self.style.edit_style;
            } else {
                style = self.style.cursor_style;
            }
        }
        
        // Apply selection highlighting
        if let Selection::Range(range) = get_selection(&state.cursor) && is_in_range(addr, range) {
            style = self.style.selection_style;
        }
        
        // Apply custom styling callback if set
        if let Some(ref callback) = state.style_callback && let Some(custom_style) = callback(addr, value) {
            style = custom_style;
        }

        // Truncate content to fit
        let truncated = truncate_text(&content, params.width as usize);
        
        // Render the content with proper alignment
        let start_pos = if right_align && truncated.len() < params.width as usize {
            params.width as usize - truncated.len()
        } else {
            0
        };

        // Fill cell background
        for i in 0..params.width as usize {
            let cell_x = params.x + i as u16;
            let ch = if i >= start_pos && i - start_pos < truncated.len() {
                truncated.chars().nth(i - start_pos).unwrap_or(' ')
            } else {
                ' '
            };

            buf[(cell_x, params.y)]
                .set_char(ch)
                .set_style(style);
        }
    }
}

/// Convert a column index to Excel-style column name (A, B, ..., Z, AA, AB, ...)
fn col_name(col: usize) -> String {
    if col < 26 {
        ((col as u8 + b'A') as char).to_string()
    } else {
        let first = ((col / 26) as u8 - 1 + b'A') as char;
        let second = ((col % 26) as u8 + b'A') as char;
        format!("{}{}", first, second)
    }
}

/// Calculate width needed for row numbers
fn calculate_row_number_width(row_count: usize) -> u16 {
    if row_count == 0 {
        3 // Minimum width
    } else {
        row_count.to_string().len().max(3) as u16
    }
}

/// Format cell value for display
fn format_cell_value(value: &CellValue) -> (String, bool) {
    match value {
        CellValue::Empty => ("".to_string(), false),
        CellValue::Text(s) => (s.clone(), false),
        CellValue::Number(n) => (format!("{}", n), true), // Right-align numbers
        CellValue::Boolean(b) => (if *b { "TRUE" } else { "FALSE" }.to_string(), false),
        CellValue::Error(e) => (e.to_string(), false),
        CellValue::Formula { cached, .. } => format_cell_value(cached), // Show cached value
    }
}

/// Truncate text to fit within specified width
fn truncate_text(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    
    if text.len() <= max_width {
        text.to_string()
    } else {
        text.chars().take(max_width).collect()
    }
}

/// Check if a cell address is within a range
fn is_in_range(addr: CellAddr, range: CellRange) -> bool {
    let min_col = range.start.col.min(range.end.col);
    let max_col = range.start.col.max(range.end.col);
    let min_row = range.start.row.min(range.end.row);
    let max_row = range.start.row.max(range.end.row);
    
    addr.col >= min_col && addr.col <= max_col &&
    addr.row >= min_row && addr.row <= max_row
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::CellError;

    #[test]
    fn test_col_name() {
        assert_eq!(col_name(0), "A");
        assert_eq!(col_name(25), "Z");
        assert_eq!(col_name(26), "AA");
        assert_eq!(col_name(27), "AB");
        assert_eq!(col_name(51), "AZ");
        assert_eq!(col_name(52), "BA");
    }



    #[test]
    fn test_format_cell_value() {
        let (content, right_align) = format_cell_value(&CellValue::Empty);
        assert_eq!(content, "");
        assert!(!right_align);

        let (content, right_align) = format_cell_value(&CellValue::Number(42.5));
        assert_eq!(content, "42.5");
        assert!(right_align);

        let (content, right_align) = format_cell_value(&CellValue::Text("hello".to_string()));
        assert_eq!(content, "hello");
        assert!(!right_align);

        let (content, right_align) = format_cell_value(&CellValue::Boolean(true));
        assert_eq!(content, "TRUE");
        assert!(!right_align);

        let (content, right_align) = format_cell_value(&CellValue::Error(CellError::DivByZero));
        assert_eq!(content, "#DIV/0!");
        assert!(!right_align);
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("hello", 10), "hello");
        assert_eq!(truncate_text("hello world", 5), "hello");
        assert_eq!(truncate_text("test", 0), "");
    }

    #[test]
    fn test_is_in_range() {
        let range = CellRange {
            start: CellAddr { col: 1, row: 1 },
            end: CellAddr { col: 3, row: 3 },
        };

        assert!(is_in_range(CellAddr { col: 2, row: 2 }, range));
        assert!(is_in_range(CellAddr { col: 1, row: 1 }, range));
        assert!(is_in_range(CellAddr { col: 3, row: 3 }, range));
        assert!(!is_in_range(CellAddr { col: 0, row: 0 }, range));
        assert!(!is_in_range(CellAddr { col: 4, row: 4 }, range));
    }

    #[test]
    fn test_calculate_row_number_width() {
        assert_eq!(calculate_row_number_width(0), 3);
        assert_eq!(calculate_row_number_width(9), 3);
        assert_eq!(calculate_row_number_width(99), 3);
        assert_eq!(calculate_row_number_width(100), 3);
        assert_eq!(calculate_row_number_width(999), 3);
        assert_eq!(calculate_row_number_width(1000), 4);
    }
}