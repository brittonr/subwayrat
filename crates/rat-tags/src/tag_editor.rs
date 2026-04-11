//! Tag editor widget with completion popup.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, StatefulWidget, Widget};

use crate::tag_model::is_valid_tag;

pub struct TagEditorState {
    pub tags: Vec<String>,
    pub input: String,
    pub vocabulary: Vec<String>,
    pub popup_items: Vec<String>,
    pub popup_selected: usize,
    pub popup_visible: bool,
}

impl TagEditorState {
    pub fn new(vocabulary: Vec<String>) -> Self {
        Self {
            tags: Vec::new(),
            input: String::new(),
            vocabulary,
            popup_items: Vec::new(),
            popup_selected: 0,
            popup_visible: false,
        }
    }

    fn update_popup(&mut self) {
        if self.input.is_empty() {
            self.popup_items.clear();
            self.popup_visible = false;
            return;
        }
        let q = self.input.to_lowercase();
        self.popup_items = self
            .vocabulary
            .iter()
            .filter(|v| v.to_lowercase().contains(&q) && !self.tags.contains(v))
            .cloned()
            .collect();
        self.popup_visible = !self.popup_items.is_empty();
        self.popup_selected = 0;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagAction {
    TypeChar(char),
    Backspace,
    SelectNext,
    SelectPrev,
    AcceptSuggestion,
    AcceptInput,
    RemoveLast,
    Close,
}

pub fn handle_tag_action(state: &mut TagEditorState, action: TagAction) {
    match action {
        TagAction::TypeChar(c) => {
            state.input.push(c);
            state.update_popup();
        }
        TagAction::Backspace => {
            if state.input.is_empty() {
                state.tags.pop();
            } else {
                state.input.pop();
                state.update_popup();
            }
        }
        TagAction::SelectNext => {
            if !state.popup_items.is_empty() {
                state.popup_selected = (state.popup_selected + 1).min(state.popup_items.len() - 1);
            }
        }
        TagAction::SelectPrev => {
            state.popup_selected = state.popup_selected.saturating_sub(1);
        }
        TagAction::AcceptSuggestion => {
            if let Some(tag) = state.popup_items.get(state.popup_selected).cloned() {
                state.tags.push(tag);
                state.input.clear();
                state.update_popup();
            }
        }
        TagAction::AcceptInput => {
            let tag = state.input.trim().to_string();
            if is_valid_tag(&tag) && !state.tags.contains(&tag) {
                state.tags.push(tag);
                state.input.clear();
                state.update_popup();
            }
        }
        TagAction::RemoveLast => {
            state.tags.pop();
        }
        TagAction::Close => {
            state.popup_visible = false;
        }
    }
}

#[derive(Debug, Clone)]
pub struct TagStyle {
    pub tag_chip: Style,
    pub input: Style,
    pub popup_normal: Style,
    pub popup_selected: Style,
    pub body: Style,
}

impl Default for TagStyle {
    fn default() -> Self {
        Self {
            tag_chip: Style::default().fg(Color::Black).bg(Color::Cyan),
            input: Style::default(),
            popup_normal: Style::default(),
            popup_selected: Style::default().bg(Color::Rgb(40, 40, 60)),
            body: Style::default(),
        }
    }
}

pub struct TagEditor<'a> {
    style: TagStyle,
    block: Option<Block<'a>>,
}

impl<'a> TagEditor<'a> {
    pub fn new(style: TagStyle) -> Self {
        Self { style, block: None }
    }
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl StatefulWidget for TagEditor<'_> {
    type State = TagEditorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };
        if inner.height == 0 {
            return;
        }

        // Tag chips + input on first line
        let mut spans: Vec<Span<'static>> = Vec::new();
        for tag in &state.tags {
            spans.push(Span::styled(format!(" {} ", tag), self.style.tag_chip));
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled(state.input.clone(), self.style.input));
        spans.push(Span::styled("▎", self.style.input));
        buf.set_line(inner.x, inner.y, &Line::from(spans), inner.width);

        // Popup below
        if state.popup_visible {
            for (i, item) in state.popup_items.iter().enumerate() {
                let y = inner.y + 1 + i as u16;
                if y >= inner.y + inner.height {
                    break;
                }
                let is_sel = i == state.popup_selected;
                let sty = if is_sel {
                    self.style.popup_selected
                } else {
                    self.style.popup_normal
                };
                if is_sel {
                    for x in inner.x..inner.x + inner.width {
                        buf[(x, y)].set_style(sty);
                    }
                }
                buf.set_line(
                    inner.x,
                    y,
                    &Line::from(Span::styled(format!("  {}", item), sty)),
                    inner.width,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_to_filter() {
        let mut state =
            TagEditorState::new(vec!["work".into(), "personal".into(), "workout".into()]);
        handle_tag_action(&mut state, TagAction::TypeChar('w'));
        handle_tag_action(&mut state, TagAction::TypeChar('o'));
        assert!(state.popup_visible);
        assert_eq!(state.popup_items, vec!["work", "workout"]);
    }

    #[test]
    fn accept_suggestion() {
        let mut state = TagEditorState::new(vec!["work".into()]);
        handle_tag_action(&mut state, TagAction::TypeChar('w'));
        handle_tag_action(&mut state, TagAction::AcceptSuggestion);
        assert_eq!(state.tags, vec!["work"]);
        assert!(state.input.is_empty());
    }

    #[test]
    fn accept_non_vocabulary_tag() {
        let mut state = TagEditorState::new(vec![]);
        state.input = "newtag".into();
        handle_tag_action(&mut state, TagAction::AcceptInput);
        assert_eq!(state.tags, vec!["newtag"]);
    }

    #[test]
    fn backspace_empty_removes_last() {
        let mut state = TagEditorState::new(vec![]);
        state.tags = vec!["a".into(), "b".into()];
        handle_tag_action(&mut state, TagAction::Backspace);
        assert_eq!(state.tags, vec!["a"]);
    }
}
