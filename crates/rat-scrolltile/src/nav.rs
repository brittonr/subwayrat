use crate::layout::{compute_layout, LayoutResult};
use crate::strip::Strip;
use crate::types::{Axis, WindowId};

/// Move focus to the next column in the given direction along the primary axis.
/// Uses cross-axis affinity to pick the closest window in the target column.
///
/// `viewport_width` and `viewport_height` are needed to compute layout for affinity matching.
pub fn focus_primary(strip: &mut Strip, forward: bool, viewport_width: u16, viewport_height: u16) {
    let Some(focus_id) = strip.focus else {
        // No focus — try to focus the first window.
        if let Some(col) = strip.columns.first() {
            if let Some(w) = col.windows.first() {
                strip.focus = Some(w.id);
            }
        }
        return;
    };

    let Some((col_idx, _win_idx)) = strip.find_window(focus_id) else {
        return;
    };

    let target_col = if forward {
        if col_idx + 1 >= strip.columns.len() {
            return; // Already at last column.
        }
        col_idx + 1
    } else {
        if col_idx == 0 {
            return; // Already at first column.
        }
        col_idx - 1
    };

    if strip.columns[target_col].windows.is_empty() {
        return;
    }

    // Compute layout to get positions for affinity matching.
    let result = compute_layout(strip, viewport_width, viewport_height);

    // Get or establish cross-axis affinity.
    let affinity = strip.cross_affinity.unwrap_or_else(|| {
        // Set affinity from current focused window's cross-axis center.
        cross_center(&result, focus_id, strip.config.axis)
    });

    // Store affinity for subsequent primary-axis moves.
    strip.cross_affinity = Some(affinity);

    // Find the window in the target column closest to the affinity.
    // On ties, prefer the window whose range contains the affinity point.
    let best = strip.columns[target_col]
        .windows
        .iter()
        .min_by(|a, b| {
            let a_center = cross_center(&result, a.id, strip.config.axis);
            let b_center = cross_center(&result, b.id, strip.config.axis);
            let a_dist = (a_center as i32 - affinity as i32).unsigned_abs();
            let b_dist = (b_center as i32 - affinity as i32).unsigned_abs();
            a_dist.cmp(&b_dist).then_with(|| {
                // Tiebreak: prefer the window that contains the affinity row.
                let a_contains = cross_contains(&result, a.id, affinity, strip.config.axis);
                let b_contains = cross_contains(&result, b.id, affinity, strip.config.axis);
                b_contains.cmp(&a_contains) // true > false, so reverse
            })
        })
        .map(|w| w.id);

    if let Some(id) = best {
        strip.focus = Some(id);
    }
}

/// Move focus to the adjacent window within the same column.
/// `forward` = true means down (horizontal mode) or right (vertical mode).
/// Resets cross-axis affinity.
pub fn focus_cross(strip: &mut Strip, forward: bool) {
    let Some(focus_id) = strip.focus else {
        return;
    };
    let Some((col_idx, win_idx)) = strip.find_window(focus_id) else {
        return;
    };

    let col = &strip.columns[col_idx];
    let target = if forward {
        if win_idx + 1 >= col.windows.len() {
            return; // At bottom/right edge.
        }
        win_idx + 1
    } else {
        if win_idx == 0 {
            return; // At top/left edge.
        }
        win_idx - 1
    };

    strip.focus = Some(col.windows[target].id);
    strip.cross_affinity = None; // Reset affinity on cross-axis move.
}

/// Focus the first window in the first column.
pub fn focus_first(strip: &mut Strip) {
    strip.cross_affinity = None;
    for col in &strip.columns {
        if let Some(w) = col.windows.first() {
            strip.focus = Some(w.id);
            return;
        }
    }
    strip.focus = None;
}

/// Focus the last window in the last column.
pub fn focus_last(strip: &mut Strip) {
    strip.cross_affinity = None;
    for col in strip.columns.iter().rev() {
        if let Some(w) = col.windows.last() {
            strip.focus = Some(w.id);
            return;
        }
    }
    strip.focus = None;
}

/// Convenience: move focus left (primary axis, backward).
pub fn focus_left(strip: &mut Strip, viewport_width: u16, viewport_height: u16) {
    focus_primary(strip, false, viewport_width, viewport_height);
}

/// Convenience: move focus right (primary axis, forward).
pub fn focus_right(strip: &mut Strip, viewport_width: u16, viewport_height: u16) {
    focus_primary(strip, true, viewport_width, viewport_height);
}

/// Convenience: move focus up (cross axis, backward).
pub fn focus_up(strip: &mut Strip) {
    focus_cross(strip, false);
}

/// Convenience: move focus down (cross axis, forward).
pub fn focus_down(strip: &mut Strip) {
    focus_cross(strip, true);
}

/// Get the cross-axis center of a window from the layout result.
fn cross_center(result: &LayoutResult, id: WindowId, axis: Axis) -> u16 {
    result
        .window_rects
        .get(&id)
        .map(|r| match axis {
            Axis::Horizontal => r.y + r.height / 2,
            Axis::Vertical => r.x + r.width / 2,
        })
        .unwrap_or(0)
}

/// Check if a window's cross-axis range contains the given position.
fn cross_contains(result: &LayoutResult, id: WindowId, pos: u16, axis: Axis) -> bool {
    result
        .window_rects
        .get(&id)
        .map(|r| {
            let (start, size) = match axis {
                Axis::Horizontal => (r.y, r.height),
                Axis::Vertical => (r.x, r.width),
            };
            pos >= start && pos < start + size
        })
        .unwrap_or(false)
}
