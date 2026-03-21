use crate::types::{SizeConstraint, StripConfig, WindowId};

/// A window in the strip with size constraints.
#[derive(Debug, Clone)]
pub struct Window {
    pub(crate) id: WindowId,
    /// Size constraint along the primary axis (width in horizontal mode).
    pub width_constraint: SizeConstraint,
    /// Size constraint along the cross axis (height in horizontal mode).
    pub height_constraint: SizeConstraint,
}

/// A column containing stacked windows.
#[derive(Debug, Clone)]
pub struct Column {
    pub(crate) windows: Vec<Window>,
    /// Width constraint for this column along the primary axis.
    pub width_constraint: SizeConstraint,
}

impl Column {
    /// Create a new empty column.
    pub fn new(width_constraint: SizeConstraint) -> Self {
        Column {
            windows: Vec::new(),
            width_constraint,
        }
    }

    /// Number of windows in this column.
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Iterate over windows.
    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    /// Find a window's index by ID.
    pub fn find_window(&self, id: WindowId) -> Option<usize> {
        self.windows.iter().position(|w| w.id == id)
    }
}

/// Scroll mode for the viewport.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ScrollMode {
    /// Viewport follows the focused window.
    FocusTracking,
    /// Viewport locked to a manual offset.
    Manual(u16),
}

/// The top-level scrollable strip container.
#[derive(Debug, Clone)]
pub struct Strip {
    pub(crate) columns: Vec<Column>,
    pub(crate) config: StripConfig,
    pub(crate) focus: Option<WindowId>,
    pub(crate) cross_affinity: Option<u16>,
    pub(crate) scroll_mode: ScrollMode,
    pub(crate) next_id: u64,
}

impl Strip {
    /// Create a new empty strip with the given configuration.
    pub fn new(config: StripConfig) -> Self {
        Strip {
            columns: Vec::new(),
            config,
            focus: None,
            cross_affinity: None,
            scroll_mode: ScrollMode::FocusTracking,
            next_id: 1,
        }
    }

    /// Get the strip configuration.
    pub fn config(&self) -> &StripConfig {
        &self.config
    }

    /// Set the strip configuration.
    pub fn set_config(&mut self, config: StripConfig) {
        self.config = config;
    }

    /// Number of columns.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Access columns.
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    fn alloc_id(&mut self) -> WindowId {
        let id = WindowId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Find which column and stack position a window is in.
    pub fn find_window(&self, id: WindowId) -> Option<(usize, usize)> {
        for (col_idx, col) in self.columns.iter().enumerate() {
            if let Some(win_idx) = col.find_window(id) {
                return Some((col_idx, win_idx));
            }
        }
        None
    }

    // -- Window Management --

    /// Insert a window into a specific column at a stack position.
    /// Creates intervening empty columns if `column_idx` is beyond the current count.
    /// Clamps `stack_pos` to append if beyond the column's window count.
    /// Returns the new window's ID.
    pub fn insert_window(
        &mut self,
        column_idx: usize,
        stack_pos: usize,
        width_constraint: SizeConstraint,
        height_constraint: SizeConstraint,
    ) -> WindowId {
        let id = self.alloc_id();
        // Create intervening columns if needed.
        while self.columns.len() <= column_idx {
            self.columns.push(Column::new(SizeConstraint::default()));
        }
        let col = &mut self.columns[column_idx];
        let pos = stack_pos.min(col.windows.len());
        col.windows.insert(
            pos,
            Window {
                id,
                width_constraint,
                height_constraint,
            },
        );
        id
    }

    /// Remove a window by ID. Removes empty columns left behind.
    /// Returns true if the window was found and removed.
    pub fn remove_window(&mut self, id: WindowId) -> bool {
        let Some((col_idx, win_idx)) = self.find_window(id) else {
            return false;
        };

        // If this window is focused, pick a fallback before removing.
        if self.focus == Some(id) {
            self.focus = self.focus_fallback(col_idx, win_idx);
            self.cross_affinity = None;
        }

        self.columns[col_idx].windows.remove(win_idx);

        // Remove empty column.
        if self.columns[col_idx].windows.is_empty() {
            self.columns.remove(col_idx);
        }

        true
    }

    /// Move a window to a different column and stack position. Atomic remove+insert.
    /// Returns false if the window ID was not found.
    pub fn move_window(
        &mut self,
        id: WindowId,
        target_column: usize,
        target_stack_pos: usize,
    ) -> bool {
        let Some((src_col, src_idx)) = self.find_window(id) else {
            return false;
        };

        let window = self.columns[src_col].windows.remove(src_idx);

        // Clean up empty source column.
        let target_adjusted = if self.columns[src_col].windows.is_empty() {
            self.columns.remove(src_col);
            // Adjust target index if source was before target.
            if src_col < target_column {
                target_column.saturating_sub(1)
            } else {
                target_column
            }
        } else {
            target_column
        };

        // Ensure target column exists.
        while self.columns.len() <= target_adjusted {
            self.columns.push(Column::new(SizeConstraint::default()));
        }

        let col = &mut self.columns[target_adjusted];
        let pos = target_stack_pos.min(col.windows.len());
        col.windows.insert(pos, window);

        true
    }

    /// Change a window's size constraints.
    pub fn resize_window(
        &mut self,
        id: WindowId,
        width_constraint: SizeConstraint,
        height_constraint: SizeConstraint,
    ) -> bool {
        let Some((col_idx, win_idx)) = self.find_window(id) else {
            return false;
        };
        let w = &mut self.columns[col_idx].windows[win_idx];
        w.width_constraint = width_constraint;
        w.height_constraint = height_constraint;
        true
    }

    /// Change a column's width constraint.
    pub fn resize_column(&mut self, column_idx: usize, constraint: SizeConstraint) -> bool {
        if column_idx >= self.columns.len() {
            return false;
        }
        self.columns[column_idx].width_constraint = constraint;
        true
    }

    /// Insert a new column at the given index, shifting existing columns right.
    /// Optionally inserts an initial window.
    pub fn insert_column(
        &mut self,
        column_idx: usize,
        width_constraint: SizeConstraint,
        initial_window: Option<(SizeConstraint, SizeConstraint)>,
    ) -> Option<WindowId> {
        let idx = column_idx.min(self.columns.len());
        let mut col = Column::new(width_constraint);
        let win_id = initial_window.map(|(wc, hc)| {
            let id = self.alloc_id();
            col.windows.push(Window {
                id,
                width_constraint: wc,
                height_constraint: hc,
            });
            id
        });
        self.columns.insert(idx, col);
        win_id
    }

    // -- Focus --

    /// Set focus to a specific window.
    pub fn focus_set(&mut self, id: WindowId) {
        if self.find_window(id).is_some() {
            self.focus = Some(id);
            self.cross_affinity = None;
            self.scroll_mode = ScrollMode::FocusTracking;
        }
    }

    /// Clear focus.
    pub fn focus_clear(&mut self) {
        self.focus = None;
        self.cross_affinity = None;
    }

    /// Get the currently focused window.
    pub fn focused(&self) -> Option<WindowId> {
        self.focus
    }

    /// Set a manual scroll offset, disabling focus tracking.
    pub fn set_scroll_offset(&mut self, offset: u16) {
        self.scroll_mode = ScrollMode::Manual(offset);
    }

    /// Re-enable focus-driven scroll tracking.
    pub fn enable_focus_tracking(&mut self) {
        self.scroll_mode = ScrollMode::FocusTracking;
    }

    /// Find a fallback focus target when a window is removed.
    /// Prefers same column (next window, or previous), then adjacent columns.
    fn focus_fallback(&self, col_idx: usize, win_idx: usize) -> Option<WindowId> {
        let col = &self.columns[col_idx];
        // Try next window in same column.
        if win_idx + 1 < col.windows.len() {
            return Some(col.windows[win_idx + 1].id);
        }
        // Try previous window in same column.
        if win_idx > 0 {
            return Some(col.windows[win_idx - 1].id);
        }
        // Try adjacent columns (right first, then left).
        if col_idx + 1 < self.columns.len() && !self.columns[col_idx + 1].windows.is_empty() {
            return Some(self.columns[col_idx + 1].windows[0].id);
        }
        if col_idx > 0 && !self.columns[col_idx - 1].windows.is_empty() {
            return Some(self.columns[col_idx - 1].windows[0].id);
        }
        None
    }
}
