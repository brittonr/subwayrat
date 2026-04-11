//! Agenda state management.

use crate::filter::FilterSpec;
use crate::types::{AgendaItem, Date, DateRange};

/// Which view layout to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Day,
    Week,
    Month,
}

/// The agenda widget's state.
pub struct AgendaState {
    /// Current view mode.
    pub view_mode: ViewMode,
    /// The currently selected/focused date.
    pub selected_date: Date,
    /// Today's date (for highlighting).
    pub today: Date,
    /// Items in the current visible range (populated by refresh).
    pub items: Vec<AgendaItem>,
    /// Which item index is selected (within the day view).
    pub selected_item: usize,
    /// Active filter.
    pub filter: FilterSpec,
    /// Whether the filter bar is visible/focused.
    pub filter_active: bool,
    /// Vertical scroll offset.
    pub scroll_offset: usize,
    /// Week start day: 0=Monday (default).
    pub week_start_day: u32,
}

impl AgendaState {
    pub fn new(today: Date) -> Self {
        Self {
            view_mode: ViewMode::Day,
            selected_date: today,
            today,
            items: Vec::new(),
            selected_item: 0,
            filter: FilterSpec::default(),
            filter_active: false,
            scroll_offset: 0,
            week_start_day: 0,
        }
    }

    /// Get the date range for the current view.
    pub fn visible_range(&self) -> DateRange {
        match self.view_mode {
            ViewMode::Day => DateRange::new(self.selected_date, self.selected_date.next_day()),
            ViewMode::Week => {
                let start = self.selected_date.week_start();
                DateRange::new(start, start.add_days(7))
            }
            ViewMode::Month => {
                let first = self.selected_date.first_of_month();
                let next_month = first.next_month().first_of_month();
                DateRange::new(first, next_month)
            }
        }
    }

    /// Refresh items from a data source.
    pub fn refresh(&mut self, source: &dyn crate::types::AgendaDataSource) {
        let range = self.visible_range();
        let all = source.items(range);
        self.items = if self.filter.is_empty() {
            all
        } else {
            all.into_iter().filter(|i| self.filter.matches(i)).collect()
        };
        self.selected_item = self.selected_item.min(self.items.len().saturating_sub(1));
    }

    /// Items for a specific date, sorted: timed first (by start time), then
    /// untimed (by priority descending).
    pub fn items_for_date(&self, date: Date) -> Vec<&AgendaItem> {
        let mut day_items: Vec<&AgendaItem> = self
            .items
            .iter()
            .filter(|i| i.display_date() == Some(date))
            .collect();

        day_items.sort_by(|a, b| {
            match (&a.time_start, &b.time_start) {
                (Some(ta), Some(tb)) => ta.cmp(tb),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => {
                    // Sort by priority descending (A < B < C)
                    a.priority.cmp(&b.priority)
                }
            }
        });

        day_items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Time;

    fn make_item(id: &str, date: Date, time: Option<Time>, priority: Option<char>) -> AgendaItem {
        AgendaItem {
            id: id.into(),
            title: id.into(),
            status: None,
            priority,
            tags: vec![],
            scheduled: Some(date),
            deadline: None,
            time_start: time,
            time_end: None,
            source_file: None,
            source_line: None,
        }
    }

    #[test]
    fn items_for_date_sorted() {
        let today = Date::new(2026, 3, 26);
        let mut state = AgendaState::new(today);
        state.items = vec![
            make_item("untimed-B", today, None, Some('B')),
            make_item("timed-14", today, Some(Time::new(14, 0)), None),
            make_item("untimed-A", today, None, Some('A')),
            make_item("timed-09", today, Some(Time::new(9, 0)), None),
        ];
        let sorted = state.items_for_date(today);
        assert_eq!(sorted[0].id, "timed-09");
        assert_eq!(sorted[1].id, "timed-14");
        assert_eq!(sorted[2].id, "untimed-A");
        assert_eq!(sorted[3].id, "untimed-B");
    }

    #[test]
    fn visible_range_day() {
        let state = AgendaState::new(Date::new(2026, 3, 26));
        let r = state.visible_range();
        assert_eq!(r.start, Date::new(2026, 3, 26));
        assert_eq!(r.end, Date::new(2026, 3, 27));
    }

    #[test]
    fn visible_range_week() {
        let mut state = AgendaState::new(Date::new(2026, 3, 26)); // Thursday
        state.view_mode = ViewMode::Week;
        let r = state.visible_range();
        assert_eq!(r.start, Date::new(2026, 3, 23)); // Monday
        assert_eq!(r.end, Date::new(2026, 3, 30));
    }
}
