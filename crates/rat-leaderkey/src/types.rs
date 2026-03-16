//! Core types for the leader menu system.

use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Leader actions — things the leader menu can trigger
// ---------------------------------------------------------------------------

/// An action that a leader menu item can trigger.
///
/// Generic over `A`, the application's action type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaderAction<A> {
    /// Dispatch a user-defined action.
    Action(A),
    /// Execute a command string (e.g., "/compact", ":write").
    Command(String),
    /// Open a named submenu.
    Submenu(String),
}

// ---------------------------------------------------------------------------
// Menu item
// ---------------------------------------------------------------------------

/// A single entry in the leader key menu.
#[derive(Debug, Clone)]
pub struct LeaderMenuItem<A> {
    /// Key to press (single char).
    pub key: char,
    /// Display label.
    pub label: String,
    /// What happens when selected.
    pub action: LeaderAction<A>,
}

// ---------------------------------------------------------------------------
// Menu definition (a flat level of items)
// ---------------------------------------------------------------------------

/// A named menu level (root or submenu).
#[derive(Debug, Clone)]
pub struct LeaderMenuDef<A> {
    pub label: String,
    pub items: Vec<LeaderMenuItem<A>>,
}

// ---------------------------------------------------------------------------
// Dynamic registration types
// ---------------------------------------------------------------------------

/// Where a menu item should appear.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MenuPlacement {
    /// Top-level root menu.
    Root,
    /// Inside a named submenu (created if it doesn't exist).
    Submenu(String),
}

/// A single contribution to the leader menu from any source.
#[derive(Debug, Clone)]
pub struct MenuContribution<A> {
    /// Key to press (single char).
    pub key: char,
    /// Display label.
    pub label: String,
    /// What happens when selected.
    pub action: LeaderAction<A>,
    /// Where this item appears.
    pub placement: MenuPlacement,
    /// Priority for conflict resolution (higher wins).
    pub priority: u16,
    /// Source identifier for diagnostics ("builtin", plugin name, "config").
    pub source: String,
}

/// Anything that contributes items to the leader menu.
pub trait MenuContributor<A> {
    fn menu_items(&self) -> Vec<MenuContribution<A>>;
}

/// Set of `(key, placement)` pairs to exclude from the built menu.
pub type HiddenSet = HashSet<(char, MenuPlacement)>;
