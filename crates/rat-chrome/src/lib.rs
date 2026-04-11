//! Structural chrome primitives for ratatui.
//!
//! `rat-chrome` starts with a small overlay frame primitive that computes a
//! popup rect from a viewport, renders optional backdrop + chrome, and returns
//! the inner content rect for caller-owned body widgets.
//!
//! # Example
//!
//! ```rust,ignore
//! use rat_chrome::{
//!     OverlayAnchor, OverlayModel, OverlaySize, OverlayStyle, overlay_frame,
//! };
//!
//! # use ratatui::{Frame, layout::Rect, widgets::Paragraph};
//! # fn draw(frame: &mut Frame, area: Rect) {
//! let model = OverlayModel::default()
//!     .with_anchor(OverlayAnchor::Center)
//!     .with_width(OverlaySize::Fixed(40))
//!     .with_height(OverlaySize::Fixed(8))
//!     .with_title(" Overlay ")
//!     .with_backdrop(true);
//!
//! let layout = overlay_frame(frame, area, &model, &OverlayStyle::default());
//! frame.render_widget(Paragraph::new("Body content"), layout.inner);
//! # }
//! ```
//!
//! Intended migration targets in this workspace include duplicated popup code
//! in `rat-widgets`, `rat-branches`, `rat-leaderkey`, `rat-streaming`, and
//! `rat-capture`.
//!
//! Deferred for follow-up changes: animation, hit testing, and broader popup
//! refactors.

pub mod overlay;

pub use overlay::{
    OverlayAnchor, OverlayLayout, OverlayModel, OverlaySize, OverlayStyle, compute_overlay_layout,
    overlay_frame,
};
