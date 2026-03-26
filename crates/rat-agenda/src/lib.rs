//! Calendar/agenda view widget for ratatui.
//!
//! Renders day, week, and month views of agenda items. Provides filter controls
//! and navigation. Data is supplied through the `AgendaDataSource` trait.

pub mod types;
pub mod state;
pub mod filter;
pub mod render;
pub mod action;

pub use types::{AgendaItem, DateRange, Date, Time, AgendaDataSource};
pub use state::{AgendaState, ViewMode};
pub use filter::FilterSpec;
pub use action::{Action, ActionResult, handle_action};
pub use render::{Agenda, AgendaStyle};
