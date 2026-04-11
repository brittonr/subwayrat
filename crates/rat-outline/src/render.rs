//! Rendering the outline as a ratatui StatefulWidget.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, StatefulWidget, Widget};

use crate::fold::visible_lines;
use crate::index::{FoldState, HeadingInfo};
use crate::state::OutlineState;

/// Style configuration for the outline widget.
#[derive(Debug, Clone)]
pub struct OutlineStyle {
    /// Base text style for body lines.
    pub body: Style,
    /// Level-1 heading style.
    pub heading1: Style,
    /// Level-2 heading style.
    pub heading2: Style,
    /// Level-3+ heading style.
    pub heading3: Style,
    /// TODO keyword style.
    pub todo_keyword: Style,
    /// DONE keyword style.
    pub done_keyword: Style,
    /// Priority cookie style.
    pub priority: Style,
    /// Tag style.
    pub tags: Style,
    /// Fold indicator in gutter.
    pub fold_indicator: Style,
    /// Gutter background.
    pub gutter: Style,
    /// Cursor line highlight.
    pub cursor_line: Style,
    /// Folded indicator character.
    pub fold_char: char,
    /// Expanded indicator character.
    pub expand_char: char,
    /// Gutter width (number of columns for fold indicators).
    pub gutter_width: u16,
}

impl Default for OutlineStyle {
    fn default() -> Self {
        Self {
            body: Style::default(),
            heading1: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            heading2: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            heading3: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            todo_keyword: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            done_keyword: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            priority: Style::default().fg(Color::Magenta),
            tags: Style::default().fg(Color::DarkGray),
            fold_indicator: Style::default().fg(Color::DarkGray),
            gutter: Style::default(),
            cursor_line: Style::default().bg(Color::Rgb(40, 40, 50)),
            fold_char: '▶',
            expand_char: '▼',
            gutter_width: 2,
        }
    }
}

/// The outline widget. Construct with a style and optional block.
pub struct Outline<'a> {
    style: OutlineStyle,
    block: Option<Block<'a>>,
}

impl<'a> Outline<'a> {
    pub fn new(style: OutlineStyle) -> Self {
        Self { style, block: None }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl StatefulWidget for Outline<'_> {
    type State = OutlineState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Draw optional block, get inner area
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        state.ensure_index();

        let vis = visible_lines(&state.headings, state.line_count());
        let visible_height = inner.height as usize;
        let gw = self.style.gutter_width;
        let content_width = inner.width.saturating_sub(gw);

        // Adjust scroll to keep cursor visible
        let cursor_vis_idx = vis
            .iter()
            .position(|&l| l == state.editor.cursor_line())
            .unwrap_or(0);
        if cursor_vis_idx < state.scroll_offset {
            state.scroll_offset = cursor_vis_idx;
        }
        if cursor_vis_idx >= state.scroll_offset + visible_height {
            state.scroll_offset = cursor_vis_idx.saturating_sub(visible_height - 1);
        }

        // Render visible lines
        let display_lines =
            &vis[state.scroll_offset..vis.len().min(state.scroll_offset + visible_height)];

        for (row, &line_idx) in display_lines.iter().enumerate() {
            let y = inner.y + row as u16;
            let line_text = &state.lines()[line_idx];
            let is_cursor_line = line_idx == state.editor.cursor_line();

            // Find if this line is a heading
            let heading_info = state.headings.iter().find(|h| h.line == line_idx);

            // ── Gutter ──
            let gutter_area = Rect::new(inner.x, y, gw, 1);
            if let Some(hi) = heading_info {
                let has_children = state
                    .headings
                    .iter()
                    .any(|h| h.line > hi.line && h.level > hi.level);
                if has_children {
                    let ch = match hi.fold {
                        FoldState::Folded => self.style.fold_char,
                        _ => self.style.expand_char,
                    };
                    let span = Span::styled(format!("{} ", ch), self.style.fold_indicator);
                    buf.set_line(gutter_area.x, y, &Line::from(span), gw);
                }
            }

            // ── Content ──
            let content_area = Rect::new(inner.x + gw, y, content_width, 1);
            let line = if let Some(hi) = heading_info {
                render_heading_line(line_text, hi, &self.style)
            } else {
                Line::from(Span::styled(line_text.to_string(), self.style.body))
            };

            // Apply cursor line highlight
            if is_cursor_line {
                // Fill background
                for x in content_area.x..content_area.x + content_area.width {
                    buf[(x, y)].set_style(self.style.cursor_line);
                }
            }

            buf.set_line(content_area.x, y, &line, content_width);
        }
    }
}

/// Render a heading line with syntax highlighting for TODO, priority, tags.
fn render_heading_line(text: &str, heading: &HeadingInfo, style: &OutlineStyle) -> Line<'static> {
    let heading_style = match heading.level {
        1 => style.heading1,
        2 => style.heading2,
        _ => style.heading3,
    };

    let mut spans: Vec<Span<'static>> = Vec::new();

    // Heading marker (stars/hashes)
    let marker_end = text.bytes().take_while(|&b| b == b'*' || b == b'#').count();
    let marker = &text[..marker_end];
    spans.push(Span::styled(format!("{} ", marker), heading_style));

    // TODO keyword
    if let Some(ref kw) = heading.todo {
        let kw_style = if kw == "DONE" || kw == "CANCELLED" || kw == "CANCELED" {
            style.done_keyword
        } else {
            style.todo_keyword
        };
        spans.push(Span::styled(format!("{} ", kw), kw_style));
    }

    // Priority
    if let Some(p) = heading.priority {
        spans.push(Span::styled(format!("[#{}] ", p), style.priority));
    }

    // Title
    spans.push(Span::styled(heading.title.clone(), heading_style));

    // Tags
    if !heading.tags.is_empty() {
        let tag_str = format!(" :{}:", heading.tags.join(":"));
        spans.push(Span::styled(tag_str, style.tags));
    }

    Line::from(spans)
}
