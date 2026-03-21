use crate::layout::{compute_layout, resolve_constraints};
use crate::nav;
use crate::strip::Strip;
use crate::types::{Axis, SizeConstraint, StripConfig};

// ── Constraint resolution ──────────────────────────────────────────

#[test]
fn resolve_all_fixed() {
    let sizes = resolve_constraints(
        &[SizeConstraint::Fixed(10), SizeConstraint::Fixed(20)],
        100,
        0,
    );
    assert_eq!(sizes, vec![10, 20]);
}

#[test]
fn resolve_all_proportional() {
    let sizes = resolve_constraints(
        &[
            SizeConstraint::Proportion(1.0),
            SizeConstraint::Proportion(2.0),
        ],
        90,
        0,
    );
    assert_eq!(sizes, vec![30, 60]);
}

#[test]
fn resolve_mixed_fixed_proportional() {
    let sizes = resolve_constraints(
        &[
            SizeConstraint::Fixed(20),
            SizeConstraint::Proportion(1.0),
        ],
        81,
        1, // 1 gap
    );
    // usable = 81 - 1 = 80, fixed = 20, remaining = 60
    assert_eq!(sizes, vec![20, 60]);
}

#[test]
fn resolve_minmax_clamps() {
    let sizes = resolve_constraints(
        &[SizeConstraint::MinMax(10, 30)],
        100,
        0,
    );
    // Proportional would give 100, clamped to 30.
    assert_eq!(sizes, vec![30]);
}

#[test]
fn resolve_min_grows() {
    let sizes = resolve_constraints(&[SizeConstraint::Min(10)], 40, 0);
    assert_eq!(sizes, vec![40]);
}

#[test]
fn resolve_empty() {
    let sizes = resolve_constraints(&[], 100, 0);
    assert!(sizes.is_empty());
}

#[test]
fn resolve_with_gaps() {
    let sizes = resolve_constraints(
        &[
            SizeConstraint::Fixed(10),
            SizeConstraint::Fixed(10),
            SizeConstraint::Fixed(10),
        ],
        100,
        5,
    );
    // Gaps don't affect fixed sizes, they just consume space.
    assert_eq!(sizes, vec![10, 10, 10]);
}

// ── Strip layout (horizontal) ──────────────────────────────────────

#[test]
fn layout_empty_strip() {
    let strip = Strip::new(StripConfig::default());
    let result = compute_layout(&strip, 80, 24);
    assert_eq!(result.strip_extent, 0);
    assert!(result.window_rects.is_empty());
    assert!(result.visible.is_empty());
}

#[test]
fn layout_single_column_single_window() {
    let mut strip = Strip::new(StripConfig {
        column_gap: 0,
        window_gap: 0,
        ..Default::default()
    });
    let id = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.focus_set(id);

    let result = compute_layout(&strip, 80, 24);
    let rect = result.window_rects[&id];
    assert_eq!(rect.x, 0);
    assert_eq!(rect.y, 0);
    assert_eq!(rect.width, 80);
    assert_eq!(rect.height, 24);
}

#[test]
fn layout_two_fixed_columns() {
    let mut strip = Strip::new(StripConfig {
        column_gap: 1,
        window_gap: 0,
        ..Default::default()
    });
    strip.resize_column(0, SizeConstraint::Fixed(20));
    // insert_window creates column 0 with default constraint, so resize after.
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.resize_column(0, SizeConstraint::Fixed(20));
    let b = strip.insert_window(1, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.resize_column(1, SizeConstraint::Fixed(30));

    let result = compute_layout(&strip, 80, 24);
    let ra = result.window_rects[&a];
    let rb = result.window_rects[&b];
    assert_eq!(ra.x, 0);
    assert_eq!(ra.width, 20);
    assert_eq!(rb.x, 21); // 20 + 1 gap
    assert_eq!(rb.width, 30);
}

#[test]
fn layout_stacked_windows_in_column() {
    let mut strip = Strip::new(StripConfig {
        column_gap: 0,
        window_gap: 0,
        ..Default::default()
    });
    let a = strip.insert_window(
        0,
        0,
        SizeConstraint::default(),
        SizeConstraint::Proportion(1.0),
    );
    let b = strip.insert_window(
        0,
        1,
        SizeConstraint::default(),
        SizeConstraint::Proportion(1.0),
    );

    let result = compute_layout(&strip, 80, 20);
    let ra = result.window_rects[&a];
    let rb = result.window_rects[&b];
    assert_eq!(ra.y, 0);
    assert_eq!(ra.height, 10);
    assert_eq!(rb.y, 10);
    assert_eq!(rb.height, 10);
    // Both span full column width.
    assert_eq!(ra.width, rb.width);
}

#[test]
fn layout_vertical_strip() {
    let mut strip = Strip::new(StripConfig {
        axis: Axis::Vertical,
        column_gap: 1,
        window_gap: 0,
    });
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.resize_column(0, SizeConstraint::Fixed(10));
    let b = strip.insert_window(1, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.resize_column(1, SizeConstraint::Fixed(15));

    let result = compute_layout(&strip, 80, 50);
    let ra = result.window_rects[&a];
    let rb = result.window_rects[&b];
    // In vertical mode, columns stack along Y.
    assert_eq!(ra.y, 0);
    assert_eq!(ra.height, 10);
    assert_eq!(rb.y, 11); // 10 + 1 gap
    assert_eq!(rb.height, 15);
    // Cross axis is X.
    assert_eq!(ra.x, 0);
    assert_eq!(ra.width, 80);
}

// ── Viewport & scrolling ───────────────────────────────────────────

#[test]
fn viewport_all_visible() {
    let mut strip = Strip::new(StripConfig {
        column_gap: 0,
        window_gap: 0,
        ..Default::default()
    });
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.resize_column(0, SizeConstraint::Fixed(30));
    strip.focus_set(a);

    let result = compute_layout(&strip, 80, 24);
    assert_eq!(result.visible.len(), 1);
    assert!(result.visible[0].fully_visible);
}

#[test]
fn viewport_offscreen_window() {
    let mut strip = Strip::new(StripConfig {
        column_gap: 0,
        window_gap: 0,
        ..Default::default()
    });
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.resize_column(0, SizeConstraint::Fixed(30));
    let _b = strip.insert_window(1, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.resize_column(1, SizeConstraint::Fixed(30));
    let c = strip.insert_window(2, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.resize_column(2, SizeConstraint::Fixed(30));

    // Focus on first window — viewport at start.
    strip.focus_set(a);
    let result = compute_layout(&strip, 40, 24);
    // Window c is at x=60, viewport only covers 0..40.
    let c_visible = result.visible.iter().find(|v| v.id == c);
    assert!(c_visible.is_none());
}

#[test]
fn viewport_focus_centers_window() {
    let mut strip = Strip::new(StripConfig {
        column_gap: 0,
        window_gap: 0,
        ..Default::default()
    });
    // Create many columns to push content beyond viewport.
    for i in 0..10 {
        strip.insert_window(i, 0, SizeConstraint::default(), SizeConstraint::default());
        strip.resize_column(i, SizeConstraint::Fixed(20));
    }
    // Focus on column 5 (at x=100).
    let col5_win = strip.columns[5].windows[0].id;
    strip.focus_set(col5_win);

    let result = compute_layout(&strip, 80, 24);
    // The focused window's center (x=110) should be near viewport center (offset+40).
    // offset ≈ 110 - 40 = 70
    assert!(result.scroll_offset > 0);
    // The focused window should be visible.
    assert!(result.visible.iter().any(|v| v.id == col5_win));
}

#[test]
fn viewport_manual_override() {
    let mut strip = Strip::new(StripConfig {
        column_gap: 0,
        window_gap: 0,
        ..Default::default()
    });
    for i in 0..5 {
        strip.insert_window(i, 0, SizeConstraint::default(), SizeConstraint::default());
        strip.resize_column(i, SizeConstraint::Fixed(20));
    }
    strip.focus_set(strip.columns[0].windows[0].id);
    strip.set_scroll_offset(50);

    let result = compute_layout(&strip, 40, 24);
    assert_eq!(result.scroll_offset, 50);

    // Re-enable focus tracking.
    strip.enable_focus_tracking();
    let result2 = compute_layout(&strip, 40, 24);
    // Focus is on column 0, so offset should go back toward 0.
    assert!(result2.scroll_offset < 50);
}

// ── Window management ──────────────────────────────────────────────

#[test]
fn insert_creates_intervening_columns() {
    let mut strip = Strip::new(StripConfig::default());
    let _id = strip.insert_window(3, 0, SizeConstraint::default(), SizeConstraint::default());
    assert_eq!(strip.column_count(), 4);
    assert_eq!(strip.columns[0].window_count(), 0);
    assert_eq!(strip.columns[3].window_count(), 1);
}

#[test]
fn insert_at_stack_position() {
    let mut strip = Strip::new(StripConfig::default());
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    let b = strip.insert_window(0, 100, SizeConstraint::default(), SizeConstraint::default());
    let c = strip.insert_window(0, 1, SizeConstraint::default(), SizeConstraint::default());

    let ids: Vec<_> = strip.columns[0].windows.iter().map(|w| w.id).collect();
    assert_eq!(ids, vec![a, c, b]);
}

#[test]
fn remove_cleans_up_empty_column() {
    let mut strip = Strip::new(StripConfig::default());
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    let _b = strip.insert_window(1, 0, SizeConstraint::default(), SizeConstraint::default());

    assert_eq!(strip.column_count(), 2);
    strip.remove_window(a);
    assert_eq!(strip.column_count(), 1);
}

#[test]
fn remove_nonexistent_is_noop() {
    let mut strip = Strip::new(StripConfig::default());
    let _a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    assert!(!strip.remove_window(crate::WindowId(999)));
}

#[test]
fn move_window_across_columns() {
    let mut strip = Strip::new(StripConfig::default());
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    let _b = strip.insert_window(1, 0, SizeConstraint::default(), SizeConstraint::default());

    assert!(strip.move_window(a, 1, 0));
    assert_eq!(strip.column_count(), 1); // Column 0 removed (was empty).
    assert_eq!(strip.columns[0].windows[0].id, a);
}

#[test]
fn move_window_within_column() {
    let mut strip = Strip::new(StripConfig::default());
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    let b = strip.insert_window(0, 1, SizeConstraint::default(), SizeConstraint::default());
    let c = strip.insert_window(0, 2, SizeConstraint::default(), SizeConstraint::default());

    strip.move_window(b, 0, 0);
    let ids: Vec<_> = strip.columns[0].windows.iter().map(|w| w.id).collect();
    assert_eq!(ids, vec![b, a, c]);
}

#[test]
fn resize_window_updates_constraints() {
    let mut strip = Strip::new(StripConfig::default());
    let a = strip.insert_window(0, 0, SizeConstraint::Fixed(10), SizeConstraint::Fixed(5));

    strip.resize_window(a, SizeConstraint::Proportion(1.0), SizeConstraint::Proportion(1.0));

    let (col, idx) = strip.find_window(a).unwrap();
    let w = &strip.columns[col].windows[idx];
    assert_eq!(w.width_constraint, SizeConstraint::Proportion(1.0));
}

#[test]
fn insert_column_shifts_right() {
    let mut strip = Strip::new(StripConfig::default());
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    let b = strip.insert_window(1, 0, SizeConstraint::default(), SizeConstraint::default());

    let c = strip.insert_column(1, SizeConstraint::Fixed(10), Some((SizeConstraint::default(), SizeConstraint::default())));
    assert!(c.is_some());
    assert_eq!(strip.column_count(), 3);
    // Original column 1 (with b) should now be at index 2.
    assert_eq!(strip.columns[2].windows[0].id, b);
    assert_eq!(strip.columns[0].windows[0].id, a);
}

#[test]
fn ids_never_reused() {
    let mut strip = Strip::new(StripConfig::default());
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.remove_window(a);
    let b = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    assert_ne!(a, b);
}

// ── Focus navigation ──────────────────────────────────────────────

#[test]
fn focus_left_right() {
    let mut strip = Strip::new(StripConfig {
        column_gap: 0,
        window_gap: 0,
        ..Default::default()
    });
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.resize_column(0, SizeConstraint::Fixed(20));
    let b = strip.insert_window(1, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.resize_column(1, SizeConstraint::Fixed(20));

    strip.focus_set(a);
    nav::focus_right(&mut strip, 80, 24);
    assert_eq!(strip.focused(), Some(b));

    nav::focus_left(&mut strip, 80, 24);
    assert_eq!(strip.focused(), Some(a));

    // Left at first column is no-op.
    nav::focus_left(&mut strip, 80, 24);
    assert_eq!(strip.focused(), Some(a));
}

#[test]
fn focus_up_down() {
    let mut strip = Strip::new(StripConfig {
        column_gap: 0,
        window_gap: 0,
        ..Default::default()
    });
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::Proportion(1.0));
    let b = strip.insert_window(0, 1, SizeConstraint::default(), SizeConstraint::Proportion(1.0));
    let c = strip.insert_window(0, 2, SizeConstraint::default(), SizeConstraint::Proportion(1.0));

    strip.focus_set(a);
    nav::focus_down(&mut strip);
    assert_eq!(strip.focused(), Some(b));
    nav::focus_down(&mut strip);
    assert_eq!(strip.focused(), Some(c));
    // Down at bottom is no-op.
    nav::focus_down(&mut strip);
    assert_eq!(strip.focused(), Some(c));

    nav::focus_up(&mut strip);
    assert_eq!(strip.focused(), Some(b));
}

#[test]
fn focus_first_last() {
    let mut strip = Strip::new(StripConfig::default());
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    let _b = strip.insert_window(1, 0, SizeConstraint::default(), SizeConstraint::default());
    let c = strip.insert_window(2, 0, SizeConstraint::default(), SizeConstraint::default());

    nav::focus_last(&mut strip);
    assert_eq!(strip.focused(), Some(c));
    nav::focus_first(&mut strip);
    assert_eq!(strip.focused(), Some(a));
}

#[test]
fn focus_first_empty_strip() {
    let mut strip = Strip::new(StripConfig::default());
    nav::focus_first(&mut strip);
    assert_eq!(strip.focused(), None);
}

#[test]
fn focus_affinity_preserved() {
    let mut strip = Strip::new(StripConfig {
        column_gap: 0,
        window_gap: 0,
        ..Default::default()
    });
    // Column 0: one tall window.
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.resize_column(0, SizeConstraint::Fixed(20));

    // Column 1: one small window at top.
    let b = strip.insert_window(1, 0, SizeConstraint::default(), SizeConstraint::Fixed(5));
    strip.resize_column(1, SizeConstraint::Fixed(20));

    // Column 2: two windows, second one is lower.
    let _c1 = strip.insert_window(2, 0, SizeConstraint::default(), SizeConstraint::Proportion(1.0));
    let c2 = strip.insert_window(2, 1, SizeConstraint::default(), SizeConstraint::Proportion(1.0));
    strip.resize_column(2, SizeConstraint::Fixed(20));

    // Focus on a (full height, center at row 12 in 24-tall viewport).
    strip.focus_set(a);

    // Move right — should go to b (only window in col 1).
    nav::focus_right(&mut strip, 80, 24);
    assert_eq!(strip.focused(), Some(b));

    // Move right again — affinity should still be ~row 12 (from a),
    // so it should pick c2 (lower half) not c1 (upper half).
    nav::focus_right(&mut strip, 80, 24);
    assert_eq!(strip.focused(), Some(c2));
}

#[test]
fn focus_affinity_reset_on_vertical_move() {
    let mut strip = Strip::new(StripConfig {
        column_gap: 0,
        window_gap: 0,
        ..Default::default()
    });
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::Proportion(1.0));
    let a2 = strip.insert_window(0, 1, SizeConstraint::default(), SizeConstraint::Proportion(1.0));
    strip.resize_column(0, SizeConstraint::Fixed(20));

    let b1 = strip.insert_window(1, 0, SizeConstraint::default(), SizeConstraint::Proportion(1.0));
    let _b2 = strip.insert_window(1, 1, SizeConstraint::default(), SizeConstraint::Proportion(1.0));
    strip.resize_column(1, SizeConstraint::Fixed(20));

    strip.focus_set(a);
    // Move right — sets affinity at a's center (row 6 in 24-tall viewport, top half).
    nav::focus_right(&mut strip, 80, 24);
    assert_eq!(strip.focused(), Some(b1));

    // Move left back to column 0.
    nav::focus_left(&mut strip, 80, 24);
    // Move down to a2.
    nav::focus_down(&mut strip);
    assert_eq!(strip.focused(), Some(a2));
    // Affinity should be reset. Cross affinity is now None.
    assert!(strip.cross_affinity.is_none());
}

#[test]
fn focus_removal_fallback() {
    let mut strip = Strip::new(StripConfig::default());
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    let b = strip.insert_window(0, 1, SizeConstraint::default(), SizeConstraint::default());

    strip.focus_set(a);
    strip.remove_window(a);
    // Should fall back to b (next in same column).
    assert_eq!(strip.focused(), Some(b));
}

#[test]
fn focus_removal_fallback_adjacent_column() {
    let mut strip = Strip::new(StripConfig::default());
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    let b = strip.insert_window(1, 0, SizeConstraint::default(), SizeConstraint::default());

    strip.focus_set(a);
    strip.remove_window(a);
    // Column 0 removed, should fall back to b in column 1 (now column 0).
    assert_eq!(strip.focused(), Some(b));
}

#[test]
fn focus_removal_empty_strip() {
    let mut strip = Strip::new(StripConfig::default());
    let a = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());

    strip.focus_set(a);
    strip.remove_window(a);
    assert_eq!(strip.focused(), None);
}

// ── Non-overlapping rects ──────────────────────────────────────────

#[test]
fn rects_never_overlap() {
    let mut strip = Strip::new(StripConfig {
        column_gap: 1,
        window_gap: 1,
        ..Default::default()
    });
    for col in 0..4 {
        for win in 0..3 {
            strip.insert_window(col, win, SizeConstraint::default(), SizeConstraint::Proportion(1.0));
        }
        strip.resize_column(col, SizeConstraint::Proportion(1.0));
    }

    let result = compute_layout(&strip, 80, 24);
    let rects: Vec<_> = result.window_rects.values().collect();
    for (i, a) in rects.iter().enumerate() {
        for b in rects.iter().skip(i + 1) {
            let x_overlap = a.x < b.x + b.width && b.x < a.x + a.width;
            let y_overlap = a.y < b.y + b.height && b.y < a.y + a.height;
            assert!(
                !(x_overlap && y_overlap),
                "Rects overlap: {:?} and {:?}",
                a,
                b
            );
        }
    }
}
