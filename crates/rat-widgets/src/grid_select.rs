//! Grid selection dialog with 2D navigation.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::theme::WidgetTheme;

#[derive(Debug, Clone)]
pub struct GridItem {
    pub label: String,
    pub color: Option<Color>,
}

impl GridItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color: None,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

pub struct GridSelect {
    pub title: String,
    pub items: Vec<GridItem>,
    pub columns: usize,
    pub selected: usize,
    pub visible: bool,
}

impl GridSelect {
    pub fn new(title: impl Into<String>, items: Vec<GridItem>, columns: usize) -> Self {
        Self {
            title: title.into(),
            items,
            columns: columns.max(1), // Ensure at least 1 column
            selected: 0,
            visible: true,
        }
    }

    pub fn move_left(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.selected + 1 < self.items.len() {
            self.selected += 1;
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(self.columns);
    }

    pub fn move_down(&mut self) {
        let new_pos = self.selected + self.columns;
        if new_pos < self.items.len() {
            self.selected = new_pos;
        } else if !self.items.is_empty() {
            // Clamp to last item
            self.selected = self.items.len() - 1;
        }
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_item(&self) -> Option<&GridItem> {
        self.items.get(self.selected)
    }

    pub fn set_selected(&mut self, idx: usize) {
        if !self.items.is_empty() {
            self.selected = idx.min(self.items.len() - 1);
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.render_themed(frame, area, &WidgetTheme::default());
    }

    pub fn render_themed(&self, frame: &mut Frame, area: Rect, theme: &WidgetTheme) {
        if !self.visible {
            return;
        }

        if self.items.is_empty() {
            self.render_empty(frame, area, theme);
            return;
        }

        // Calculate popup dimensions
        let rows = self.items.len().div_ceil(self.columns);
        let cell_width = 20; // Fixed cell width for simplicity
        let popup_width = (self.columns * cell_width + self.columns - 1 + 2) as u16; // +2 for borders
        let popup_height = (rows + 2) as u16; // +2 for borders

        // Clamp to area size
        let width = popup_width.min(area.width.saturating_sub(4));
        let height = popup_height.min(area.height.saturating_sub(4));
        
        // Center popup
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let popup = Rect::new(x, y, width, height);

        frame.render_widget(Clear, popup);
        
        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary));

        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        // Create grid layout
        self.render_grid(frame, inner, theme);
    }

    fn render_empty(&self, frame: &mut Frame, area: Rect, theme: &WidgetTheme) {
        let width = 30.min(area.width.saturating_sub(4));
        let height = 3;
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let popup = Rect::new(x, y, width, height);

        frame.render_widget(Clear, popup);
        
        let block = Block::default()
            .title(format!(" {} ", self.title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary));

        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        let line = Line::from(Span::styled("(empty)", theme.text_muted));
        frame.render_widget(Paragraph::new(line), inner);
    }

    fn render_grid(&self, frame: &mut Frame, area: Rect, theme: &WidgetTheme) {
        let rows = self.items.len().div_ceil(self.columns);
        
        // Create row constraints
        let row_constraints: Vec<Constraint> = (0..rows)
            .map(|_| Constraint::Length(1))
            .collect();

        let row_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(area);

        for row in 0..rows {
            if row >= row_layout.len() {
                break;
            }

            let row_area = row_layout[row];
            
            // Create column constraints for this row
            let items_in_row = ((row + 1) * self.columns).min(self.items.len()) - row * self.columns;
            let col_constraints: Vec<Constraint> = (0..items_in_row)
                .map(|_| Constraint::Ratio(1, items_in_row as u32))
                .collect();

            let col_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints)
                .split(row_area);

            for col in 0..items_in_row {
                let item_index = row * self.columns + col;
                if item_index >= self.items.len() {
                    break;
                }

                let item = &self.items[item_index];
                let cell_area = col_layout[col];
                
                let is_selected = item_index == self.selected;
                self.render_cell(frame, cell_area, item, is_selected, theme);
            }
        }
    }

    fn render_cell(&self, frame: &mut Frame, area: Rect, item: &GridItem, selected: bool, theme: &WidgetTheme) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let style = if selected {
            theme.highlight_style()
        } else {
            Style::default().fg(theme.text)
        };

        let mut spans = Vec::new();

        // Add color swatch if present
        if let Some(color) = item.color {
            spans.push(Span::styled("█ ", Style::default().fg(color)));
        }

        // Add label (truncate if necessary)
        let available_width = if item.color.is_some() {
            area.width.saturating_sub(2) // Account for swatch + space
        } else {
            area.width
        } as usize;

        let label = if item.label.len() > available_width {
            format!("{}…", &item.label[..available_width.saturating_sub(1)])
        } else {
            item.label.clone()
        };

        spans.push(Span::styled(label, style));

        let line = Line::from(spans);
        frame.render_widget(Paragraph::new(line), area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_select() {
        let items = vec![
            GridItem::new("Item 1"),
            GridItem::new("Item 2"),
            GridItem::new("Item 3"),
        ];
        let grid = GridSelect::new("Test", items, 2);
        
        assert_eq!(grid.title, "Test");
        assert_eq!(grid.items.len(), 3);
        assert_eq!(grid.columns, 2);
        assert_eq!(grid.selected, 0);
        assert!(grid.visible);
    }

    #[test]
    fn new_empty_grid() {
        let grid = GridSelect::new("Empty", vec![], 3);
        assert_eq!(grid.items.len(), 0);
        assert_eq!(grid.selected_index(), 0);
        assert!(grid.selected_item().is_none());
    }

    #[test]
    fn columns_minimum_one() {
        let items = vec![GridItem::new("Item")];
        let grid = GridSelect::new("Test", items, 0);
        assert_eq!(grid.columns, 1);
    }

    #[test]
    fn move_right_within_bounds() {
        let items = vec![
            GridItem::new("A"),
            GridItem::new("B"),
            GridItem::new("C"),
        ];
        let mut grid = GridSelect::new("Test", items, 3);
        
        assert_eq!(grid.selected, 0);
        grid.move_right();
        assert_eq!(grid.selected, 1);
        grid.move_right();
        assert_eq!(grid.selected, 2);
    }

    #[test]
    fn move_right_at_end_stays() {
        let items = vec![GridItem::new("A"), GridItem::new("B")];
        let mut grid = GridSelect::new("Test", items, 2);
        
        grid.set_selected(1); // Last item
        grid.move_right();
        assert_eq!(grid.selected, 1); // Should stay at last item
    }

    #[test]
    fn move_left_within_bounds() {
        let items = vec![
            GridItem::new("A"),
            GridItem::new("B"),
            GridItem::new("C"),
        ];
        let mut grid = GridSelect::new("Test", items, 3);
        
        grid.set_selected(2);
        assert_eq!(grid.selected, 2);
        grid.move_left();
        assert_eq!(grid.selected, 1);
        grid.move_left();
        assert_eq!(grid.selected, 0);
    }

    #[test]
    fn move_left_at_start_stays() {
        let items = vec![GridItem::new("A"), GridItem::new("B")];
        let mut grid = GridSelect::new("Test", items, 2);
        
        assert_eq!(grid.selected, 0);
        grid.move_left();
        assert_eq!(grid.selected, 0); // Should stay at 0
    }

    #[test]
    fn move_down_by_columns() {
        let items = vec![
            GridItem::new("A"), GridItem::new("B"), GridItem::new("C"),
            GridItem::new("D"), GridItem::new("E"), GridItem::new("F"),
        ];
        let mut grid = GridSelect::new("Test", items, 3);
        
        grid.set_selected(1); // Second item in first row
        grid.move_down();
        assert_eq!(grid.selected, 4); // Second item in second row (1 + 3)
    }

    #[test]
    fn move_down_clamps_to_last() {
        let items = vec![
            GridItem::new("A"), GridItem::new("B"), GridItem::new("C"),
            GridItem::new("D"), GridItem::new("E"), // 5 items in 3-column grid
        ];
        let mut grid = GridSelect::new("Test", items, 3);
        
        grid.set_selected(2); // Third item in first row
        grid.move_down();
        assert_eq!(grid.selected, 4); // Should clamp to last item (index 4)
    }

    #[test]
    fn move_up_by_columns() {
        let items = vec![
            GridItem::new("A"), GridItem::new("B"), GridItem::new("C"),
            GridItem::new("D"), GridItem::new("E"), GridItem::new("F"),
        ];
        let mut grid = GridSelect::new("Test", items, 3);
        
        grid.set_selected(4); // Second item in second row
        grid.move_up();
        assert_eq!(grid.selected, 1); // Second item in first row (4 - 3)
    }

    #[test]
    fn move_up_clamps_to_zero() {
        let items = vec![
            GridItem::new("A"), GridItem::new("B"), GridItem::new("C"),
            GridItem::new("D"),
        ];
        let mut grid = GridSelect::new("Test", items, 3);
        
        grid.set_selected(1); // Second item in first row
        grid.move_up();
        assert_eq!(grid.selected, 0); // Should saturating_sub to 0
    }

    #[test]
    fn selected_item_access() {
        let items = vec![
            GridItem::new("First"),
            GridItem::new("Second"),
        ];
        let mut grid = GridSelect::new("Test", items, 2);
        
        assert_eq!(grid.selected_item().unwrap().label, "First");
        grid.set_selected(1);
        assert_eq!(grid.selected_item().unwrap().label, "Second");
        
        // Out of bounds
        grid.set_selected(10);
        assert_eq!(grid.selected_index(), 1); // Should clamp to last valid index
    }

    #[test]
    fn grid_item_with_color() {
        let item = GridItem::new("Red Item").with_color(Color::Red);
        assert_eq!(item.label, "Red Item");
        assert_eq!(item.color, Some(Color::Red));
    }

    #[test]
    fn grid_item_without_color() {
        let item = GridItem::new("Plain Item");
        assert_eq!(item.label, "Plain Item");
        assert_eq!(item.color, None);
    }

    #[test]
    fn set_selected_clamps() {
        let items = vec![GridItem::new("A"), GridItem::new("B")];
        let mut grid = GridSelect::new("Test", items, 2);
        
        grid.set_selected(5); // Way out of bounds
        assert_eq!(grid.selected, 1); // Should clamp to last valid index (1)
    }

    #[test]
    fn empty_grid_navigation() {
        let mut grid = GridSelect::new("Empty", vec![], 3);
        
        // All operations should be safe on empty grid
        grid.move_left();
        grid.move_right();
        grid.move_up();
        grid.move_down();
        assert_eq!(grid.selected, 0);
        assert!(grid.selected_item().is_none());
    }

    #[test]
    fn complex_navigation_scenario() {
        // 8 items in 3 columns:
        // 0 1 2
        // 3 4 5  
        // 6 7
        let items: Vec<GridItem> = (0..8).map(|i| GridItem::new(format!("Item {}", i))).collect();
        let mut grid = GridSelect::new("Test", items, 3);
        
        // Start at 0
        assert_eq!(grid.selected, 0);
        
        // Right to 1
        grid.move_right();
        assert_eq!(grid.selected, 1);
        
        // Down to 4
        grid.move_down();
        assert_eq!(grid.selected, 4);
        
        // Right to 5
        grid.move_right();
        assert_eq!(grid.selected, 5);
        
        // Down to 7 (clamped from 5+3=8 to last item 7)
        grid.move_down();
        assert_eq!(grid.selected, 7);
        
        // Up to 4
        grid.move_up();
        assert_eq!(grid.selected, 4);
        
        // Left to 3
        grid.move_left();
        assert_eq!(grid.selected, 3);
    }
}