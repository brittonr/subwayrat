//! Property drawer editor: key-value pair list.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, StatefulWidget, Widget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode { None, Key, Value }

pub struct PropertyEditorState {
    pub properties: Vec<(String, String)>,
    pub selected: usize,
    pub edit_mode: EditMode,
}

impl PropertyEditorState {
    pub fn new(properties: Vec<(String, String)>) -> Self {
        Self { properties, selected: 0, edit_mode: EditMode::None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyAction {
    SelectNext, SelectPrev, EditKey, EditValue,
    TypeChar(char), Backspace,
    AddProperty, DeleteProperty, Confirm, Cancel,
}

pub fn handle_property_action(state: &mut PropertyEditorState, action: PropertyAction) {
    match action {
        PropertyAction::SelectNext => {
            state.edit_mode = EditMode::None;
            state.selected = (state.selected + 1).min(state.properties.len().saturating_sub(1));
        }
        PropertyAction::SelectPrev => {
            state.edit_mode = EditMode::None;
            state.selected = state.selected.saturating_sub(1);
        }
        PropertyAction::EditKey => { state.edit_mode = EditMode::Key; }
        PropertyAction::EditValue => { state.edit_mode = EditMode::Value; }
        PropertyAction::TypeChar(c) => {
            if let Some(prop) = state.properties.get_mut(state.selected) {
                match state.edit_mode {
                    EditMode::Key => prop.0.push(c),
                    EditMode::Value => prop.1.push(c),
                    EditMode::None => {}
                }
            }
        }
        PropertyAction::Backspace => {
            if let Some(prop) = state.properties.get_mut(state.selected) {
                match state.edit_mode {
                    EditMode::Key => { prop.0.pop(); }
                    EditMode::Value => { prop.1.pop(); }
                    EditMode::None => {}
                }
            }
        }
        PropertyAction::AddProperty => {
            state.properties.push((String::new(), String::new()));
            state.selected = state.properties.len() - 1;
            state.edit_mode = EditMode::Key;
        }
        PropertyAction::DeleteProperty => {
            if !state.properties.is_empty() {
                state.properties.remove(state.selected);
                if state.selected >= state.properties.len() && !state.properties.is_empty() {
                    state.selected = state.properties.len() - 1;
                }
                state.edit_mode = EditMode::None;
            }
        }
        PropertyAction::Confirm => { state.edit_mode = EditMode::None; }
        PropertyAction::Cancel => { state.edit_mode = EditMode::None; }
    }
}

#[derive(Debug, Clone)]
pub struct PropertyStyle {
    pub key: Style,
    pub value: Style,
    pub separator: Style,
    pub selected: Style,
    pub edit_highlight: Style,
    pub body: Style,
}

impl Default for PropertyStyle {
    fn default() -> Self {
        Self {
            key: Style::default().fg(Color::Cyan),
            value: Style::default(),
            separator: Style::default().fg(Color::DarkGray),
            selected: Style::default().bg(Color::Rgb(40, 40, 60)),
            edit_highlight: Style::default().add_modifier(Modifier::REVERSED),
            body: Style::default(),
        }
    }
}

pub struct PropertyEditor<'a> {
    style: PropertyStyle,
    block: Option<Block<'a>>,
}

impl<'a> PropertyEditor<'a> {
    pub fn new(style: PropertyStyle) -> Self { Self { style, block: None } }
    pub fn block(mut self, block: Block<'a>) -> Self { self.block = Some(block); self }
}

impl StatefulWidget for PropertyEditor<'_> {
    type State = PropertyEditorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area); block.clone().render(area, buf); inner
        } else { area };
        if inner.height == 0 { return; }

        for (i, (key, value)) in state.properties.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height { break; }
            let is_sel = i == state.selected;

            if is_sel {
                for x in inner.x..inner.x + inner.width { buf[(x, y)].set_style(self.style.selected); }
            }

            let ks = if is_sel && state.edit_mode == EditMode::Key { self.style.edit_highlight } else { self.style.key };
            let vs = if is_sel && state.edit_mode == EditMode::Value { self.style.edit_highlight } else { self.style.value };

            let line = Line::from(vec![
                Span::styled(format!("  {}", key), ks),
                Span::styled(": ", self.style.separator),
                Span::styled(value.clone(), vs),
            ]);
            buf.set_line(inner.x, y, &line, inner.width);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_edit_property() {
        let mut state = PropertyEditorState::new(vec![]);
        handle_property_action(&mut state, PropertyAction::AddProperty);
        assert_eq!(state.properties.len(), 1);
        assert_eq!(state.edit_mode, EditMode::Key);
        handle_property_action(&mut state, PropertyAction::TypeChar('I'));
        handle_property_action(&mut state, PropertyAction::TypeChar('D'));
        assert_eq!(state.properties[0].0, "ID");
    }

    #[test]
    fn delete_property() {
        let mut state = PropertyEditorState::new(vec![
            ("ID".into(), "abc".into()),
            ("EFFORT".into(), "45".into()),
        ]);
        state.selected = 1;
        handle_property_action(&mut state, PropertyAction::DeleteProperty);
        assert_eq!(state.properties.len(), 1);
        assert_eq!(state.properties[0].0, "ID");
    }
}
