use std::collections::HashMap;

use ratatui::layout::Rect;

use crate::strip::Strip;
use crate::types::{Axis, SizeConstraint, WindowId};

/// A visible window with its viewport-local rect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleWindow {
    /// Window identifier.
    pub id: WindowId,
    /// Rect in viewport-local coordinates (0,0 = top-left of viewport).
    pub rect: Rect,
    /// Whether the window is fully visible (not clipped).
    pub fully_visible: bool,
}

/// Result of computing the strip layout.
#[derive(Debug, Clone)]
pub struct LayoutResult {
    /// Strip-space rect for every window, keyed by ID.
    pub window_rects: HashMap<WindowId, Rect>,
    /// Total extent of the strip along the primary axis.
    pub strip_extent: u16,
    /// Computed scroll offset along the primary axis.
    pub scroll_offset: u16,
    /// Windows visible in the viewport with viewport-local rects.
    pub visible: Vec<VisibleWindow>,
}

/// Resolve a list of size constraints into concrete cell counts.
///
/// `total` is the available space. `gap` is the space between items.
/// Returns a vec of resolved sizes, one per constraint.
pub(crate) fn resolve_constraints(constraints: &[SizeConstraint], total: u16, gap: u16) -> Vec<u16> {
    let n = constraints.len();
    if n == 0 {
        return Vec::new();
    }

    let total_gaps = gap.saturating_mul(n.saturating_sub(1) as u16);
    let usable = total.saturating_sub(total_gaps);

    // First pass: allocate fixed and min sizes, collect proportional items.
    let mut sizes = vec![0u16; n];
    let mut fixed_total: u16 = 0;
    let mut prop_total: f32 = 0.0;
    let mut prop_indices = Vec::new();

    for (i, c) in constraints.iter().enumerate() {
        match *c {
            SizeConstraint::Fixed(v) => {
                sizes[i] = v;
                fixed_total = fixed_total.saturating_add(v);
            }
            SizeConstraint::Proportion(p) => {
                prop_indices.push(i);
                prop_total += p;
            }
            SizeConstraint::Min(min) => {
                // Treated as proportional with a floor.
                sizes[i] = min;
                prop_indices.push(i);
                prop_total += 1.0;
            }
            SizeConstraint::MinMax(min, _max) => {
                sizes[i] = min;
                prop_indices.push(i);
                prop_total += 1.0;
            }
        }
    }

    // Second pass: distribute remaining space among proportional items.
    let remaining = usable.saturating_sub(fixed_total);
    if prop_total > 0.0 && !prop_indices.is_empty() {
        // Subtract already-allocated min sizes from remaining.
        let min_allocated: u16 = prop_indices.iter().map(|&i| sizes[i]).sum();
        let distributable = remaining.saturating_sub(min_allocated);

        for &i in &prop_indices {
            let fraction = match constraints[i] {
                SizeConstraint::Proportion(p) => p / prop_total,
                SizeConstraint::Min(_) | SizeConstraint::MinMax(_, _) => 1.0 / prop_total,
                _ => 0.0,
            };
            let alloc = (distributable as f32 * fraction) as u16;
            let base = sizes[i]; // existing min

            let resolved = match constraints[i] {
                SizeConstraint::MinMax(min, max) => (base + alloc).clamp(min, max),
                SizeConstraint::Min(min) => (base + alloc).max(min),
                _ => base + alloc,
            };
            sizes[i] = resolved;
        }
    }

    sizes
}

/// Compute the full layout for a strip given a viewport size.
pub fn compute_layout(strip: &Strip, viewport_width: u16, viewport_height: u16) -> LayoutResult {
    let config = &strip.config;
    let (vp_primary, vp_cross) = match config.axis {
        Axis::Horizontal => (viewport_width, viewport_height),
        Axis::Vertical => (viewport_height, viewport_width),
    };

    if strip.columns.is_empty() {
        return LayoutResult {
            window_rects: HashMap::new(),
            strip_extent: 0,
            scroll_offset: 0,
            visible: Vec::new(),
        };
    }

    // Resolve column widths (along primary axis).
    // For the strip, columns can exceed the viewport — we use the viewport as the
    // "available space" hint for proportional columns, but fixed columns may push
    // the total beyond it.
    let col_constraints: Vec<SizeConstraint> =
        strip.columns.iter().map(|c| c.width_constraint).collect();
    let col_widths = resolve_constraints(&col_constraints, vp_primary, config.column_gap);

    // Assign primary-axis positions to columns.
    let mut col_positions = Vec::with_capacity(strip.columns.len());
    let mut pos: u16 = 0;
    for (i, &w) in col_widths.iter().enumerate() {
        col_positions.push(pos);
        pos = pos.saturating_add(w);
        if i + 1 < col_widths.len() {
            pos = pos.saturating_add(config.column_gap);
        }
    }
    let strip_extent = pos;

    // Resolve windows within each column.
    let mut window_rects = HashMap::new();
    for (col_idx, col) in strip.columns.iter().enumerate() {
        let col_primary_pos = col_positions[col_idx];
        let col_width = col_widths[col_idx];

        let win_constraints: Vec<SizeConstraint> =
            col.windows.iter().map(|w| w.height_constraint).collect();
        let win_heights = resolve_constraints(&win_constraints, vp_cross, config.window_gap);

        let mut cross_pos: u16 = 0;
        for (win_idx, window) in col.windows.iter().enumerate() {
            let win_height = win_heights.get(win_idx).copied().unwrap_or(0);

            let rect = match config.axis {
                Axis::Horizontal => Rect {
                    x: col_primary_pos,
                    y: cross_pos,
                    width: col_width,
                    height: win_height,
                },
                Axis::Vertical => Rect {
                    x: cross_pos,
                    y: col_primary_pos,
                    width: win_height,
                    height: col_width,
                },
            };
            window_rects.insert(window.id, rect);

            cross_pos = cross_pos.saturating_add(win_height);
            if win_idx + 1 < col.windows.len() {
                cross_pos = cross_pos.saturating_add(config.window_gap);
            }
        }
    }

    // Compute scroll offset.
    let scroll_offset = compute_scroll_offset(strip, &window_rects, vp_primary, strip_extent);

    // Compute visible windows.
    let visible = compute_visible(
        &window_rects,
        scroll_offset,
        viewport_width,
        viewport_height,
        config.axis,
    );

    LayoutResult {
        window_rects,
        strip_extent,
        scroll_offset,
        visible,
    }
}

fn compute_scroll_offset(
    strip: &Strip,
    rects: &HashMap<WindowId, Rect>,
    vp_primary: u16,
    strip_extent: u16,
) -> u16 {
    use crate::strip::ScrollMode;

    match strip.scroll_mode {
        ScrollMode::Manual(offset) => offset.min(strip_extent.saturating_sub(vp_primary)),
        ScrollMode::FocusTracking => {
            let Some(focus_id) = strip.focus else {
                return 0;
            };
            let Some(rect) = rects.get(&focus_id) else {
                return 0;
            };

            let (win_start, win_size) = match strip.config.axis {
                Axis::Horizontal => (rect.x, rect.width),
                Axis::Vertical => (rect.y, rect.height),
            };
            // Center the window in the viewport.
            let win_center = win_start.saturating_add(win_size / 2);
            let ideal = win_center.saturating_sub(vp_primary / 2);
            let max_offset = strip_extent.saturating_sub(vp_primary);

            // But don't scroll if the window is already fully visible at offset 0
            // We need to check against the *computed* offset, so just clamp the centering.
            ideal.min(max_offset)
        }
    }
}

fn compute_visible(
    rects: &HashMap<WindowId, Rect>,
    scroll_offset: u16,
    viewport_width: u16,
    viewport_height: u16,
    axis: Axis,
) -> Vec<VisibleWindow> {
    let vp_start = scroll_offset;
    let (vp_primary_size, _vp_cross_size) = match axis {
        Axis::Horizontal => (viewport_width, viewport_height),
        Axis::Vertical => (viewport_height, viewport_width),
    };
    let vp_end = vp_start.saturating_add(vp_primary_size);

    let mut visible = Vec::new();

    for (&id, &rect) in rects {
        let (win_start, win_size, win_cross_start, win_cross_size) = match axis {
            Axis::Horizontal => (rect.x, rect.width, rect.y, rect.height),
            Axis::Vertical => (rect.y, rect.height, rect.x, rect.width),
        };
        let win_end = win_start.saturating_add(win_size);

        // Check overlap on primary axis.
        if win_end <= vp_start || win_start >= vp_end {
            continue;
        }

        // Compute clipped viewport-local rect.
        let clip_start = win_start.max(vp_start);
        let clip_end = win_end.min(vp_end);
        let local_primary_pos = clip_start - vp_start;
        let local_primary_size = clip_end - clip_start;

        let fully_visible = win_start >= vp_start && win_end <= vp_end;

        let vp_rect = match axis {
            Axis::Horizontal => Rect {
                x: local_primary_pos,
                y: win_cross_start,
                width: local_primary_size,
                height: win_cross_size,
            },
            Axis::Vertical => Rect {
                x: win_cross_start,
                y: local_primary_pos,
                width: win_cross_size,
                height: local_primary_size,
            },
        };

        visible.push(VisibleWindow {
            id,
            rect: vp_rect,
            fully_visible,
        });
    }

    // Sort by primary axis position for deterministic ordering.
    visible.sort_by_key(|v| match axis {
        Axis::Horizontal => v.rect.x,
        Axis::Vertical => v.rect.y,
    });

    visible
}
