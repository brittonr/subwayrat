//! Rendering functions for search overlay and highlighting.
//!
//! Separated from core search logic to allow UI-independent testing
//! and different front-end implementations.

use ratatui::{Frame, layout::Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use crate::output_search::{OutputSearch, SearchMode};

/// Render the search overlay bar at the top-right of the messages area
pub fn render_search_overlay(search: &OutputSearch, frame: &mut Frame, area: Rect) {
    if !search.active {
        return;
    }

    let width = area.width.saturating_sub(4).min(60);
    let height = 3u16;
    let x = area.x + area.width.saturating_sub(width + 2);
    let y = area.y + 1;
    let popup = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup);

    let mode_label = match search.mode {
        SearchMode::Substring => "find",
        SearchMode::Fuzzy => "fuzzy",
    };

    let match_info = if search.query.is_empty() {
        String::new()
    } else if search.matches.is_empty() {
        " no matches".to_string()
    } else {
        format!(" {}/{}", search.current + 1, search.matches.len())
    };

    let match_color = if search.matches.is_empty() && !search.query.is_empty() {
        Color::Red
    } else {
        Color::DarkGray
    };

    let search_line = Line::from(vec![
        Span::styled(format!(" {} ", mode_label), Style::default().fg(Color::Black).bg(Color::Yellow)),
        Span::styled(" ", Style::default()),
        Span::styled(&search.query, Style::default().fg(Color::White)),
        Span::styled("\u{2588}", Style::default().fg(Color::Gray).add_modifier(Modifier::SLOW_BLINK)),
        Span::styled(match_info, Style::default().fg(match_color)),
    ]);

    let title = " Search (Ctrl+R: mode) ";
    let block =
        Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)).title(title);
    let paragraph = Paragraph::new(search_line).block(block);
    frame.render_widget(paragraph, popup);
}

/// Apply search match highlighting to rendered lines.
///
/// Returns the original base styles for each modified row so that later
/// highlighting passes (e.g. selection) can use the correct base style.
pub fn apply_search_highlights<'a>(
    lines: &mut [Line<'a>],
    plain_lines: &'a [String],
    search: &OutputSearch,
    match_style: Style,
    current_match_style: Style,
) -> Vec<Option<Style>> {
    let mut original_styles: Vec<Option<Style>> = vec![None; lines.len()];

    if search.query.is_empty() || search.matches.is_empty() {
        return original_styles;
    }

    // Walk through matches (already sorted by row) using a two-pointer approach
    let mut mi = 0;
    for row in 0..lines.len() {
        // Collect all matches on this row
        let row_start = mi;
        while mi < search.matches.len() && search.matches[mi].row == row {
            mi += 1;
        }
        if row_start == mi {
            continue; // no matches on this row
        }

        let plain = match plain_lines.get(row) {
            Some(p) if !p.is_empty() => p,
            _ => continue,
        };

        let base_style = lines[row].spans.first().map(|s| s.style).unwrap_or_default();
        original_styles[row] = Some(base_style);

        let mut spans = Vec::new();
        let mut pos = 0;

        for idx in row_start..mi {
            let m = &search.matches[idx];
            let byte_start = m.byte_start.min(plain.len());
            let byte_end = m.byte_end.min(plain.len());

            // Validate character boundaries
            if !plain.is_char_boundary(byte_start) || !plain.is_char_boundary(byte_end) {
                continue;
            }

            let style = if idx == search.current {
                current_match_style
            } else {
                match_style
            };

            // Gap before this match
            if byte_start > pos && plain.is_char_boundary(pos) {
                spans.push(Span::styled(&plain[pos..byte_start], base_style));
            }

            // The match itself
            let start = byte_start.max(pos);
            if start < byte_end && plain.is_char_boundary(start) {
                spans.push(Span::styled(&plain[start..byte_end], style));
            }
            pos = byte_end;
        }

        // Remainder after last match
        if pos < plain.len() && plain.is_char_boundary(pos) {
            spans.push(Span::styled(&plain[pos..], base_style));
        }

        if !spans.is_empty() {
            lines[row] = Line::from(spans);
        }
    }

    original_styles
}