//! Helix-style leader key popup menu for ratatui applications.
//!
//! Pressing a leader key (typically Space) opens a which-key overlay showing
//! available actions. A single keypress executes an action or opens a submenu.
//! Escape or any unrecognized key dismisses the menu.
//!
//! The menu is generic over the action type `A`, so any application can plug in
//! its own action enum. Items are contributed dynamically via [`MenuContributor`],
//! supporting builtins, plugins, and user config with priority-based conflict
//! resolution.
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
