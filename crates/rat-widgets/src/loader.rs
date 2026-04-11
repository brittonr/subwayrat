//! Loading/spinner indicator

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::theme::WidgetTheme;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub struct Loader {
    frame: usize,
    pub message: String,
}

impl Loader {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            frame: 0,
            message: message.into(),
        }
    }

    pub fn tick(&mut self) {
        self.frame = (self.frame + 1) % SPINNER_FRAMES.len();
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.render_themed(frame, area, &WidgetTheme::default());
    }

    pub fn render_themed(&self, frame: &mut Frame, area: Rect, theme: &WidgetTheme) {
        let spinner = SPINNER_FRAMES[self.frame];
        let line = Line::from(vec![
            Span::styled(format!("{} ", spinner), Style::default().fg(theme.primary)),
            Span::styled(&self.message, Style::default().fg(theme.text_muted)),
        ]);
        frame.render_widget(Paragraph::new(line), area);
    }
}
