//! Quick capture overlay: transient popup for fast note/task entry.

use rat_editor::Editor;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, StatefulWidget, Widget};

// ── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CaptureTemplate {
    pub name: String,
    pub icon: Option<char>,
    pub target_file: Option<String>,
    pub target_heading: Option<String>,
    pub initial_content: Option<String>,
    pub include_body: bool,
}

impl CaptureTemplate {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(), icon: None, target_file: None,
            target_heading: None, initial_content: None, include_body: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CaptureResult {
    pub template: CaptureTemplate,
    pub title: String,
    pub body: Option<String>,
    pub timestamp_epoch_secs: u64,
}

// ── State ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase { Closed, TemplateSelect, Editing }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditFocus { Title, Body }

pub struct CaptureState {
    pub phase: Phase,
    pub templates: Vec<CaptureTemplate>,
    pub template_idx: usize,
    pub title: String,
    pub body_editor: Editor,
    pub edit_focus: EditFocus,
    result: Option<CaptureResult>,
}

impl CaptureState {
    pub fn new() -> Self {
        Self {
            phase: Phase::Closed, templates: Vec::new(), template_idx: 0,
            title: String::new(), body_editor: Editor::new(), edit_focus: EditFocus::Title,
            result: None,
        }
    }

    pub fn open(&mut self, templates: Vec<CaptureTemplate>) {
        self.templates = templates;
        self.template_idx = 0;
        self.title.clear();
        self.body_editor.clear();
        self.edit_focus = EditFocus::Title;
        self.result = None;
        if self.templates.len() == 1 {
            self.phase = Phase::Editing;
        } else {
            self.phase = Phase::TemplateSelect;
        }
    }

    pub fn is_open(&self) -> bool { self.phase != Phase::Closed }

    pub fn take_result(&mut self) -> Option<CaptureResult> { self.result.take() }

    fn selected_template(&self) -> Option<&CaptureTemplate> {
        self.templates.get(self.template_idx)
    }
}

impl Default for CaptureState {
    fn default() -> Self { Self::new() }
}

// ── Actions ──────────────────────────────────────────────────────────────────

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    SelectNext, SelectPrev, ConfirmTemplate, Cancel,
    TitleChar(char), TitleBackspace, BodyChar(char), BodyBackspace,
    FocusNext, Confirm,
}

pub fn handle_action(state: &mut CaptureState, action: Action) {
    match state.phase {
        Phase::Closed => {}
        Phase::TemplateSelect => match action {
            Action::SelectNext => {
                state.template_idx = (state.template_idx + 1).min(state.templates.len().saturating_sub(1));
            }
            Action::SelectPrev => {
                state.template_idx = state.template_idx.saturating_sub(1);
            }
            Action::ConfirmTemplate | Action::Confirm => {
                state.phase = Phase::Editing;
            }
            Action::Cancel => {
                state.phase = Phase::Closed;
            }
            _ => {}
        },
        Phase::Editing => match action {
            Action::Cancel => {
                state.phase = Phase::Closed;
            }
            Action::FocusNext => {
                state.edit_focus = match state.edit_focus {
                    EditFocus::Title => EditFocus::Body,
                    EditFocus::Body => EditFocus::Title,
                };
            }
            Action::TitleChar(c) => { state.title.push(c); }
            Action::TitleBackspace => { state.title.pop(); }
            Action::BodyChar(c) => { state.body_editor.insert_char(c); }
            Action::BodyBackspace => { state.body_editor.delete_back(); }
            Action::Confirm => {
                if !state.title.trim().is_empty() {
                    let tmpl = state.selected_template().cloned().unwrap_or_else(|| CaptureTemplate::new(""));
                    let body_text = state.body_editor.content().join("\n");
                    state.result = Some(CaptureResult {
                        template: tmpl,
                        title: state.title.clone(),
                        body: if body_text.trim().is_empty() { None } else { Some(body_text) },
                        timestamp_epoch_secs: 0, // caller sets real timestamp
                    });
                    state.phase = Phase::Closed;
                }
            }
            _ => {}
        },
    }
}

// ── Style ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CaptureStyle {
    pub border: Style,
    pub title_bar: Style,
    pub input: Style,
    pub input_focused: Style,
    pub template_normal: Style,
    pub template_selected: Style,
    pub label: Style,
}

impl Default for CaptureStyle {
    fn default() -> Self {
        Self {
            border: Style::default().fg(Color::Cyan),
            title_bar: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            input: Style::default(),
            input_focused: Style::default().add_modifier(Modifier::REVERSED),
            template_normal: Style::default(),
            template_selected: Style::default().bg(Color::Rgb(40, 40, 60)),
            label: Style::default().fg(Color::DarkGray),
        }
    }
}

// ── Widget ───────────────────────────────────────────────────────────────────

pub struct CaptureOverlay {
    style: CaptureStyle,
    /// Fraction of available area to use (0.0-1.0).
    pub width_pct: f32,
    pub height_pct: f32,
}

impl CaptureOverlay {
    pub fn new(style: CaptureStyle) -> Self {
        Self { style, width_pct: 0.6, height_pct: 0.5 }
    }
}

impl StatefulWidget for CaptureOverlay {
    type State = CaptureState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.phase == Phase::Closed { return; }

        let w = (area.width as f32 * self.width_pct) as u16;
        let h = (area.height as f32 * self.height_pct) as u16;
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        let popup = Rect::new(x, y, w.min(area.width), h.min(area.height));

        Clear.render(popup, buf);

        let title = match state.phase {
            Phase::TemplateSelect => " Select Template ",
            Phase::Editing => {
                if let Some(t) = state.selected_template() { " Capture " } else { " Capture " }
            }
            Phase::Closed => "",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.style.border)
            .title(title);
        let inner = block.inner(popup);
        block.render(popup, buf);

        match state.phase {
            Phase::TemplateSelect => {
                for (i, tmpl) in state.templates.iter().enumerate() {
                    let row = inner.y + i as u16;
                    if row >= inner.y + inner.height { break; }
                    let is_sel = i == state.template_idx;
                    let sty = if is_sel { self.style.template_selected } else { self.style.template_normal };
                    if is_sel {
                        for cx in inner.x..inner.x + inner.width {
                            buf[(cx, row)].set_style(sty);
                        }
                    }
                    let icon = tmpl.icon.map(|c| format!("{} ", c)).unwrap_or_default();
                    let line = Line::from(Span::styled(format!("  {}{}", icon, tmpl.name), sty));
                    buf.set_line(inner.x, row, &line, inner.width);
                }
            }
            Phase::Editing => {
                // Title label + input
                let label = Line::from(Span::styled("Title:", self.style.label));
                buf.set_line(inner.x, inner.y, &label, inner.width);

                let title_sty = if state.edit_focus == EditFocus::Title { self.style.input_focused } else { self.style.input };
                let title_line = Line::from(Span::styled(format!(" {}", state.title), title_sty));
                buf.set_line(inner.x, inner.y + 1, &title_line, inner.width);

                // Body
                if state.selected_template().map(|t| t.include_body).unwrap_or(true) {
                    let body_label = Line::from(Span::styled("Body:", self.style.label));
                    buf.set_line(inner.x, inner.y + 3, &body_label, inner.width);

                    let body_sty = if state.edit_focus == EditFocus::Body { self.style.input_focused } else { self.style.input };
                    for (i, line) in state.body_editor.content().iter().enumerate() {
                        let row = inner.y + 4 + i as u16;
                        if row >= inner.y + inner.height { break; }
                        buf.set_line(inner.x, row, &Line::from(Span::styled(format!(" {}", line), body_sty)), inner.width);
                    }
                }
            }
            Phase::Closed => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_template_skips_select() {
        let mut state = CaptureState::new();
        state.open(vec![CaptureTemplate::new("Note")]);
        assert_eq!(state.phase, Phase::Editing);
    }

    #[test]
    fn multiple_templates_shows_select() {
        let mut state = CaptureState::new();
        state.open(vec![CaptureTemplate::new("Note"), CaptureTemplate::new("TODO")]);
        assert_eq!(state.phase, Phase::TemplateSelect);
    }

    #[test]
    fn confirm_with_title() {
        let mut state = CaptureState::new();
        state.open(vec![CaptureTemplate::new("Note")]);
        handle_action(&mut state, Action::TitleChar('H'));
        handle_action(&mut state, Action::TitleChar('i'));
        handle_action(&mut state, Action::Confirm);
        assert_eq!(state.phase, Phase::Closed);
        let result = state.take_result().unwrap();
        assert_eq!(result.title, "Hi");
    }

    #[test]
    fn confirm_empty_title_rejected() {
        let mut state = CaptureState::new();
        state.open(vec![CaptureTemplate::new("Note")]);
        handle_action(&mut state, Action::Confirm);
        assert_eq!(state.phase, Phase::Editing); // still open
        assert!(state.take_result().is_none());
    }

    #[test]
    fn cancel_returns_none() {
        let mut state = CaptureState::new();
        state.open(vec![CaptureTemplate::new("Note")]);
        handle_action(&mut state, Action::Cancel);
        assert_eq!(state.phase, Phase::Closed);
        assert!(state.take_result().is_none());
    }
}
