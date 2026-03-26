//! # rat-spreadsheet
//!
//! An editable spreadsheet widget for ratatui with formula support, cell navigation,
//! and inline editing.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use rat_spreadsheet::{Spreadsheet, SpreadsheetState};
//! use ratatui::Frame;
//! use ratatui::layout::Rect;
//!
//! let mut state = SpreadsheetState::new(10, 20);
//! // ... set cell values, handle input ...
//! ```
//!
//! ## Architecture
//!
//! The crate separates the widget (`Spreadsheet`) from state (`SpreadsheetState`),
//! following ratatui's `StatefulWidget` pattern. The application owns the state and
//! passes the widget as a temporary for rendering.
//!
//! ## Modules
//!
//! - `cell` - Cell addressing, value types, and grid storage
//! - `formula` - Expression parsing, evaluation, and dependency tracking
//! - `nav` - Cursor movement, scrolling, and selection
//! - `render` - Widget and state definitions, rendering logic
//! - `event` - Unified action handling, copy/paste, edit lifecycle

pub mod cell;
pub mod edit_state;
pub mod formula;
pub mod nav;
pub mod render;
pub mod event;

// Re-export primary types at crate root for convenience
pub use cell::{CellAddr, CellRange, CellValue, CellError, Grid};
pub use edit_state::EditState;
pub use formula::{DependencyGraph, FunctionRegistry, Expr, Op};
pub use formula::{parse as parse_formula, evaluate, evaluate_with_registry};
pub use nav::{CursorState, ScrollState, Selection};
pub use render::{Spreadsheet, SpreadsheetState, SpreadsheetStyle};
pub use event::{Action, Clipboard, handle_action};

#[cfg(feature = "org-compat")]
pub mod org_table;
