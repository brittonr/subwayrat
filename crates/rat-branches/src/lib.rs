//! Generic tree branch utilities and comparison widgets for ratatui
//!
//! This crate provides generic tree algorithms, branch comparison structures,
//! and TUI components for working with tree-like data structures, extracted
//! from clankers-tui's conversation branch system.

pub mod compare;
pub mod compare_view;
pub mod switcher;
pub mod tree;

// Re-export main types for convenience
pub use compare::{BranchComparison, CompareBlock};
pub use compare_view::BranchCompareView;
pub use switcher::{NodeSwitcher, SwitcherItem};
pub use tree::TreeNode;