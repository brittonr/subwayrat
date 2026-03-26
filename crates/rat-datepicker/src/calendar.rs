//! CalendarGrid: month view date picker.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, StatefulWidget, Widget};

use crate::style::CalendarStyle;

/// Simple date type (duplicated from rat-agenda for zero-dep).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CalDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl CalDate {
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    pub fn weekday(&self) -> u32 {
        let t = [0i32, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let mut y = self.year;
        if self.month < 3 { y -= 1; }
        let dow = (y + y/4 - y/100 + y/400 + t[(self.month-1) as usize] + self.day as i32) % 7;
        ((dow + 6) % 7) as u32
    }

    pub fn days_in_month(&self) -> u32 {
        match self.month {
            1|3|5|7|8|10|12 => 31, 4|6|9|11 => 30,
            2 => if self.year%4==0 && (self.year%100!=0 || self.year%400==0) { 29 } else { 28 },
            _ => 30,
        }
    }

    pub fn next_day(self) -> Self {
        if self.day < self.days_in_month() { Self { day: self.day+1, ..self } }
        else if self.month < 12 { Self { month: self.month+1, day: 1, ..self } }
        else { Self { year: self.year+1, month: 1, day: 1 } }
    }

    pub fn prev_day(self) -> Self {
        if self.day > 1 { Self { day: self.day-1, ..self } }
        else if self.month > 1 { let pm = Self { month: self.month-1, day: 1, ..self }; Self { day: pm.days_in_month(), ..pm } }
        else { Self { year: self.year-1, month: 12, day: 31 } }
    }

    pub fn add_days(self, n: i32) -> Self {
        let mut d = self;
        if n >= 0 { for _ in 0..n { d = d.next_day(); } }
        else { for _ in 0..(-n) { d = d.prev_day(); } }
        d
    }

    fn month_name(&self) -> &'static str {
        ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"]
            .get((self.month as usize).wrapping_sub(1)).copied().unwrap_or("???")
    }
}

/// Calendar grid state.
pub struct CalendarGridState {
    pub selected: CalDate,
    pub display_month: u32,
    pub display_year: i32,
    pub today: CalDate,
    pub week_start: u32, // 0=Monday
}

impl CalendarGridState {
    pub fn new(today: CalDate) -> Self {
        Self {
            selected: today,
            display_month: today.month,
            display_year: today.year,
            today,
            week_start: 0,
        }
    }

    fn sync_display(&mut self) {
        self.display_month = self.selected.month;
        self.display_year = self.selected.year;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalendarAction {
    NextDay, PrevDay, NextWeek, PrevWeek, NextMonth, PrevMonth, Confirm, Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalendarResult {
    Handled,
    Confirmed(CalDate),
    Cancelled,
}

pub fn handle_calendar_action(state: &mut CalendarGridState, action: CalendarAction) -> CalendarResult {
    match action {
        CalendarAction::NextDay => { state.selected = state.selected.next_day(); state.sync_display(); CalendarResult::Handled }
        CalendarAction::PrevDay => { state.selected = state.selected.prev_day(); state.sync_display(); CalendarResult::Handled }
        CalendarAction::NextWeek => { state.selected = state.selected.add_days(7); state.sync_display(); CalendarResult::Handled }
        CalendarAction::PrevWeek => { state.selected = state.selected.add_days(-7); state.sync_display(); CalendarResult::Handled }
        CalendarAction::NextMonth => {
            let (y, m) = if state.display_month == 12 { (state.display_year+1, 1) } else { (state.display_year, state.display_month+1) };
            let dim = CalDate::new(y, m, 1).days_in_month();
            state.selected = CalDate::new(y, m, state.selected.day.min(dim));
            state.sync_display(); CalendarResult::Handled
        }
        CalendarAction::PrevMonth => {
            let (y, m) = if state.display_month == 1 { (state.display_year-1, 12) } else { (state.display_year, state.display_month-1) };
            let dim = CalDate::new(y, m, 1).days_in_month();
            state.selected = CalDate::new(y, m, state.selected.day.min(dim));
            state.sync_display(); CalendarResult::Handled
        }
        CalendarAction::Confirm => CalendarResult::Confirmed(state.selected),
        CalendarAction::Cancel => CalendarResult::Cancelled,
    }
}

/// The calendar grid widget.
pub struct CalendarGrid<'a> {
    style: CalendarStyle,
    block: Option<Block<'a>>,
}

impl<'a> CalendarGrid<'a> {
    pub fn new(style: CalendarStyle) -> Self { Self { style, block: None } }
    pub fn block(mut self, block: Block<'a>) -> Self { self.block = Some(block); self }
}

impl StatefulWidget for CalendarGrid<'_> {
    type State = CalendarGridState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else { area };
        if inner.width < 21 || inner.height < 4 { return; }

        let first = CalDate::new(state.display_year, state.display_month, 1);
        let dim = first.days_in_month();
        let col_w = inner.width / 7;

        // Title
        let title = format!("{} {}", first.month_name(), first.year);
        buf.set_line(inner.x, inner.y, &Line::from(Span::styled(title, self.style.title)), inner.width);

        // Day headers
        let names = ["Mo","Tu","We","Th","Fr","Sa","Su"];
        for (i, n) in names.iter().enumerate() {
            let x = inner.x + (i as u16) * col_w;
            buf.set_line(x, inner.y + 1, &Line::from(Span::styled(*n, self.style.weekday_header)), col_w);
        }

        // Day cells
        for d in 1..=dim {
            let date = CalDate::new(first.year, first.month, d);
            let wd = date.weekday();
            let week_row = (first.weekday() + d - 1) / 7;
            let x = inner.x + (wd as u16) * col_w;
            let y = inner.y + 2 + week_row as u16;
            if y >= inner.y + inner.height { break; }

            let sty = if date == state.selected { self.style.selected }
                      else if date == state.today { self.style.today }
                      else if wd >= 5 { self.style.weekend }
                      else { self.style.body };
            buf.set_line(x, y, &Line::from(Span::styled(format!("{:2}", d), sty)), col_w);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigate_next_day_across_month() {
        let mut state = CalendarGridState::new(CalDate::new(2026, 3, 31));
        handle_calendar_action(&mut state, CalendarAction::NextDay);
        assert_eq!(state.selected, CalDate::new(2026, 4, 1));
        assert_eq!(state.display_month, 4);
    }

    #[test]
    fn navigate_prev_month() {
        let mut state = CalendarGridState::new(CalDate::new(2026, 3, 31));
        handle_calendar_action(&mut state, CalendarAction::PrevMonth);
        // March 31 → Feb (28 days) → clamped to 28
        assert_eq!(state.selected, CalDate::new(2026, 2, 28));
    }

    #[test]
    fn confirm_returns_date() {
        let state_date = CalDate::new(2026, 6, 15);
        let mut state = CalendarGridState::new(state_date);
        assert_eq!(handle_calendar_action(&mut state, CalendarAction::Confirm), CalendarResult::Confirmed(state_date));
    }

    #[test]
    fn feb_leap_year() {
        assert_eq!(CalDate::new(2024, 2, 1).days_in_month(), 29);
        assert_eq!(CalDate::new(2026, 2, 1).days_in_month(), 28);
    }
}
