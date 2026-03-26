//! TimeInput: HH:MM time entry widget.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, StatefulWidget, Widget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeField { Hours, Minutes }

pub struct TimeInputState {
    pub hours: u8,
    pub minutes: u8,
    pub focused_field: TimeField,
    /// Partial digit buffer (for typing "14" as two keystrokes).
    digit_buf: Option<u8>,
}

impl TimeInputState {
    pub fn new(hours: u8, minutes: u8) -> Self {
        Self { hours: hours.min(23), minutes: minutes.min(59), focused_field: TimeField::Hours, digit_buf: None }
    }

    pub fn to_string(&self) -> String {
        format!("{:02}:{:02}", self.hours, self.minutes)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeAction {
    Digit(u8),
    Increment,
    Decrement,
    NextField,
    PrevField,
}

pub fn handle_time_action(state: &mut TimeInputState, action: TimeAction) {
    match action {
        TimeAction::Digit(d) if d <= 9 => {
            if let Some(first) = state.digit_buf.take() {
                let val = first * 10 + d;
                match state.focused_field {
                    TimeField::Hours => { state.hours = val.min(23); state.focused_field = TimeField::Minutes; }
                    TimeField::Minutes => { state.minutes = val.min(59); }
                }
            } else {
                state.digit_buf = Some(d);
            }
        }
        TimeAction::Increment => {
            state.digit_buf = None;
            match state.focused_field {
                TimeField::Hours => state.hours = (state.hours + 1) % 24,
                TimeField::Minutes => state.minutes = (state.minutes + 1) % 60,
            }
        }
        TimeAction::Decrement => {
            state.digit_buf = None;
            match state.focused_field {
                TimeField::Hours => state.hours = if state.hours == 0 { 23 } else { state.hours - 1 },
                TimeField::Minutes => state.minutes = if state.minutes == 0 { 59 } else { state.minutes - 1 },
            }
        }
        TimeAction::NextField => { state.digit_buf = None; state.focused_field = TimeField::Minutes; }
        TimeAction::PrevField => { state.digit_buf = None; state.focused_field = TimeField::Hours; }
        _ => {}
    }
}

pub struct TimeInput<'a> {
    block: Option<Block<'a>>,
}

impl<'a> TimeInput<'a> {
    pub fn new() -> Self { Self { block: None } }
    pub fn block(mut self, block: Block<'a>) -> Self { self.block = Some(block); self }
}

impl StatefulWidget for TimeInput<'_> {
    type State = TimeInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area); block.clone().render(area, buf); inner
        } else { area };
        if inner.width < 5 || inner.height == 0 { return; }

        let h_style = if state.focused_field == TimeField::Hours {
            Style::default().add_modifier(Modifier::REVERSED)
        } else { Style::default() };
        let m_style = if state.focused_field == TimeField::Minutes {
            Style::default().add_modifier(Modifier::REVERSED)
        } else { Style::default() };

        let line = Line::from(vec![
            Span::styled(format!("{:02}", state.hours), h_style),
            Span::raw(":"),
            Span::styled(format!("{:02}", state.minutes), m_style),
        ]);
        buf.set_line(inner.x, inner.y, &line, inner.width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_time() {
        let mut s = TimeInputState::new(0, 0);
        handle_time_action(&mut s, TimeAction::Digit(1));
        handle_time_action(&mut s, TimeAction::Digit(4));
        assert_eq!(s.hours, 14);
        assert_eq!(s.focused_field, TimeField::Minutes);
        handle_time_action(&mut s, TimeAction::Digit(3));
        handle_time_action(&mut s, TimeAction::Digit(0));
        assert_eq!(s.minutes, 30);
    }

    #[test]
    fn hours_wrap() {
        let mut s = TimeInputState::new(23, 0);
        handle_time_action(&mut s, TimeAction::Increment);
        assert_eq!(s.hours, 0);
    }

    #[test]
    fn minutes_wrap() {
        let mut s = TimeInputState::new(0, 59);
        s.focused_field = TimeField::Minutes;
        handle_time_action(&mut s, TimeAction::Increment);
        assert_eq!(s.minutes, 0);
    }

    #[test]
    fn decrement_wrap() {
        let mut s = TimeInputState::new(0, 0);
        handle_time_action(&mut s, TimeAction::Decrement);
        assert_eq!(s.hours, 23);
    }
}
