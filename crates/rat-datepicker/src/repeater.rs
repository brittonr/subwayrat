//! RepeaterInput: org-style repeater interval (+1w, ++2m, .+3d).

use std::fmt;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, StatefulWidget, Widget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeaterMode {
    Plus,
    PlusPlus,
    DotPlus,
}

impl RepeaterMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Plus => "+",
            Self::PlusPlus => "++",
            Self::DotPlus => ".+",
        }
    }
    pub fn cycle(self) -> Self {
        match self {
            Self::Plus => Self::PlusPlus,
            Self::PlusPlus => Self::DotPlus,
            Self::DotPlus => Self::Plus,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeaterUnit {
    Day,
    Week,
    Month,
    Year,
}

impl RepeaterUnit {
    pub fn as_char(&self) -> char {
        match self {
            Self::Day => 'd',
            Self::Week => 'w',
            Self::Month => 'm',
            Self::Year => 'y',
        }
    }
    pub fn cycle(self) -> Self {
        match self {
            Self::Day => Self::Week,
            Self::Week => Self::Month,
            Self::Month => Self::Year,
            Self::Year => Self::Day,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Repeater {
    pub mode: RepeaterMode,
    pub count: u32,
    pub unit: RepeaterUnit,
}

impl fmt::Display for Repeater {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.mode.as_str(),
            self.count,
            self.unit.as_char()
        )
    }
}

pub struct RepeaterInputState {
    pub repeater: Repeater,
    pub enabled: bool,
    pub focused: RepeaterField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeaterField {
    Mode,
    Count,
    Unit,
}

impl RepeaterInputState {
    pub fn new() -> Self {
        Self {
            repeater: Repeater {
                mode: RepeaterMode::Plus,
                count: 1,
                unit: RepeaterUnit::Week,
            },
            enabled: false,
            focused: RepeaterField::Count,
        }
    }

    pub fn value(&self) -> Option<Repeater> {
        if self.enabled {
            Some(self.repeater)
        } else {
            None
        }
    }
}

impl Default for RepeaterInputState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepeaterAction {
    Toggle,
    CycleMode,
    CycleUnit,
    IncrementCount,
    DecrementCount,
    NextField,
    PrevField,
}

pub fn handle_repeater_action(state: &mut RepeaterInputState, action: RepeaterAction) {
    match action {
        RepeaterAction::Toggle => state.enabled = !state.enabled,
        RepeaterAction::CycleMode => state.repeater.mode = state.repeater.mode.cycle(),
        RepeaterAction::CycleUnit => state.repeater.unit = state.repeater.unit.cycle(),
        RepeaterAction::IncrementCount => {
            state.repeater.count = state.repeater.count.saturating_add(1).min(99)
        }
        RepeaterAction::DecrementCount => {
            state.repeater.count = state.repeater.count.saturating_sub(1).max(1)
        }
        RepeaterAction::NextField => {
            state.focused = match state.focused {
                RepeaterField::Mode => RepeaterField::Count,
                RepeaterField::Count => RepeaterField::Unit,
                RepeaterField::Unit => RepeaterField::Mode,
            };
        }
        RepeaterAction::PrevField => {
            state.focused = match state.focused {
                RepeaterField::Mode => RepeaterField::Unit,
                RepeaterField::Count => RepeaterField::Mode,
                RepeaterField::Unit => RepeaterField::Count,
            };
        }
    }
}

pub struct RepeaterInput<'a> {
    block: Option<Block<'a>>,
}

impl<'a> RepeaterInput<'a> {
    pub fn new() -> Self {
        Self { block: None }
    }
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Default for RepeaterInput<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl StatefulWidget for RepeaterInput<'_> {
    type State = RepeaterInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };
        if inner.width < 6 || inner.height == 0 {
            return;
        }

        if !state.enabled {
            buf.set_line(
                inner.x,
                inner.y,
                &Line::from(Span::raw("(none)")),
                inner.width,
            );
            return;
        }

        let r = &state.repeater;
        let ms = if state.focused == RepeaterField::Mode {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let cs = if state.focused == RepeaterField::Count {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let us = if state.focused == RepeaterField::Unit {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::styled(r.mode.as_str().to_string(), ms),
            Span::styled(r.count.to_string(), cs),
            Span::styled(r.unit.as_char().to_string(), us),
        ]);
        buf.set_line(inner.x, inner.y, &line, inner.width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_value() {
        let s = RepeaterInputState::new();
        assert_eq!(s.value(), None); // disabled by default
    }

    #[test]
    fn toggle_enables() {
        let mut s = RepeaterInputState::new();
        handle_repeater_action(&mut s, RepeaterAction::Toggle);
        assert_eq!(
            s.value(),
            Some(Repeater {
                mode: RepeaterMode::Plus,
                count: 1,
                unit: RepeaterUnit::Week
            })
        );
    }

    #[test]
    fn cycle_unit() {
        let mut s = RepeaterInputState::new();
        s.enabled = true;
        handle_repeater_action(&mut s, RepeaterAction::CycleUnit);
        assert_eq!(s.repeater.unit, RepeaterUnit::Month);
    }

    #[test]
    fn format_repeater() {
        let r = Repeater {
            mode: RepeaterMode::PlusPlus,
            count: 2,
            unit: RepeaterUnit::Month,
        };
        assert_eq!(r.to_string(), "++2m");
    }
}
