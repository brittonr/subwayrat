//! Action enum and dispatch for the outline editor.

use crate::fold::{cycle_visibility, cycle_visibility_global};
use crate::index::FoldState;
use crate::parse::HeadingSyntax;
use crate::state::OutlineState;
use crate::structure::{demote, move_subtree_down, move_subtree_up, promote};
use crate::todo::cycle_todo;

/// Actions that can be performed on an outline.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // ── Folding ──
    /// Cycle visibility of the heading at cursor: folded→children→all→folded.
    CycleVisibility,
    /// Fold all headings.
    FoldAll,
    /// Unfold all headings.
    UnfoldAll,

    // ── Structure ──
    /// Promote heading (and subtree) at cursor — decrease level.
    Promote,
    /// Demote heading (and subtree) at cursor — increase level.
    Demote,
    /// Move subtree at cursor above its previous sibling.
    MoveSubtreeUp,
    /// Move subtree at cursor below its next sibling.
    MoveSubtreeDown,

    // ── TODO ──
    /// Cycle the TODO keyword on the heading at cursor.
    CycleTodo,

    // ── Heading insertion ──
    /// Insert a new heading at the same level after the current subtree.
    InsertHeading,
    /// Insert a new sub-heading (level + 1) after the current line.
    InsertSubheading,

    // ── Text editing (delegated to inner Editor) ──
    InsertChar(char),
    DeleteBack,
    DeleteForward,
    DeleteWordBack,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveHome,
    MoveEnd,
}

/// Result of handling an action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionResult {
    /// Action was handled, state changed.
    Handled,
    /// Action was not applicable (e.g. promote at level 1).
    Noop,
}

/// Process an action on the outline state.
pub fn handle_action(state: &mut OutlineState, action: Action) -> ActionResult {
    state.ensure_index();

    match action {
        Action::CycleVisibility => {
            if let Some(hi) = state.current_heading_idx() {
                cycle_visibility(&mut state.headings, hi);
                ActionResult::Handled
            } else {
                ActionResult::Noop
            }
        }
        Action::FoldAll => {
            cycle_visibility_global(&mut state.headings, FoldState::Folded);
            ActionResult::Handled
        }
        Action::UnfoldAll => {
            cycle_visibility_global(&mut state.headings, FoldState::All);
            ActionResult::Handled
        }
        Action::Promote => {
            if let Some(hi) = state.current_heading_idx() {
                let syntax = state.syntax();
                let mut lines = state.lines().to_vec();
                if promote(&mut lines, &state.headings, hi, syntax) {
                    replace_buffer(state, &lines);
                    ActionResult::Handled
                } else {
                    ActionResult::Noop
                }
            } else {
                ActionResult::Noop
            }
        }
        Action::Demote => {
            if let Some(hi) = state.current_heading_idx() {
                let syntax = state.syntax();
                let mut lines = state.lines().to_vec();
                if demote(&mut lines, &state.headings, hi, syntax) {
                    replace_buffer(state, &lines);
                    ActionResult::Handled
                } else {
                    ActionResult::Noop
                }
            } else {
                ActionResult::Noop
            }
        }
        Action::MoveSubtreeUp => {
            if let Some(hi) = state.current_heading_idx() {
                let mut lines = state.lines().to_vec();
                if let Some(new_line) = move_subtree_up(&mut lines, &state.headings, hi) {
                    replace_buffer(state, &lines);
                    state.editor.set_cursor(new_line, 0);
                    ActionResult::Handled
                } else {
                    ActionResult::Noop
                }
            } else {
                ActionResult::Noop
            }
        }
        Action::MoveSubtreeDown => {
            if let Some(hi) = state.current_heading_idx() {
                let mut lines = state.lines().to_vec();
                if let Some(new_line) = move_subtree_down(&mut lines, &state.headings, hi) {
                    replace_buffer(state, &lines);
                    state.editor.set_cursor(new_line, 0);
                    ActionResult::Handled
                } else {
                    ActionResult::Noop
                }
            } else {
                ActionResult::Noop
            }
        }
        Action::CycleTodo => {
            if let Some(hi) = state.current_heading_idx() {
                let heading = state.headings[hi].clone();
                let kws = state.todo_keywords.clone();
                cycle_todo(state.editor.content_mut(), &heading, &kws);
                state.mark_dirty();
                ActionResult::Handled
            } else {
                ActionResult::Noop
            }
        }
        Action::InsertHeading => {
            insert_heading_after(state, false);
            ActionResult::Handled
        }
        Action::InsertSubheading => {
            insert_heading_after(state, true);
            ActionResult::Handled
        }

        // ── Text editing delegation ──
        Action::InsertChar(c) => {
            state.editor.insert_char(c);
            state.mark_dirty();
            ActionResult::Handled
        }
        Action::DeleteBack => {
            state.editor.delete_back();
            state.mark_dirty();
            ActionResult::Handled
        }
        Action::DeleteForward => {
            state.editor.delete_forward();
            state.mark_dirty();
            ActionResult::Handled
        }
        Action::DeleteWordBack => {
            state.editor.delete_word_back();
            state.mark_dirty();
            ActionResult::Handled
        }
        Action::MoveLeft => {
            state.editor.move_left();
            ActionResult::Handled
        }
        Action::MoveRight => {
            state.editor.move_right();
            ActionResult::Handled
        }
        Action::MoveUp => {
            let cl = state.editor.cursor_line();
            if cl > 0 {
                let col = state.editor.cursor_col();
                state.editor.set_cursor(cl - 1, col);
            }
            ActionResult::Handled
        }
        Action::MoveDown => {
            let cl = state.editor.cursor_line();
            if cl + 1 < state.editor.line_count() {
                let col = state.editor.cursor_col();
                state.editor.set_cursor(cl + 1, col);
            }
            ActionResult::Handled
        }
        Action::MoveHome => {
            state.editor.move_home();
            ActionResult::Handled
        }
        Action::MoveEnd => {
            state.editor.move_end();
            ActionResult::Handled
        }
    }
}

/// Replace the entire buffer content and rebuild index.
fn replace_buffer(state: &mut OutlineState, new_lines: &[String]) {
    let cursor_line = state.editor.cursor_line();
    let cursor_col = state.editor.cursor_col();
    let text = new_lines.join("\n");
    state.load_text(&text);
    state.editor.set_cursor(cursor_line, cursor_col);
}

/// Insert a new heading after the current subtree or line.
fn insert_heading_after(state: &mut OutlineState, sub: bool) {
    let level = if let Some(hi) = state.current_heading_idx() {
        let base = state.headings[hi].level;
        if sub { base + 1 } else { base }
    } else {
        1
    };

    let marker = match state.syntax() {
        HeadingSyntax::Org => "*".repeat(level),
        HeadingSyntax::Markdown => "#".repeat(level),
    };

    let insert_at = state.editor.cursor_line() + 1;
    let new_line = format!("{} ", marker);

    let mut lines = state.lines().to_vec();
    lines.insert(insert_at, new_line);
    let text = lines.join("\n");
    state.load_text(&text);
    state.editor.set_cursor(insert_at, marker.len() + 1);
}
