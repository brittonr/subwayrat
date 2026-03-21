/// Opaque identifier for a window in the strip. Never reused after removal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub(crate) u64);

/// Primary axis for the scrollable strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Columns arranged left-to-right, windows stacked top-to-bottom within columns.
    Horizontal,
    /// Columns arranged top-to-bottom, windows stacked left-to-right within columns.
    Vertical,
}

/// Size constraint for a window or column along a single axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeConstraint {
    /// Exact cell count.
    Fixed(u16),
    /// Fraction of remaining space after fixed items are placed.
    /// Proportions are normalized across siblings.
    Proportion(f32),
    /// At least N cells, grows to fill remaining space.
    Min(u16),
    /// Clamped to [min, max] range after proportional allocation.
    MinMax(u16, u16),
}

impl Default for SizeConstraint {
    fn default() -> Self {
        SizeConstraint::Proportion(1.0)
    }
}

/// Configuration for the strip layout.
#[derive(Debug, Clone)]
pub struct StripConfig {
    /// Primary axis along which columns are arranged.
    pub axis: Axis,
    /// Gap in cells between columns along the primary axis.
    pub column_gap: u16,
    /// Gap in cells between windows within a column along the cross axis.
    pub window_gap: u16,
}

impl Default for StripConfig {
    fn default() -> Self {
        StripConfig {
            axis: Axis::Horizontal,
            column_gap: 1,
            window_gap: 0,
        }
    }
}
