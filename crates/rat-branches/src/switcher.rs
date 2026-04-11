//! Generic branch switcher — floating overlay for quick tree node switching with filtering
//!
//! Provides a type-ahead filtered list picker for tree nodes, suitable for
//! branch switching or general node selection scenarios.

use ratatui::Frame;
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

use crate::tree::{TreeNode, find_leaves, walk_to_root};

/// An item for the switcher overlay
#[derive(Debug, Clone)]
pub struct SwitcherItem {
    /// Node ID
    pub node_id: usize,
    /// Display name
    pub name: String,
    /// Preview text (summary of content)
    pub preview: String,
    /// Whether this item is currently active/selected
    pub is_active: bool,
    /// Metadata counts (e.g., message count, token count)
    pub metadata: Vec<(String, usize)>,
}

impl SwitcherItem {
    /// Create a new switcher item
    pub fn new(node_id: usize, name: String, preview: String, is_active: bool) -> Self {
        Self {
            node_id,
            name,
            preview,
            is_active,
            metadata: Vec::new(),
        }
    }

    /// Add metadata (label, value) pair
    pub fn add_metadata(mut self, label: impl Into<String>, value: usize) -> Self {
        self.metadata.push((label.into(), value));
        self
    }
}

/// Pure data model for tree node switcher with filtering
#[derive(Debug, Default)]
pub struct NodeSwitcherModel {
    /// All items
    pub items: Vec<SwitcherItem>,
    /// Current filter text
    pub filter: String,
    /// Selected index in the filtered list
    pub selected: usize,
    /// Whether the switcher is visible
    pub visible: bool,
}

/// Generic tree node switcher with filtering
#[derive(Debug, Default)]
pub struct NodeSwitcher {
    /// The data model
    pub model: NodeSwitcherModel,
}

impl NodeSwitcherModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the switcher with nodes from a tree
    ///
    /// The `to_item` function converts each leaf node to a `SwitcherItem`
    pub fn open<N: TreeNode>(
        &mut self,
        nodes: &[N],
        to_item: impl Fn(&N, Vec<usize>) -> SwitcherItem,
    ) {
        let leaves = find_leaves(nodes);

        self.items = leaves
            .iter()
            .filter_map(|&leaf_id| {
                let node = nodes.iter().find(|n| n.id() == leaf_id)?;
                let path = walk_to_root(leaf_id, nodes);
                Some(to_item(node, path))
            })
            .collect();

        // Sort: active first, then by node ID descending (most recent first)
        self.items.sort_by(|a, b| {
            b.is_active
                .cmp(&a.is_active)
                .then(b.node_id.cmp(&a.node_id))
        });

        self.filter.clear();
        self.selected = 0;
        self.visible = true;
    }

    /// Close the switcher
    pub fn close(&mut self) {
        self.visible = false;
        self.filter.clear();
    }

    /// Get filtered items based on current filter text
    pub fn filtered_items(&self) -> Vec<&SwitcherItem> {
        let filter_lower = self.filter.to_lowercase();
        self.items
            .iter()
            .filter(|item| {
                filter_lower.is_empty()
                    || item.name.to_lowercase().contains(&filter_lower)
                    || item.preview.to_lowercase().contains(&filter_lower)
            })
            .collect()
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        let max = self.filtered_items().len().saturating_sub(1);
        self.selected = (self.selected + 1).min(max);
    }

    /// Type a character into the filter
    pub fn type_char(&mut self, c: char) {
        self.filter.push(c);
        self.selected = 0;
    }

    /// Delete the last filter character
    pub fn backspace(&mut self) {
        self.filter.pop();
        self.selected = 0;
    }

    /// Get the selected item's node ID
    pub fn selected_node_id(&self) -> Option<usize> {
        let filtered = self.filtered_items();
        filtered.get(self.selected).map(|item| item.node_id)
    }
}

impl NodeSwitcher {
    pub fn new() -> Self {
        Self {
            model: NodeSwitcherModel::new(),
        }
    }

    /// Open the switcher with nodes from a tree
    ///
    /// The `to_item` function converts each leaf node to a `SwitcherItem`
    pub fn open<N: TreeNode>(
        &mut self,
        nodes: &[N],
        to_item: impl Fn(&N, Vec<usize>) -> SwitcherItem,
    ) {
        self.model.open(nodes, to_item);
    }

    /// Close the switcher
    pub fn close(&mut self) {
        self.model.close();
    }

    /// Get filtered items based on current filter text
    pub fn filtered_items(&self) -> Vec<&SwitcherItem> {
        self.model.filtered_items()
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        self.model.move_up();
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        self.model.move_down();
    }

    /// Type a character into the filter
    pub fn type_char(&mut self, c: char) {
        self.model.type_char(c);
    }

    /// Delete the last filter character
    pub fn backspace(&mut self) {
        self.model.backspace();
    }

    /// Get the selected item's node ID
    pub fn selected_node_id(&self) -> Option<usize> {
        self.model.selected_node_id()
    }

    /// Render the switcher as a floating overlay
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.model.visible {
            return;
        }

        let width = 60.min(area.width.saturating_sub(4));
        let height = 16.min(area.height.saturating_sub(4));
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let popup_area = Rect::new(x, y, width, height);

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(Span::styled(
                " Switch Node ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        if inner.height < 2 || inner.width < 4 {
            return;
        }

        // Filter input line
        let filter_line = Line::from(vec![
            Span::styled(" Filter: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&self.model.filter, Style::default().fg(Color::White)),
            Span::styled(
                "_",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
        ]);

        let filter_area = Rect::new(inner.x, inner.y, inner.width, 1);
        frame.render_widget(Paragraph::new(filter_line), filter_area);

        // Item list
        let list_area = Rect::new(inner.x, inner.y + 1, inner.width, inner.height - 1);
        let filtered = self.model.filtered_items();

        let mut lines = Vec::new();
        for (i, item) in filtered.iter().enumerate() {
            let is_selected = i == self.model.selected;

            let bg = if is_selected {
                Color::DarkGray
            } else {
                Color::Reset
            };
            let fg = if is_selected {
                Color::White
            } else {
                Color::Gray
            };

            let marker = if item.is_active {
                Span::styled("● ", Style::default().fg(Color::Green).bg(bg))
            } else {
                Span::styled("○ ", Style::default().fg(Color::DarkGray).bg(bg))
            };

            let name = Span::styled(
                &item.name,
                Style::default().fg(fg).bg(bg).add_modifier(if is_selected {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
            );

            // Metadata display
            let meta_text = if !item.metadata.is_empty() {
                let parts: Vec<String> = item
                    .metadata
                    .iter()
                    .map(|(label, value)| format!("{} {}", value, label))
                    .collect();
                format!(" ({})", parts.join(", "))
            } else {
                String::new()
            };

            let meta = Span::styled(meta_text, Style::default().fg(Color::DarkGray).bg(bg));

            lines.push(Line::from(vec![
                Span::styled(" ", Style::default().bg(bg)),
                marker,
                name,
                meta,
            ]));

            // Preview line
            lines.push(Line::from(vec![
                Span::styled("   ", Style::default().bg(bg)),
                Span::styled(&item.preview, Style::default().fg(Color::DarkGray).bg(bg)),
            ]));
        }

        if filtered.is_empty() {
            lines.push(Line::from(Span::styled(
                " No matching items",
                Style::default().fg(Color::DarkGray),
            )));
        }

        // Scroll to keep selected visible
        let visible_height = list_area.height as usize;
        let selected_visual = self.model.selected * 2; // 2 lines per entry
        let scroll = if selected_visual >= visible_height {
            (selected_visual - visible_height / 2) as u16
        } else {
            0
        };

        frame.render_widget(
            Paragraph::new(lines)
                .scroll((scroll, 0))
                .wrap(Wrap { trim: false }),
            list_area,
        );
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestNode {
        id: usize,
        parent: Option<usize>,
        content: String,
    }

    impl TestNode {
        fn new(id: usize, content: &str, parent: Option<usize>) -> Self {
            Self {
                id,
                parent,
                content: content.to_string(),
            }
        }
    }

    impl TreeNode for TestNode {
        fn id(&self) -> usize {
            self.id
        }

        fn parent_id(&self) -> Option<usize> {
            self.parent
        }
    }

    fn node_to_item(node: &TestNode, path: Vec<usize>) -> SwitcherItem {
        SwitcherItem::new(
            node.id(),
            format!("branch-{}", node.id()),
            node.content.clone(),
            node.id() == 1, // make node 1 active for testing
        )
        .add_metadata("msgs", path.len())
    }

    #[test]
    fn open_discovers_leaves() {
        let nodes = vec![
            TestNode::new(0, "root", None),
            TestNode::new(1, "branch-a", Some(0)),
            TestNode::new(2, "branch-b", Some(0)),
        ];

        let mut switcher = NodeSwitcher::new();
        switcher.open(&nodes, node_to_item);

        assert!(switcher.model.visible);
        assert_eq!(switcher.model.items.len(), 2);
        // Active item should sort first
        assert!(switcher.model.items[0].is_active);
        assert_eq!(switcher.model.items[0].node_id, 1);
    }

    #[test]
    fn filter_narrows_results() {
        let nodes = vec![
            TestNode::new(0, "root", None),
            TestNode::new(1, "implement auth", Some(0)),
            TestNode::new(2, "fix bug in parser", Some(0)),
        ];

        let mut switcher = NodeSwitcher::new();
        switcher.open(&nodes, node_to_item);
        assert_eq!(switcher.filtered_items().len(), 2);

        switcher.type_char('a');
        switcher.type_char('u');
        switcher.type_char('t');
        switcher.type_char('h');
        // "auth" should match "implement auth"
        let filtered = switcher.filtered_items();
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].preview.contains("auth"));
    }

    #[test]
    fn backspace_widens_filter() {
        let nodes = vec![
            TestNode::new(0, "root", None),
            TestNode::new(1, "alpha", Some(0)),
            TestNode::new(2, "beta", Some(0)),
        ];

        let mut switcher = NodeSwitcher::new();
        switcher.open(&nodes, node_to_item);

        switcher.type_char('a');
        switcher.type_char('l');
        assert_eq!(switcher.filtered_items().len(), 1);

        switcher.backspace();
        switcher.backspace();
        assert_eq!(switcher.filtered_items().len(), 2);
    }

    #[test]
    fn navigation_clamps() {
        let nodes = vec![
            TestNode::new(0, "root", None),
            TestNode::new(1, "a", Some(0)),
            TestNode::new(2, "b", Some(0)),
        ];

        let mut switcher = NodeSwitcher::new();
        switcher.open(&nodes, node_to_item);

        assert_eq!(switcher.model.selected, 0);
        switcher.move_down();
        assert_eq!(switcher.model.selected, 1);
        switcher.move_down();
        assert_eq!(switcher.model.selected, 1); // clamped

        switcher.move_up();
        assert_eq!(switcher.model.selected, 0);
        switcher.move_up();
        assert_eq!(switcher.model.selected, 0); // clamped
    }

    #[test]
    fn selected_node_id_returns_correct() {
        let nodes = vec![
            TestNode::new(0, "root", None),
            TestNode::new(1, "a", Some(0)),
            TestNode::new(2, "b", Some(0)),
        ];

        let mut switcher = NodeSwitcher::new();
        switcher.open(&nodes, node_to_item);

        assert_eq!(switcher.selected_node_id(), Some(1)); // active first
        switcher.move_down();
        assert_eq!(switcher.selected_node_id(), Some(2));
    }

    #[test]
    fn close_resets_state() {
        let mut switcher = NodeSwitcher::new();
        switcher.model.visible = true;
        switcher.model.filter = "test".to_string();
        switcher.close();
        assert!(!switcher.model.visible);
        assert!(switcher.model.filter.is_empty());
    }

    #[test]
    fn empty_switcher_selected_is_none() {
        let switcher = NodeSwitcher::new();
        assert!(switcher.selected_node_id().is_none());
    }
}
