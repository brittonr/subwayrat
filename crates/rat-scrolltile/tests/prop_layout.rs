//! Property-based tests for rat-scrolltile layout invariants.
//!
//! Generates random strip configurations and verifies that compute_layout
//! always produces results satisfying the core tiling invariants:
//! non-overlapping rects, constraint satisfaction, viewport containment,
//! scroll bounds, and focus visibility.

use std::collections::HashSet;

use proptest::prelude::*;
use ratatui::layout::Rect;

use rat_scrolltile::{compute_layout, nav, SizeConstraint, Strip, StripConfig, WindowId, Axis};

// --- Generators ---

fn arb_axis() -> impl Strategy<Value = Axis> {
    prop_oneof![Just(Axis::Horizontal), Just(Axis::Vertical)]
}

fn arb_size_constraint() -> impl Strategy<Value = SizeConstraint> {
    prop_oneof![
        (1u16..=200).prop_map(SizeConstraint::Fixed),
        (1u16..=200).prop_map(|n| SizeConstraint::Min(n)),
        prop::num::f32::POSITIVE.prop_map(|f| SizeConstraint::Proportion(f.clamp(0.1, 10.0))),
        (1u16..=100, 1u16..=200).prop_map(|(lo, spread)| {
            SizeConstraint::MinMax(lo, lo.saturating_add(spread))
        }),
    ]
}

fn arb_strip_config() -> impl Strategy<Value = StripConfig> {
    (arb_axis(), 0u16..=3, 0u16..=3).prop_map(|(axis, column_gap, window_gap)| StripConfig {
        axis,
        column_gap,
        window_gap,
    })
}

/// A strip together with all the WindowIds it contains (tracked at insertion time).
#[derive(Debug, Clone)]
struct StripWithIds {
    strip: Strip,
    ids: Vec<WindowId>,           // all window IDs in insertion order
    ids_by_col: Vec<Vec<WindowId>>, // IDs grouped by column
}

/// Build a strip with random columns and windows, tracking IDs.
fn arb_strip() -> impl Strategy<Value = StripWithIds> {
    (
        arb_strip_config(),
        1usize..=6,
        1usize..=4,
    )
        .prop_flat_map(|(config, num_cols, max_wins)| {
            let col_constraints = proptest::collection::vec(arb_size_constraint(), num_cols);
            let win_counts = proptest::collection::vec(1usize..=max_wins, num_cols);
            let win_constraints = proptest::collection::vec(
                proptest::collection::vec(arb_size_constraint(), 1..=max_wins),
                num_cols,
            );
            (Just(config), col_constraints, win_counts, win_constraints)
        })
        .prop_map(|(config, col_constraints, win_counts, win_constraints)| {
            let mut strip = Strip::new(config);
            let mut ids = Vec::new();
            let mut ids_by_col = Vec::new();
            for (col_idx, (&count, (col_c, win_cs))) in win_counts
                .iter()
                .zip(col_constraints.iter().zip(win_constraints.iter()))
                .enumerate()
            {
                let mut col_ids = Vec::new();
                for win_idx in 0..count {
                    let wc = win_cs.get(win_idx).copied().unwrap_or_default();
                    let id = strip.insert_window(col_idx, win_idx, *col_c, wc);
                    ids.push(id);
                    col_ids.push(id);
                }
                strip.resize_column(col_idx, *col_c);
                ids_by_col.push(col_ids);
            }
            StripWithIds { strip, ids, ids_by_col }
        })
}

fn arb_viewport() -> impl Strategy<Value = (u16, u16)> {
    (10u16..=300, 10u16..=100)
}

// --- Helpers ---

fn rects_overlap(a: &Rect, b: &Rect) -> bool {
    if a.width == 0 || a.height == 0 || b.width == 0 || b.height == 0 {
        return false;
    }
    a.x < b.x + b.width && b.x < a.x + a.width
        && a.y < b.y + b.height && b.y < a.y + a.height
}

// --- Properties ---

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// All window rects are pairwise non-overlapping.
    #[test]
    fn prop_no_overlap(sw in arb_strip(), (vw, vh) in arb_viewport()) {
        let result = compute_layout(&sw.strip, vw, vh);
        let rects: Vec<_> = result.window_rects.values().collect();
        for (i, a) in rects.iter().enumerate() {
            for b in rects.iter().skip(i + 1) {
                prop_assert!(
                    !rects_overlap(a, b),
                    "Rects overlap: {:?} and {:?}",
                    a, b
                );
            }
        }
    }

    /// Visible windows are clipped to the viewport on the primary (scroll) axis.
    /// The cross axis is NOT clipped — windows can exceed the viewport height
    /// (in horizontal mode) or width (in vertical mode).
    #[test]
    fn prop_visible_clipped_on_primary_axis(sw in arb_strip(), (vw, vh) in arb_viewport()) {
        let result = compute_layout(&sw.strip, vw, vh);
        let axis = sw.strip.config().axis;
        for vwin in &result.visible {
            let r = &vwin.rect;
            let (pos, size, vp_size) = match axis {
                Axis::Horizontal => (r.x, r.width, vw),
                Axis::Vertical => (r.y, r.height, vh),
            };
            let end = pos as u32 + size as u32;
            prop_assert!(
                end <= vp_size as u32,
                "Visible window {:?} exceeds viewport on primary axis: end={} > vp={}",
                vwin.id, end, vp_size
            );
        }
    }

    /// Fixed constraints produce exact widths.
    #[test]
    fn prop_fixed_constraints_exact(
        (vw, vh) in arb_viewport(),
        fixed_width in 1u16..=50,
    ) {
        let mut strip = Strip::new(StripConfig {
            column_gap: 0,
            window_gap: 0,
            ..Default::default()
        });
        let id = strip.insert_window(0, 0, SizeConstraint::Fixed(fixed_width), SizeConstraint::default());
        strip.resize_column(0, SizeConstraint::Fixed(fixed_width));

        let result = compute_layout(&strip, vw, vh);
        let rect = result.window_rects[&id];
        prop_assert_eq!(
            rect.width, fixed_width,
            "Fixed({}) produced width {}",
            fixed_width, rect.width
        );
    }

    /// MinMax constraints produce sizes within bounds.
    #[test]
    fn prop_minmax_constraints_bounded(
        (vw, vh) in arb_viewport(),
        lo in 5u16..=30,
        spread in 5u16..=50,
    ) {
        let hi = lo.saturating_add(spread);
        let mut strip = Strip::new(StripConfig {
            column_gap: 0,
            window_gap: 0,
            ..Default::default()
        });
        let id = strip.insert_window(0, 0, SizeConstraint::MinMax(lo, hi), SizeConstraint::default());
        strip.resize_column(0, SizeConstraint::MinMax(lo, hi));

        let result = compute_layout(&strip, vw, vh);
        let rect = result.window_rects[&id];
        prop_assert!(
            rect.width >= lo && rect.width <= hi,
            "MinMax({}, {}) produced width {}",
            lo, hi, rect.width
        );
    }

    /// Scroll offset doesn't push viewport past the strip extent.
    #[test]
    fn prop_scroll_bounded(sw in arb_strip(), (vw, vh) in arb_viewport()) {
        let result = compute_layout(&sw.strip, vw, vh);
        if result.strip_extent > vw {
            prop_assert!(
                result.scroll_offset <= result.strip_extent.saturating_sub(vw),
                "Scroll {} exceeds max {} (extent={}, vp={})",
                result.scroll_offset,
                result.strip_extent.saturating_sub(vw),
                result.strip_extent,
                vw
            );
        }
    }

    /// Focus tracking: when focus tracking is active and the focused window
    /// has non-zero size on the primary axis, it appears in the visible set.
    /// (A window squeezed to zero width by oversized fixed siblings can't
    /// be "visible" even when focused — this is a valid degenerate case.)
    #[test]
    fn prop_focus_visible_when_nonzero(sw in arb_strip(), (vw, vh) in arb_viewport()) {
        if sw.ids.is_empty() {
            return Ok(());
        }
        let mut strip = sw.strip.clone();
        let id = sw.ids[0];
        strip.focus_set(id);
        strip.enable_focus_tracking();

        let result = compute_layout(&strip, vw, vh);

        // Only check if the focused window has non-zero primary-axis size.
        if let Some(rect) = result.window_rects.get(&id) {
            let primary_size = match sw.strip.config().axis {
                Axis::Horizontal => rect.width,
                Axis::Vertical => rect.height,
            };
            if primary_size > 0 {
                let focused_visible = result.visible.iter().any(|v| v.id == id);
                prop_assert!(
                    focused_visible,
                    "Focused window {:?} (size={}) not visible (scroll={}, extent={}, vp={})",
                    id, primary_size, result.scroll_offset, result.strip_extent, vw
                );
            }
        }
    }

    /// Window IDs in layout result match the IDs in the strip.
    #[test]
    fn prop_ids_consistent(sw in arb_strip(), (vw, vh) in arb_viewport()) {
        let result = compute_layout(&sw.strip, vw, vh);

        let strip_ids: HashSet<_> = sw.ids.iter().copied().collect();
        let layout_ids: HashSet<_> = result.window_rects.keys().copied().collect();

        prop_assert_eq!(
            strip_ids, layout_ids,
            "Strip IDs and layout IDs differ"
        );
    }

    /// Every window gets a rect (layout is total over windows).
    #[test]
    fn prop_layout_total(sw in arb_strip(), (vw, vh) in arb_viewport()) {
        let result = compute_layout(&sw.strip, vw, vh);
        prop_assert_eq!(
            result.window_rects.len(), sw.ids.len(),
            "Layout produced {} rects for {} windows",
            result.window_rects.len(), sw.ids.len()
        );
    }

    /// Navigation: focus_right then focus_left returns to the same window
    /// (when there are at least 2 columns).
    #[test]
    fn prop_nav_left_right_roundtrip(sw in arb_strip(), (vw, vh) in arb_viewport()) {
        if sw.strip.column_count() < 2 || sw.ids_by_col[0].is_empty() {
            return Ok(());
        }
        let mut strip = sw.strip.clone();
        let original = sw.ids_by_col[0][0];
        strip.focus_set(original);

        nav::focus_right(&mut strip, vw, vh);
        nav::focus_left(&mut strip, vw, vh);

        prop_assert_eq!(
            strip.focused(), Some(original),
            "Left-right roundtrip didn't return to original"
        );
    }

    /// Navigation: focus_down then focus_up returns to the same window.
    #[test]
    fn prop_nav_up_down_roundtrip(sw in arb_strip()) {
        // Need a column with >= 2 windows.
        let stacked_col = sw.ids_by_col.iter().find(|col| col.len() >= 2);
        let Some(col_ids) = stacked_col else {
            return Ok(());
        };
        let mut strip = sw.strip.clone();
        let original = col_ids[0];
        strip.focus_set(original);

        nav::focus_down(&mut strip);
        nav::focus_up(&mut strip);

        prop_assert_eq!(
            strip.focused(), Some(original),
            "Up-down roundtrip didn't return to original"
        );
    }

    /// Removing a window preserves valid focus (either None or existing window).
    #[test]
    fn prop_remove_preserves_focus(sw in arb_strip()) {
        if sw.ids.is_empty() {
            return Ok(());
        }
        let mut strip = sw.strip.clone();

        // Focus the first window, then remove it.
        strip.focus_set(sw.ids[0]);
        strip.remove_window(sw.ids[0]);

        match strip.focused() {
            None => {
                // Valid — strip might be empty now.
            }
            Some(id) => {
                prop_assert!(
                    strip.find_window(id).is_some(),
                    "Focus {:?} points to nonexistent window after removal",
                    id
                );
            }
        }
    }

    /// Windows within the same column don't overlap on the cross axis.
    #[test]
    fn prop_column_windows_no_cross_overlap(sw in arb_strip(), (vw, vh) in arb_viewport()) {
        let result = compute_layout(&sw.strip, vw, vh);
        let axis = sw.strip.config().axis;

        for col_ids in &sw.ids_by_col {
            let col_rects: Vec<_> = col_ids.iter()
                .filter_map(|id| result.window_rects.get(id))
                .collect();

            for (i, a) in col_rects.iter().enumerate() {
                for b in col_rects.iter().skip(i + 1) {
                    let (a_start, a_size, b_start, b_size) = match axis {
                        Axis::Horizontal => (a.y, a.height, b.y, b.height),
                        Axis::Vertical => (a.x, a.width, b.x, b.width),
                    };
                    if a_size > 0 && b_size > 0 {
                        let a_end = a_start as u32 + a_size as u32;
                        let b_end = b_start as u32 + b_size as u32;
                        prop_assert!(
                            a_end <= b_start as u32 || b_end <= a_start as u32,
                            "Windows in same column overlap on cross axis: {:?} and {:?}",
                            a, b
                        );
                    }
                }
            }
        }
    }
}
