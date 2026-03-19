//! Event handling: unified action dispatch, edit lifecycle, copy/paste.
//!
//! The [`Action`] enum represents all input actions the spreadsheet can handle.
//! Call [`handle_action`] to process an action and update the [`SpreadsheetState`].
//! The [`Clipboard`] holds copied cell data for paste operations.

use crate::cell::{CellAddr, CellValue, CellError};
use crate::formula::{parse, evaluate_with_registry};
use crate::nav::{self, Selection, get_selection};
use crate::render::SpreadsheetState;

/// Actions the spreadsheet can perform in response to input
pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveHome,
    MoveEnd,
    MoveHomeAll,
    MoveEndAll,
    PageUp,
    PageDown,
    Tab,
    TabBack,
    /// Start or continue editing - if a character is provided, begin typing
    EnterEdit(Option<char>),
    /// Commit the current edit
    CommitEdit,
    /// Cancel the current edit
    CancelEdit,
    /// Undo last edit
    Undo,
    /// Copy selected range
    Copy,
    /// Paste from clipboard
    Paste,
    /// Begin selection (Shift held)
    StartSelection,
    /// Clear selection
    ClearSelection,
    /// Mouse click at grid position
    ClickCell(CellAddr),
    /// Delete cell content
    DeleteContent,
    /// Type a character in edit mode
    TypeChar(char),
    /// Backspace in edit mode
    Backspace,
    /// Delete forward in edit mode
    Delete,
    /// Move edit cursor left
    EditCursorLeft,
    /// Move edit cursor right
    EditCursorRight,
}

/// Clipboard holding copied cell values
#[derive(Debug, Clone, Default)]
pub struct Clipboard {
    /// Copied cells as (relative_col, relative_row, value)
    cells: Vec<(usize, usize, CellValue)>,
    /// Width of copied region
    width: usize,
    /// Height of copied region
    height: usize,
}

/// Handle an action on the spreadsheet state. Returns true if the state changed.
pub fn handle_action(state: &mut SpreadsheetState, action: Action, clipboard: &mut Clipboard) -> bool {
    match action {
        Action::MoveUp => {
            if state.edit.editing { return false; }
            nav::clear_selection(&mut state.cursor);
            nav::move_up(&mut state.cursor, &state.grid);
            state.scroll.ensure_visible(state.cursor.position);
            true
        }
        Action::MoveDown => {
            if state.edit.editing { return false; }
            nav::clear_selection(&mut state.cursor);
            nav::move_down(&mut state.cursor, &state.grid);
            state.scroll.ensure_visible(state.cursor.position);
            true
        }
        Action::MoveLeft => {
            if state.edit.editing { return false; }
            nav::clear_selection(&mut state.cursor);
            nav::move_left(&mut state.cursor, &state.grid);
            state.scroll.ensure_visible(state.cursor.position);
            true
        }
        Action::MoveRight => {
            if state.edit.editing { return false; }
            nav::clear_selection(&mut state.cursor);
            nav::move_right(&mut state.cursor, &state.grid);
            state.scroll.ensure_visible(state.cursor.position);
            true
        }
        Action::MoveHome => {
            if state.edit.editing { return false; }
            nav::clear_selection(&mut state.cursor);
            nav::move_home(&mut state.cursor, &state.grid);
            state.scroll.ensure_visible(state.cursor.position);
            true
        }
        Action::MoveEnd => {
            if state.edit.editing { return false; }
            nav::clear_selection(&mut state.cursor);
            nav::move_end(&mut state.cursor, &state.grid);
            state.scroll.ensure_visible(state.cursor.position);
            true
        }
        Action::MoveHomeAll => {
            if state.edit.editing { return false; }
            nav::clear_selection(&mut state.cursor);
            nav::move_home_all(&mut state.cursor, &state.grid);
            state.scroll.ensure_visible(state.cursor.position);
            true
        }
        Action::MoveEndAll => {
            if state.edit.editing { return false; }
            nav::clear_selection(&mut state.cursor);
            nav::move_end_all(&mut state.cursor, &state.grid);
            state.scroll.ensure_visible(state.cursor.position);
            true
        }
        Action::PageUp => {
            if state.edit.editing { return false; }
            nav::clear_selection(&mut state.cursor);
            nav::move_page_up(&mut state.cursor, &state.grid, state.scroll.visible_rows);
            state.scroll.ensure_visible(state.cursor.position);
            true
        }
        Action::PageDown => {
            if state.edit.editing { return false; }
            nav::clear_selection(&mut state.cursor);
            nav::move_page_down(&mut state.cursor, &state.grid, state.scroll.visible_rows);
            state.scroll.ensure_visible(state.cursor.position);
            true
        }
        Action::Tab => {
            if state.edit.editing {
                commit_edit(state);
            }
            nav::clear_selection(&mut state.cursor);
            nav::move_tab(&mut state.cursor, &state.grid);
            state.scroll.ensure_visible(state.cursor.position);
            true
        }
        Action::TabBack => {
            if state.edit.editing {
                commit_edit(state);
            }
            nav::clear_selection(&mut state.cursor);
            nav::move_tab_back(&mut state.cursor, &state.grid);
            state.scroll.ensure_visible(state.cursor.position);
            true
        }
        Action::EnterEdit(ch) => {
            if !state.edit.editing {
                let current = state.grid.get(state.cursor.position);
                let initial = match current {
                    CellValue::Formula { expr, .. } => format!("={}", expr),
                    CellValue::Empty => String::new(),
                    other => other.to_string(),
                };
                state.edit.previous_value = Some(current.clone());
                if let Some(c) = ch {
                    // Typing starts fresh edit
                    state.edit.start_edit(String::new());
                    state.edit.insert_char(c);
                } else {
                    // Enter key edits existing content
                    state.edit.start_edit(initial);
                }
            }
            true
        }
        Action::CommitEdit => {
            if state.edit.editing {
                commit_edit(state);
            }
            true
        }
        Action::CancelEdit => {
            if state.edit.editing && let Some(prev) = state.edit.cancel() {
                state.grid.set(state.cursor.position, prev);
            }
            true
        }
        Action::Undo => {
            if !state.edit.editing && let Some((addr, prev_value)) = state.last_undo.take() {
                state.grid.set(addr, prev_value);
                recalc_dependents(state, addr);
            }
            true
        }
        Action::Copy => {
            copy_selection(state, clipboard);
            true
        }
        Action::Paste => {
            paste_clipboard(state, clipboard);
            true
        }
        Action::StartSelection => {
            nav::start_selection(&mut state.cursor);
            true
        }
        Action::ClearSelection => {
            nav::clear_selection(&mut state.cursor);
            true
        }
        Action::ClickCell(addr) => {
            if state.edit.editing {
                commit_edit(state);
            }
            nav::clear_selection(&mut state.cursor);
            state.cursor.position = addr;
            state.scroll.ensure_visible(addr);
            true
        }
        Action::DeleteContent => {
            if !state.edit.editing {
                let addr = state.cursor.position;
                let prev = state.grid.get(addr).clone();
                state.last_undo = Some((addr, prev));
                state.grid.set(addr, CellValue::Empty);
                recalc_dependents(state, addr);
            }
            true
        }
        Action::TypeChar(ch) => {
            if state.edit.editing {
                state.edit.insert_char(ch);
            }
            true
        }
        Action::Backspace => {
            if state.edit.editing {
                state.edit.backspace();
            }
            true
        }
        Action::Delete => {
            if state.edit.editing {
                state.edit.delete_char();
            }
            true
        }
        Action::EditCursorLeft => {
            if state.edit.editing {
                state.edit.move_cursor_left();
            }
            true
        }
        Action::EditCursorRight => {
            if state.edit.editing {
                state.edit.move_cursor_right();
            }
            true
        }
    }
}

/// Commit the current edit buffer to the grid
fn commit_edit(state: &mut SpreadsheetState) {
    let addr = state.cursor.position;
    let input = state.edit.commit_buffer();

    // Validate if a validator exists for this column
    if let Err(_msg) = state.validate_input(addr.col, &input) {
        // Validation failed - re-enter edit mode with the same buffer
        state.edit.start_edit(input);
        return;
    }

    // Store previous value for undo
    let prev = state.grid.get(addr).clone();
    state.last_undo = Some((addr, prev));

    // Parse the input into a CellValue
    let value = if input.starts_with('=') && input.len() > 1 && !input.starts_with("= ") {
        let expr_str = &input[1..];
        match parse(expr_str) {
            Ok(expr) => {
                // Update dependency graph
                state.dep_graph.update_deps(addr, &expr);
                // Evaluate formula
                let cached = evaluate_with_registry(&expr, &state.grid, &state.fn_registry);
                CellValue::Formula {
                    expr: expr_str.to_string(),
                    cached: Box::new(cached),
                }
            }
            Err(_) => CellValue::Error(CellError::ParseError),
        }
    } else if let Ok(n) = input.parse::<f64>() {
        CellValue::Number(n)
    } else if input.eq_ignore_ascii_case("true") {
        CellValue::Boolean(true)
    } else if input.eq_ignore_ascii_case("false") {
        CellValue::Boolean(false)
    } else if input.is_empty() {
        CellValue::Empty
    } else {
        CellValue::Text(input)
    };

    state.grid.set(addr, value);
    recalc_dependents(state, addr);
}

/// Recalculate all cells that depend on the changed cell
fn recalc_dependents(state: &mut SpreadsheetState, changed: CellAddr) {
    let order = match state.dep_graph.get_recalc_order(changed) {
        Ok(order) => order,
        Err(_) => {
            // Cycle detected - mark all dependents as cycle errors
            let deps = state.dep_graph.get_dependents(changed);
            for dep in deps {
                state.grid.set(dep, CellValue::Error(CellError::CycleError));
            }
            return;
        }
    };

    for dep_addr in order {
        let cell = state.grid.get(dep_addr).clone();
        if let CellValue::Formula { ref expr, .. } = cell {
            match parse(expr) {
                Ok(parsed) => {
                    let cached = evaluate_with_registry(&parsed, &state.grid, &state.fn_registry);
                    state.grid.set(dep_addr, CellValue::Formula {
                        expr: expr.clone(),
                        cached: Box::new(cached),
                    });
                }
                Err(_) => {
                    state.grid.set(dep_addr, CellValue::Error(CellError::ParseError));
                }
            }
        }
    }
}

/// Copy selected cells to clipboard
fn copy_selection(state: &SpreadsheetState, clipboard: &mut Clipboard) {
    clipboard.cells.clear();

    let (start, end) = match get_selection(&state.cursor) {
        Selection::Range(range) => {
            let min_col = range.start.col.min(range.end.col);
            let max_col = range.start.col.max(range.end.col);
            let min_row = range.start.row.min(range.end.row);
            let max_row = range.start.row.max(range.end.row);
            (CellAddr { col: min_col, row: min_row }, CellAddr { col: max_col, row: max_row })
        }
        Selection::None => {
            // Copy single cell
            let pos = state.cursor.position;
            (pos, pos)
        }
    };

    clipboard.width = end.col - start.col + 1;
    clipboard.height = end.row - start.row + 1;

    for row in start.row..=end.row {
        for col in start.col..=end.col {
            let addr = CellAddr { col, row };
            let value = state.grid.get(addr).clone();
            clipboard.cells.push((col - start.col, row - start.row, value));
        }
    }
}

/// Paste clipboard contents at cursor position
fn paste_clipboard(state: &mut SpreadsheetState, clipboard: &Clipboard) {
    if clipboard.cells.is_empty() {
        return;
    }

    let origin = state.cursor.position;

    for (rel_col, rel_row, value) in &clipboard.cells {
        let target = CellAddr {
            col: origin.col + rel_col,
            row: origin.row + rel_row,
        };
        state.grid.set(target, value.clone());

        // Re-evaluate if it's a formula
        if let CellValue::Formula { expr, .. } = value && let Ok(parsed) = parse(expr) {
            state.dep_graph.update_deps(target, &parsed);
            let cached = evaluate_with_registry(&parsed, &state.grid, &state.fn_registry);
            state.grid.set(target, CellValue::Formula {
                expr: expr.clone(),
                cached: Box::new(cached),
            });
        }
    }

    // Recalc anything downstream
    for (rel_col, rel_row, _) in &clipboard.cells {
        let target = CellAddr {
            col: origin.col + rel_col,
            row: origin.row + rel_row,
        };
        recalc_dependents(state, target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_actions() {
        let mut state = SpreadsheetState::new(5, 5);
        let mut clip = Clipboard::default();

        handle_action(&mut state, Action::MoveRight, &mut clip);
        assert_eq!(state.cursor.position, CellAddr { col: 1, row: 0 });

        handle_action(&mut state, Action::MoveDown, &mut clip);
        assert_eq!(state.cursor.position, CellAddr { col: 1, row: 1 });

        handle_action(&mut state, Action::MoveLeft, &mut clip);
        assert_eq!(state.cursor.position, CellAddr { col: 0, row: 1 });

        handle_action(&mut state, Action::MoveUp, &mut clip);
        assert_eq!(state.cursor.position, CellAddr { col: 0, row: 0 });
    }

    #[test]
    fn test_edit_commit_number() {
        let mut state = SpreadsheetState::new(5, 5);
        let mut clip = Clipboard::default();

        handle_action(&mut state, Action::EnterEdit(None), &mut clip);
        assert!(state.edit.editing);

        for ch in "42.5".chars() {
            handle_action(&mut state, Action::TypeChar(ch), &mut clip);
        }

        handle_action(&mut state, Action::CommitEdit, &mut clip);
        assert!(!state.edit.editing);
        assert_eq!(*state.grid.get(CellAddr { col: 0, row: 0 }), CellValue::Number(42.5));
    }

    #[test]
    fn test_edit_commit_text() {
        let mut state = SpreadsheetState::new(5, 5);
        let mut clip = Clipboard::default();

        handle_action(&mut state, Action::EnterEdit(None), &mut clip);
        for ch in "hello".chars() {
            handle_action(&mut state, Action::TypeChar(ch), &mut clip);
        }
        handle_action(&mut state, Action::CommitEdit, &mut clip);

        assert_eq!(*state.grid.get(CellAddr { col: 0, row: 0 }), CellValue::Text("hello".to_string()));
    }

    #[test]
    fn test_edit_commit_formula() {
        let mut state = SpreadsheetState::new(5, 5);
        let mut clip = Clipboard::default();

        // Set A1 = 10
        state.grid.set(CellAddr { col: 0, row: 0 }, CellValue::Number(10.0));

        // Move to B1 and enter formula
        handle_action(&mut state, Action::MoveRight, &mut clip);
        handle_action(&mut state, Action::EnterEdit(None), &mut clip);
        for ch in "=A1+5".chars() {
            handle_action(&mut state, Action::TypeChar(ch), &mut clip);
        }
        handle_action(&mut state, Action::CommitEdit, &mut clip);

        let val = state.grid.get(CellAddr { col: 1, row: 0 });
        match val {
            CellValue::Formula { cached, .. } => {
                assert_eq!(**cached, CellValue::Number(15.0));
            }
            _ => panic!("Expected formula, got {:?}", val),
        }
    }

    #[test]
    fn test_cancel_edit() {
        let mut state = SpreadsheetState::new(5, 5);
        let mut clip = Clipboard::default();

        state.grid.set(CellAddr { col: 0, row: 0 }, CellValue::Text("old".to_string()));

        handle_action(&mut state, Action::EnterEdit(None), &mut clip);
        for ch in "new".chars() {
            handle_action(&mut state, Action::TypeChar(ch), &mut clip);
        }
        handle_action(&mut state, Action::CancelEdit, &mut clip);

        assert_eq!(*state.grid.get(CellAddr { col: 0, row: 0 }), CellValue::Text("old".to_string()));
    }

    #[test]
    fn test_undo() {
        let mut state = SpreadsheetState::new(5, 5);
        let mut clip = Clipboard::default();

        state.grid.set(CellAddr { col: 0, row: 0 }, CellValue::Text("old".to_string()));

        // Type 'n' to start fresh edit (replaces existing content)
        handle_action(&mut state, Action::EnterEdit(Some('n')), &mut clip);
        for ch in "ew".chars() {
            handle_action(&mut state, Action::TypeChar(ch), &mut clip);
        }
        handle_action(&mut state, Action::CommitEdit, &mut clip);
        assert_eq!(*state.grid.get(CellAddr { col: 0, row: 0 }), CellValue::Text("new".to_string()));

        handle_action(&mut state, Action::Undo, &mut clip);
        assert_eq!(*state.grid.get(CellAddr { col: 0, row: 0 }), CellValue::Text("old".to_string()));

        // Second undo does nothing (single-level)
        handle_action(&mut state, Action::Undo, &mut clip);
        assert_eq!(*state.grid.get(CellAddr { col: 0, row: 0 }), CellValue::Text("old".to_string()));
    }

    #[test]
    fn test_copy_paste_single_cell() {
        let mut state = SpreadsheetState::new(5, 5);
        let mut clip = Clipboard::default();

        state.grid.set(CellAddr { col: 0, row: 0 }, CellValue::Number(42.0));

        // Copy A1
        handle_action(&mut state, Action::Copy, &mut clip);
        assert_eq!(clip.cells.len(), 1);

        // Move to B2 and paste
        state.cursor.position = CellAddr { col: 1, row: 1 };
        handle_action(&mut state, Action::Paste, &mut clip);

        assert_eq!(*state.grid.get(CellAddr { col: 1, row: 1 }), CellValue::Number(42.0));
    }

    #[test]
    fn test_copy_paste_range() {
        let mut state = SpreadsheetState::new(5, 5);
        let mut clip = Clipboard::default();

        state.grid.set(CellAddr { col: 0, row: 0 }, CellValue::Number(1.0));
        state.grid.set(CellAddr { col: 1, row: 0 }, CellValue::Number(2.0));
        state.grid.set(CellAddr { col: 0, row: 1 }, CellValue::Number(3.0));
        state.grid.set(CellAddr { col: 1, row: 1 }, CellValue::Number(4.0));

        // Select A1:B2
        state.cursor.position = CellAddr { col: 0, row: 0 };
        state.cursor.anchor = Some(CellAddr { col: 1, row: 1 });

        handle_action(&mut state, Action::Copy, &mut clip);
        assert_eq!(clip.cells.len(), 4);
        assert_eq!(clip.width, 2);
        assert_eq!(clip.height, 2);

        // Paste at C3
        state.cursor.position = CellAddr { col: 2, row: 2 };
        handle_action(&mut state, Action::Paste, &mut clip);

        assert_eq!(*state.grid.get(CellAddr { col: 2, row: 2 }), CellValue::Number(1.0));
        assert_eq!(*state.grid.get(CellAddr { col: 3, row: 2 }), CellValue::Number(2.0));
        assert_eq!(*state.grid.get(CellAddr { col: 2, row: 3 }), CellValue::Number(3.0));
        assert_eq!(*state.grid.get(CellAddr { col: 3, row: 3 }), CellValue::Number(4.0));
    }

    #[test]
    fn test_typing_starts_edit() {
        let mut state = SpreadsheetState::new(5, 5);
        let mut clip = Clipboard::default();

        handle_action(&mut state, Action::EnterEdit(Some('x')), &mut clip);
        assert!(state.edit.editing);
        assert_eq!(state.edit.buffer, "x");
    }

    #[test]
    fn test_delete_content() {
        let mut state = SpreadsheetState::new(5, 5);
        let mut clip = Clipboard::default();

        state.grid.set(CellAddr { col: 0, row: 0 }, CellValue::Number(42.0));
        handle_action(&mut state, Action::DeleteContent, &mut clip);
        assert_eq!(*state.grid.get(CellAddr { col: 0, row: 0 }), CellValue::Empty);
    }

    #[test]
    fn test_formula_recalc_on_dependency_change() {
        let mut state = SpreadsheetState::new(5, 5);
        let mut clip = Clipboard::default();

        // Set A1 = 1
        state.grid.set(CellAddr { col: 0, row: 0 }, CellValue::Number(1.0));

        // Set B1 = =A1+1 via edit
        state.cursor.position = CellAddr { col: 1, row: 0 };
        handle_action(&mut state, Action::EnterEdit(None), &mut clip);
        for ch in "=A1+1".chars() {
            handle_action(&mut state, Action::TypeChar(ch), &mut clip);
        }
        handle_action(&mut state, Action::CommitEdit, &mut clip);

        // B1 should be 2
        let b1 = state.grid.get(CellAddr { col: 1, row: 0 });
        match b1 {
            CellValue::Formula { cached, .. } => assert_eq!(**cached, CellValue::Number(2.0)),
            _ => panic!("Expected formula"),
        }

        // Now change A1 to 10
        state.cursor.position = CellAddr { col: 0, row: 0 };
        handle_action(&mut state, Action::EnterEdit(None), &mut clip);
        // Clear the buffer then type new value
        while !state.edit.buffer.is_empty() {
            handle_action(&mut state, Action::Backspace, &mut clip);
        }
        for ch in "10".chars() {
            handle_action(&mut state, Action::TypeChar(ch), &mut clip);
        }
        handle_action(&mut state, Action::CommitEdit, &mut clip);

        // B1 should recalculate to 11
        let b1 = state.grid.get(CellAddr { col: 1, row: 0 });
        match b1 {
            CellValue::Formula { cached, .. } => assert_eq!(**cached, CellValue::Number(11.0)),
            _ => panic!("Expected formula"),
        }
    }

    #[test]
    fn test_validation_rejects_invalid_input() {
        let mut state = SpreadsheetState::new(5, 5);
        let mut clip = Clipboard::default();

        // Set column 0 to numeric-only
        state.set_column_validator(0, |input| {
            if input.parse::<f64>().is_ok() || input.is_empty() {
                Ok(())
            } else {
                Err("Must be numeric".to_string())
            }
        });

        // Try to commit text in column 0 - should be rejected
        handle_action(&mut state, Action::EnterEdit(None), &mut clip);
        for ch in "abc".chars() {
            handle_action(&mut state, Action::TypeChar(ch), &mut clip);
        }
        handle_action(&mut state, Action::CommitEdit, &mut clip);

        // Should still be in edit mode (validation failed)
        assert!(state.edit.editing);

        // Cancel and try a valid number
        handle_action(&mut state, Action::CancelEdit, &mut clip);
        handle_action(&mut state, Action::EnterEdit(None), &mut clip);
        for ch in "42".chars() {
            handle_action(&mut state, Action::TypeChar(ch), &mut clip);
        }
        handle_action(&mut state, Action::CommitEdit, &mut clip);

        // Should commit successfully
        assert!(!state.edit.editing);
        assert_eq!(*state.grid.get(CellAddr { col: 0, row: 0 }), CellValue::Number(42.0));
    }
}
