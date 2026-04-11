//! # rat-scrolltile
//!
//! Niri-style scrolling tiled layout engine for ratatui.
//!
//! Arranges windows in a scrollable strip of columns. Each column contains one
//! or more stacked windows. The viewport follows the focused window, and layout
//! produces ratatui [`Rect`] values ready for rendering.
//!
//! ## Quick start
//!
//! ```rust
//! use rat_scrolltile::{Strip, StripConfig, SizeConstraint, compute_layout};
//! use rat_scrolltile::nav;
//!
//! let mut strip = Strip::new(StripConfig::default());
//!
//! // Add windows — returns an opaque WindowId.
//! let a = strip.insert_window(0, 0, SizeConstraint::Fixed(30), SizeConstraint::default());
//! let b = strip.insert_window(1, 0, SizeConstraint::Fixed(40), SizeConstraint::default());
//! let c = strip.insert_window(1, 1, SizeConstraint::Fixed(40), SizeConstraint::Proportion(1.0));
//!
//! // Focus and navigate.
//! strip.focus_set(a);
//! nav::focus_right(&mut strip, 80, 24);
//!
//! // Compute layout for an 80×24 viewport.
//! let result = compute_layout(&strip, 80, 24);
//!
//! // Render visible windows.
//! for vw in &result.visible {
//!     // vw.id identifies the window, vw.rect is viewport-local.
//!     println!("{:?}: {:?}", vw.id, vw.rect);
//! }
//! ```
//!
//! ## Architecture
//!
//! - **[`Strip`]** — top-level container holding columns, config, and focus state.
//! - **[`Column`]** — a group of stacked windows sharing a primary-axis slot.
//! - **[`Window`]** — a leaf with size constraints, identified by [`WindowId`].
//! - **[`compute_layout`]** — pure function: `&Strip` + viewport size → [`LayoutResult`].
//! - **[`nav`]** — focus navigation (left/right/up/down/first/last).
//!
//! No retained layout tree. No float arithmetic. Integer cells throughout.

mod layout;
pub mod nav;
mod strip;
mod types;

pub use layout::{LayoutResult, VisibleWindow, compute_layout};
pub use strip::{Column, Strip, Window};
pub use types::{Axis, SizeConstraint, StripConfig, WindowId};

#[cfg(test)]
mod tests;
