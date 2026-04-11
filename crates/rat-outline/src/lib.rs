//! Folding structured editor for ratatui.
//!
//! Wraps `rat_editor::Editor` with heading-aware features: fold/unfold,
//! visibility cycling, structural editing (promote/demote/move subtree),
//! and TODO state management.
//!
//! # Architecture
//!
//! `OutlineState` owns an `Editor` for the text buffer and maintains a
//! parallel `Vec<HeadingInfo>` index. The index is rebuilt whenever the
//! buffer changes. Rendering filters visible lines through fold state.
//!
//! # Example
//!
//! ```rust,ignore
//! use rat_outline::{OutlineState, Action, handle_action, Outline, OutlineStyle};
//!
//! let mut state = OutlineState::new();
//! state.load_text("* TODO Top heading\nbody\n** Child heading\n");
//! handle_action(&mut state, Action::CycleVisibility);
//! ```

pub mod action;
pub mod fold;
pub mod index;
pub mod parse;
pub mod render;
pub mod state;
pub mod structure;
pub mod todo;

pub use action::{Action, ActionResult, handle_action};
pub use fold::{cycle_visibility, cycle_visibility_global, visible_lines};
pub use index::{FoldState, HeadingInfo, build_heading_index};
pub use parse::{HeadingParser, HeadingSyntax, MarkdownParser, OrgParser, ParsedHeading};
pub use render::{Outline, OutlineStyle};
pub use state::OutlineState;
pub use structure::{demote, move_subtree_down, move_subtree_up, promote};
pub use todo::cycle_todo;
