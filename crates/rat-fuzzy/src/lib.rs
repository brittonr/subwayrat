//! Incremental fuzzy finder widget for ratatui.
//!
//! Provides fuzzy matching with scoring, match highlighting, and a scrollable
//! result list. Data is supplied through the `FuzzySource` trait.

pub mod types;
pub mod score;
pub mod state;
pub mod action;
pub mod render;

pub use types::{FuzzyCandidate, FuzzySource};
pub use score::{fuzzy_score, ScoredMatch};
pub use state::FuzzyState;
pub use action::{Action, handle_action};
pub use render::{FuzzyFinder, FuzzyStyle};
