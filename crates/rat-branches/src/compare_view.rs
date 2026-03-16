//! Branch comparison view — side-by-side diff of two tree branches
//!
//! Shows the divergence point (last common ancestor) at the top, then
//! unique blocks from each branch in a split-pane layout. Provides
//! navigation and actions.

use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;

use crate::compare::{BranchComparison, CompareBlock};

/// Branch comparison overlay state for TUI rendering
#[derive(Debug, Default)]
pub struct BranchCompareView {
    /// The comparison data (None when not open)
    pub comparison: Option<BranchComparison>,
    /// Whether the view is visible
    pub visible: bool,
    /// Which pane is focused (false = left/A, true = right/B)
    pub right_focused: bool,
    /// Scroll offset for the left pane
    pub scroll_a: usize,
    /// Scroll offset for the right pane
    pub scroll_b: usize,
}

impl BranchCompareView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the comparison view with comparison data
    pub fn open(&mut self, comparison: BranchComparison) {
        self.comparison = Some(comparison);
        self.visible = true;
        self.right_focused = false;
        self.scroll_a = 0;
        self.scroll_b = 0;
    }

    /// Close the view
    pub fn close(&mut self) {
        self.visible = false;
        self.comparison = None;
    }

    /// Scroll the focused pane down
    pub fn scroll_down(&mut self) {
        if let Some(cmp) = &self.comparison {
            if self.right_focused {
                let max = cmp.branch_b.len().saturating_sub(1);
                self.scroll_b = (self.scroll_b + 1).min(max);
            } else {
                let max = cmp.branch_a.len().saturating_sub(1);
                self.scroll_a = (self.scroll_a + 1).min(max);
            }
        }
    }

    /// Scroll the focused pane up
    pub fn scroll_up(&mut self) {
        if self.right_focused {
            self.scroll_b = self.scroll_b.saturating_sub(1);
        } else {
            self.scroll_a = self.scroll_a.saturating_sub(1);
        }
    }

    /// Toggle focus between left and right pane
    pub fn toggle_focus(&mut self) {
        self.right_focused = !self.right_focused;
    }

    /// Get the leaf ID of the focused branch
    pub fn focused_leaf_id(&self) -> Option<usize> {
        self.comparison.as_ref().map(|c| {
            if self.right_focused {
                c.leaf_b
            } else {
                c.leaf_a
            }
        })
    }

    /// Render the comparison view as a floating overlay
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }
        
        let cmp = match &self.comparison {
            Some(c) => c,
            None => return,
        };

        // Size: 80% width, 80% height, centered
        let width = (area.width * 80 / 100).max(50).min(area.width.saturating_sub(4));
        let height = (area.height * 80 / 100).max(15).min(area.height.saturating_sub(4));
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let popup_area = Rect::new(x, y, width, height);

        frame.render_widget(Clear, popup_area);

        let outer = Block::default()
            .title(Span::styled(
                " Branch Comparison ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = outer.inner(popup_area);
        frame.render_widget(outer, popup_area);

        if inner.height < 5 || inner.width < 10 {
            return;
        }

        // Top: divergence info (2 lines)
        let div_area = Rect::new(inner.x, inner.y, inner.width, 2);
        let div_lines = vec![
            Line::from(vec![
                Span::styled(" Diverges at: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    cmp.divergence_id
                        .map(|id| format!("#{}", id))
                        .unwrap_or_else(|| "root".to_string()),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!(" — {}", cmp.divergence_summary),
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(vec![Span::styled(
                " ←/→: pane  j/k: scroll  s: switch  q: close",
                Style::default().fg(Color::DarkGray),
            )]),
        ];
        frame.render_widget(
            Paragraph::new(div_lines).wrap(Wrap { trim: false }),
            div_area,
        );

        // Split remaining area into two panes
        let pane_area = Rect::new(inner.x, inner.y + 2, inner.width, inner.height - 2);
        let panes = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(pane_area);

        render_comparison_pane(
            frame,
            &cmp.name_a,
            &cmp.branch_a,
            cmp.tokens_a,
            self.scroll_a,
            !self.right_focused,
            panes[0],
        );
        render_comparison_pane(
            frame,
            &cmp.name_b,
            &cmp.branch_b,
            cmp.tokens_b,
            self.scroll_b,
            self.right_focused,
            panes[1],
        );
    }
}

/// Render one pane of the comparison
#[allow(clippy::too_many_arguments)]
fn render_comparison_pane(
    frame: &mut Frame,
    name: &str,
    blocks: &[CompareBlock],
    total_tokens: usize,
    scroll: usize,
    focused: bool,
    area: Rect,
) {
    let border_color = if focused { Color::Cyan } else { Color::DarkGray };
    let title = format!(" {} ({} unique, {}tok) ", name, blocks.len(), total_tokens);

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if blocks.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "(no unique blocks)",
                Style::default().fg(Color::DarkGray),
            )),
            inner,
        );
        return;
    }

    let mut lines = Vec::new();
    for (i, b) in blocks.iter().enumerate().skip(scroll) {
        let num_style = if focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        lines.push(Line::from(vec![
            Span::styled(format!("#{} ", b.id), num_style),
            Span::styled(
                &b.preview,
                Style::default().fg(if i == scroll {
                    Color::White
                } else {
                    Color::Gray
                }),
            ),
        ]));

        // Detail counts (compact stats line)
        if !b.detail_counts.is_empty() {
            let stats: Vec<String> = b.detail_counts
                .iter()
                .map(|(label, count)| {
                    // Abbreviate common labels
                    let short_label = match label.as_str() {
                        "responses" => "r",
                        "tools" => "t", 
                        "tool_calls" => "t",
                        _ => label,
                    };
                    format!("{}{}", count, short_label)
                })
                .collect();
            
            let mut stats_text = format!("  {}", stats.join(" "));
            if b.tokens > 0 {
                stats_text.push_str(&format!(" {}tok", b.tokens));
            }
            
            lines.push(Line::from(Span::styled(
                stats_text,
                Style::default().fg(Color::DarkGray),
            )));
        } else if b.tokens > 0 {
            lines.push(Line::from(Span::styled(
                format!("  {}tok", b.tokens),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare::CompareBlock;

    fn make_comparison() -> BranchComparison {
        BranchComparison {
            divergence_id: Some(0),
            divergence_summary: "root".to_string(),
            branch_a: vec![CompareBlock::new(1, "branch-a".to_string(), 200)
                .add_detail_count("responses", 2)
                .add_detail_count("tools", 1)],
            branch_b: vec![CompareBlock::new(2, "branch-b".to_string(), 150)
                .add_detail_count("responses", 1)],
            leaf_a: 1,
            leaf_b: 2,
            name_a: "Branch A".to_string(),
            name_b: "Branch B".to_string(),
            tokens_a: 200,
            tokens_b: 150,
        }
    }

    #[test]
    fn view_toggle_focus() {
        let mut view = BranchCompareView::new();
        assert!(!view.right_focused);
        view.toggle_focus();
        assert!(view.right_focused);
        view.toggle_focus();
        assert!(!view.right_focused);
    }

    #[test]
    fn view_scroll_clamps() {
        let comparison = make_comparison();
        let mut view = BranchCompareView::new();
        view.open(comparison);

        // Each branch has 1 unique block
        view.scroll_down();
        assert_eq!(view.scroll_a, 0); // clamped (only 1 block)

        // Scroll up from 0 stays at 0
        view.scroll_up();
        assert_eq!(view.scroll_a, 0);
    }

    #[test]
    fn focused_leaf_id_tracks_pane() {
        let comparison = make_comparison();
        let mut view = BranchCompareView::new();
        view.open(comparison);

        assert_eq!(view.focused_leaf_id(), Some(1)); // left focused
        view.toggle_focus();
        assert_eq!(view.focused_leaf_id(), Some(2)); // right focused
    }

    #[test]
    fn close_clears_state() {
        let comparison = make_comparison();
        let mut view = BranchCompareView::new();
        view.open(comparison);
        assert!(view.visible);
        assert!(view.comparison.is_some());

        view.close();
        assert!(!view.visible);
        assert!(view.comparison.is_none());
    }

    #[test]
    fn open_sets_initial_state() {
        let comparison = make_comparison();
        let mut view = BranchCompareView::new();
        view.open(comparison);
        
        assert!(view.visible);
        assert!(view.comparison.is_some());
        assert!(!view.right_focused);
        assert_eq!(view.scroll_a, 0);
        assert_eq!(view.scroll_b, 0);
    }
}