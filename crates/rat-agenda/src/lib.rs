//! Calendar/agenda view widget for ratatui.
//!
//! Renders day, week, and month views of agenda items. Provides filter controls
//! and navigation. Data is supplied through the `AgendaDataSource` trait.

pub mod action;
pub mod filter;
pub mod render;
pub mod state;
pub mod types;

pub use action::{Action, ActionResult, handle_action};
pub use filter::FilterSpec;
pub use render::{Agenda, AgendaStyle};
pub use state::{AgendaState, ViewMode};
pub use types::{AgendaDataSource, AgendaItem, Date, DateRange, Time};
