//! Helix-style leader key popup menu for ratatui applications.
//!
//! The state machine, builder, and types live in `ratcore::leaderkey`
//! (shared with the Dioxus web frontend). This crate provides:
//! - crossterm `KeyEvent` → `MenuInput` conversion
//! - ratatui rendering
//!
//! # Example
//!
//! ```rust
//! use rat_leaderkey::*;
//!
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! enum MyAction { Save, Quit }
//!
//! struct Builtins;
//! impl MenuContributor<MyAction> for Builtins {
//!     fn menu_items(&self) -> Vec<MenuContribution<MyAction>> {
//!         vec![
//!             MenuContribution {
//!                 key: 's',
//!                 label: "save".into(),
//!                 action: LeaderAction::Action(MyAction::Save),
//!                 placement: MenuPlacement::Root,
//!                 priority: PRIORITY_BUILTIN,
//!                 source: "builtin".into(),
//!             },
//!             MenuContribution {
//!                 key: 'q',
//!                 label: "quit".into(),
//!                 action: LeaderAction::Action(MyAction::Quit),
//!                 placement: MenuPlacement::Root,
//!                 priority: PRIORITY_BUILTIN,
//!                 source: "builtin".into(),
//!             },
//!         ]
//!     }
//! }
//!
//! let hidden = std::collections::HashSet::new();
//! let (mut menu, conflicts) = build(&[&Builtins], &hidden);
//! assert!(conflicts.is_empty());
//! menu.open();
//! ```

mod builder;
mod registry;
mod render;
mod state;
mod types;

pub use builder::{build, build_from_items, BuildResult};
pub use registry::*;
pub use state::LeaderMenu;
pub use types::*;
