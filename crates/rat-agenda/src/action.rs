//! Action enum and dispatch for the agenda widget.

use crate::filter::FilterSpec;
use crate::state::{AgendaState, ViewMode};

/// Actions for the agenda widget.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    NextDay,
    PrevDay,
    NextWeek,
    PrevWeek,
    NextMonth,
    PrevMonth,
    SwitchView(ViewMode),
    SelectNextItem,
    SelectPrevItem,
    ToggleFilter,
    SetFilter(FilterSpec),
    ClearFilters,
    Refresh,
}

/// Result of handling an action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionResult {
    Handled,
    NeedsRefresh,
    Noop,
}

/// Process an agenda action.
pub fn handle_action(state: &mut AgendaState, action: Action) -> ActionResult {
    match action {
        Action::NextDay => {
            state.selected_date = state.selected_date.next_day();
            ActionResult::NeedsRefresh
        }
        Action::PrevDay => {
            state.selected_date = state.selected_date.prev_day();
            ActionResult::NeedsRefresh
        }
        Action::NextWeek => {
            state.selected_date = state.selected_date.add_days(7);
            ActionResult::NeedsRefresh
        }
        Action::PrevWeek => {
            state.selected_date = state.selected_date.add_days(-7);
            ActionResult::NeedsRefresh
        }
        Action::NextMonth => {
            state.selected_date = state.selected_date.next_month();
            ActionResult::NeedsRefresh
        }
        Action::PrevMonth => {
            state.selected_date = state.selected_date.prev_month();
            ActionResult::NeedsRefresh
        }
        Action::SwitchView(mode) => {
            state.view_mode = mode;
            state.scroll_offset = 0;
            ActionResult::NeedsRefresh
        }
        Action::SelectNextItem => {
            if !state.items.is_empty() {
                state.selected_item = (state.selected_item + 1).min(state.items.len() - 1);
            }
            ActionResult::Handled
        }
        Action::SelectPrevItem => {
            state.selected_item = state.selected_item.saturating_sub(1);
            ActionResult::Handled
        }
        Action::ToggleFilter => {
            state.filter_active = !state.filter_active;
            ActionResult::Handled
        }
        Action::SetFilter(spec) => {
            state.filter = spec;
            ActionResult::NeedsRefresh
        }
        Action::ClearFilters => {
            state.filter = FilterSpec::default();
            state.filter_active = false;
            ActionResult::NeedsRefresh
        }
        Action::Refresh => ActionResult::NeedsRefresh,
    }
}
