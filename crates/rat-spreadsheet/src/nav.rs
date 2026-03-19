//! Grid navigation: cursor movement, scrolling, and selection.
//!
//! Provides functions for moving the cursor within the grid, handling
//! Tab/Shift+Tab wrapping, Home/End/Page navigation, viewport scrolling,
//! and rectangular cell selection.

use crate::cell::{CellAddr, CellRange, CellValue, Grid};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorState {
    pub position: CellAddr,
    pub anchor: Option<CellAddr>,
}

impl CursorState {
    pub fn new() -> Self {
        Self {
            position: CellAddr { col: 0, row: 0 },
            anchor: None,
        }
    }
}

impl Default for CursorState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    None,
    Range(CellRange),
}

impl Selection {
    pub fn selected_range(cursor: &CursorState) -> Selection {
        if let Some(anchor) = cursor.anchor {
            let start = CellAddr {
                col: anchor.col.min(cursor.position.col),
                row: anchor.row.min(cursor.position.row),
            };
            let end = CellAddr {
                col: anchor.col.max(cursor.position.col),
                row: anchor.row.max(cursor.position.row),
            };
            Selection::Range(CellRange { start, end })
        } else {
            Selection::None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollState {
    pub offset_col: usize,
    pub offset_row: usize,
    pub visible_cols: usize,
    pub visible_rows: usize,
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            offset_col: 0,
            offset_row: 0,
            visible_cols: 10,
            visible_rows: 20,
        }
    }

    pub fn ensure_visible(&mut self, addr: CellAddr) {
        // Ensure column is visible
        if addr.col < self.offset_col {
            self.offset_col = addr.col;
        } else if addr.col >= self.offset_col + self.visible_cols {
            self.offset_col = addr.col.saturating_sub(self.visible_cols - 1);
        }

        // Ensure row is visible
        if addr.row < self.offset_row {
            self.offset_row = addr.row;
        } else if addr.row >= self.offset_row + self.visible_rows {
            self.offset_row = addr.row.saturating_sub(self.visible_rows - 1);
        }
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

// Navigation functions
pub fn move_up(cursor: &mut CursorState, _grid: &Grid) {
    cursor.position.row = cursor.position.row.saturating_sub(1);
}

pub fn move_down(cursor: &mut CursorState, grid: &Grid) {
    if grid.row_count() > 0 {
        cursor.position.row = (cursor.position.row + 1).min(grid.row_count() - 1);
    }
}

pub fn move_left(cursor: &mut CursorState, _grid: &Grid) {
    cursor.position.col = cursor.position.col.saturating_sub(1);
}

pub fn move_right(cursor: &mut CursorState, grid: &Grid) {
    if grid.col_count() > 0 {
        cursor.position.col = (cursor.position.col + 1).min(grid.col_count() - 1);
    }
}

pub fn move_home(cursor: &mut CursorState, _grid: &Grid) {
    cursor.position.col = 0;
}

pub fn move_end(cursor: &mut CursorState, grid: &Grid) {
    if grid.col_count() > 0 {
        cursor.position.col = grid.col_count() - 1;
    }
}

pub fn move_home_all(cursor: &mut CursorState, _grid: &Grid) {
    cursor.position = CellAddr { col: 0, row: 0 };
}

pub fn move_end_all(cursor: &mut CursorState, grid: &Grid) {
    let mut last_col = 0;
    let mut last_row = 0;
    
    // Scan grid to find last cell with content
    for row in 0..grid.row_count() {
        for col in 0..grid.col_count() {
            let addr = CellAddr { col, row };
            if !matches!(grid.get(addr), CellValue::Empty) {
                last_row = last_row.max(row);
                last_col = last_col.max(col);
            }
        }
    }
    
    cursor.position = CellAddr { col: last_col, row: last_row };
}

pub fn move_page_up(cursor: &mut CursorState, _grid: &Grid, visible_rows: usize) {
    cursor.position.row = cursor.position.row.saturating_sub(visible_rows);
}

pub fn move_page_down(cursor: &mut CursorState, grid: &Grid, visible_rows: usize) {
    let max_row = if grid.row_count() > 0 { grid.row_count() - 1 } else { 0 };
    cursor.position.row = (cursor.position.row + visible_rows).min(max_row);
}

pub fn move_tab(cursor: &mut CursorState, grid: &Grid) {
    let max_col = if grid.col_count() > 0 { grid.col_count() - 1 } else { 0 };
    let max_row = if grid.row_count() > 0 { grid.row_count() - 1 } else { 0 };
    
    if cursor.position.col < max_col {
        cursor.position.col += 1;
    } else if cursor.position.row < max_row {
        cursor.position.col = 0;
        cursor.position.row += 1;
    }
    // If at last cell, stay there
}

pub fn move_tab_back(cursor: &mut CursorState, grid: &Grid) {
    let max_col = if grid.col_count() > 0 { grid.col_count() - 1 } else { 0 };
    
    if cursor.position.col > 0 {
        cursor.position.col -= 1;
    } else if cursor.position.row > 0 {
        cursor.position.col = max_col;
        cursor.position.row -= 1;
    }
    // If at first cell, stay there
}

// Selection functions
pub fn start_selection(cursor: &mut CursorState) {
    if cursor.anchor.is_none() {
        cursor.anchor = Some(cursor.position);
    }
}

pub fn clear_selection(cursor: &mut CursorState) {
    cursor.anchor = None;
}

pub fn get_selection(cursor: &CursorState) -> Selection {
    Selection::selected_range(cursor)
}

// Mouse hit test
pub fn cell_at_pixel(
    x: u16,
    y: u16,
    scroll: &ScrollState,
    col_widths: &[u16],
    row_height: u16,
    header_width: u16,
    header_height: u16,
) -> Option<CellAddr> {
    // Check if click is on headers
    if x < header_width || y < header_height {
        return None;
    }
    
    // Adjust coordinates for headers
    let grid_x = x - header_width;
    let grid_y = y - header_height;
    
    // Calculate row
    let row = scroll.offset_row + (grid_y / row_height) as usize;
    
    // Calculate column
    let mut col_offset = 0;
    let mut col = scroll.offset_col;
    
    while col_offset < grid_x {
        if col >= col_widths.len() {
            return None;
        }
        
        if col_offset + col_widths[col] > grid_x {
            break;
        }
        
        col_offset += col_widths[col];
        col += 1;
    }
    
    Some(CellAddr { col, row })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_state_new() {
        let cursor = CursorState::new();
        assert_eq!(cursor.position, CellAddr { col: 0, row: 0 });
        assert_eq!(cursor.anchor, None);
    }

    #[test]
    fn test_move_up_clamping() {
        let mut cursor = CursorState::new();
        let grid = Grid::new(5, 5);
        
        // Start at A1 (0,0), moving up should stay at A1
        move_up(&mut cursor, &grid);
        assert_eq!(cursor.position, CellAddr { col: 0, row: 0 });
    }

    #[test]
    fn test_move_down_clamping() {
        let mut cursor = CursorState::new();
        let grid = Grid::new(5, 3);
        
        cursor.position = CellAddr { col: 0, row: 2 }; // Last row
        move_down(&mut cursor, &grid);
        assert_eq!(cursor.position, CellAddr { col: 0, row: 2 }); // Should stay
    }

    #[test]
    fn test_move_left_clamping() {
        let mut cursor = CursorState::new();
        let grid = Grid::new(5, 5);
        
        // Start at A1 (0,0), moving left should stay at A1
        move_left(&mut cursor, &grid);
        assert_eq!(cursor.position, CellAddr { col: 0, row: 0 });
    }

    #[test]
    fn test_move_right_clamping() {
        let mut cursor = CursorState::new();
        let grid = Grid::new(3, 5);
        
        cursor.position = CellAddr { col: 2, row: 0 }; // Last column
        move_right(&mut cursor, &grid);
        assert_eq!(cursor.position, CellAddr { col: 2, row: 0 }); // Should stay
    }

    #[test]
    fn test_tab_wrapping() {
        let mut cursor = CursorState::new();
        let grid = Grid::new(3, 3);
        
        // Move to end of first row
        cursor.position = CellAddr { col: 2, row: 0 };
        move_tab(&mut cursor, &grid);
        assert_eq!(cursor.position, CellAddr { col: 0, row: 1 });
    }

    #[test]
    fn test_tab_back_wrapping() {
        let mut cursor = CursorState::new();
        let grid = Grid::new(3, 3);
        
        // Move to start of second row
        cursor.position = CellAddr { col: 0, row: 1 };
        move_tab_back(&mut cursor, &grid);
        assert_eq!(cursor.position, CellAddr { col: 2, row: 0 });
    }

    #[test]
    fn test_home_end_navigation() {
        let mut cursor = CursorState::new();
        let grid = Grid::new(5, 5);
        
        cursor.position = CellAddr { col: 3, row: 2 };
        
        move_home(&mut cursor, &grid);
        assert_eq!(cursor.position, CellAddr { col: 0, row: 2 });
        
        move_end(&mut cursor, &grid);
        assert_eq!(cursor.position, CellAddr { col: 4, row: 2 });
    }

    #[test]
    fn test_page_up_down() {
        let mut cursor = CursorState::new();
        let grid = Grid::new(5, 25);
        
        cursor.position = CellAddr { col: 2, row: 10 };
        
        move_page_up(&mut cursor, &grid, 5);
        assert_eq!(cursor.position, CellAddr { col: 2, row: 5 });
        
        move_page_down(&mut cursor, &grid, 10);
        assert_eq!(cursor.position, CellAddr { col: 2, row: 15 });
        
        // Test clamping at top
        cursor.position = CellAddr { col: 2, row: 2 };
        move_page_up(&mut cursor, &grid, 5);
        assert_eq!(cursor.position, CellAddr { col: 2, row: 0 });
    }

    #[test]
    fn test_scroll_state_ensure_visible() {
        let mut scroll = ScrollState::new();
        scroll.visible_cols = 5;
        scroll.visible_rows = 10;
        
        // Test scrolling right
        scroll.ensure_visible(CellAddr { col: 15, row: 5 });
        assert_eq!(scroll.offset_col, 11); // 15 - 5 + 1
        
        // Test scrolling left
        scroll.ensure_visible(CellAddr { col: 5, row: 5 });
        assert_eq!(scroll.offset_col, 5);
        
        // Test scrolling down
        scroll.ensure_visible(CellAddr { col: 5, row: 25 });
        assert_eq!(scroll.offset_row, 16); // 25 - 10 + 1
        
        // Test scrolling up
        scroll.ensure_visible(CellAddr { col: 5, row: 5 });
        assert_eq!(scroll.offset_row, 5);
    }

    #[test]
    fn test_selection_start_clear() {
        let mut cursor = CursorState::new();
        cursor.position = CellAddr { col: 2, row: 3 };
        
        start_selection(&mut cursor);
        assert_eq!(cursor.anchor, Some(CellAddr { col: 2, row: 3 }));
        
        clear_selection(&mut cursor);
        assert_eq!(cursor.anchor, None);
    }

    #[test]
    fn test_selection_range() {
        let mut cursor = CursorState::new();
        cursor.position = CellAddr { col: 2, row: 3 };
        cursor.anchor = Some(CellAddr { col: 0, row: 1 });
        
        let selection = get_selection(&cursor);
        match selection {
            Selection::Range(range) => {
                assert_eq!(range.start, CellAddr { col: 0, row: 1 });
                assert_eq!(range.end, CellAddr { col: 2, row: 3 });
            }
            _ => panic!("Expected range selection"),
        }
    }

    #[test]
    fn test_selection_none() {
        let cursor = CursorState::new();
        let selection = get_selection(&cursor);
        assert_eq!(selection, Selection::None);
    }

    #[test]
    fn test_mouse_hit_test_headers() {
        let scroll = ScrollState::new();
        let col_widths = vec![80, 80, 80];
        
        // Click on row header
        assert_eq!(
            cell_at_pixel(10, 50, &scroll, &col_widths, 20, 50, 30),
            None
        );
        
        // Click on column header
        assert_eq!(
            cell_at_pixel(100, 10, &scroll, &col_widths, 20, 50, 30),
            None
        );
    }

    #[test]
    fn test_mouse_hit_test_grid() {
        let scroll = ScrollState::new();
        let col_widths = vec![80, 80, 80];
        
        // Click on first cell (after headers)
        assert_eq!(
            cell_at_pixel(60, 40, &scroll, &col_widths, 20, 50, 30),
            Some(CellAddr { col: 0, row: 0 })
        );
        
        // Click on second column, third row
        assert_eq!(
            cell_at_pixel(140, 90, &scroll, &col_widths, 20, 50, 30),
            Some(CellAddr { col: 1, row: 3 })
        );
    }

    #[test]
    fn test_move_end_all_with_content() {
        let mut cursor = CursorState::new();
        let mut grid = Grid::new(5, 5);
        
        // Add content at various positions
        grid.set(CellAddr { col: 3, row: 2 }, CellValue::Number(42.0));
        grid.set(CellAddr { col: 1, row: 4 }, CellValue::Text("test".to_string()));
        
        move_end_all(&mut cursor, &grid);
        assert_eq!(cursor.position, CellAddr { col: 3, row: 4 });
    }

    #[test]
    fn test_move_end_all_empty_grid() {
        let mut cursor = CursorState::new();
        let grid = Grid::new(0, 0);
        
        move_end_all(&mut cursor, &grid);
        assert_eq!(cursor.position, CellAddr { col: 0, row: 0 });
    }
}