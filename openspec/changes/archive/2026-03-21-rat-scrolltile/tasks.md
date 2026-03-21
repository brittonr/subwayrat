## 1. Crate Scaffold

- [x] 1.1 Create `crates/rat-scrolltile/` with Cargo.toml (ratatui dependency only), `src/lib.rs`, and add to workspace members
- [x] 1.2 Define core types: `WindowId`, `SizeConstraint` enum (Fixed, Proportion, Min, MinMax), `Axis` enum (Horizontal, Vertical), `StripConfig` (axis, column_gap, window_gap)

## 2. Data Model

- [x] 2.1 Implement `Window` struct (id, width_constraint, height_constraint) with ID generation that never reuses IDs
- [x] 2.2 Implement `Column` struct (windows vec, width_constraint) with stack operations (insert at index, remove by ID, reorder)
- [x] 2.3 Implement `Strip` struct (columns vec, config, focus state, ID counter) as the top-level container

## 3. Strip Layout Algorithm

- [x] 3.1 Implement column width resolution: partition viewport primary-axis space among columns respecting Fixed/Proportion/Min/MinMax constraints
- [x] 3.2 Implement window height resolution: partition column cross-axis space among windows using the same constraint solver
- [x] 3.3 Implement `compute_layout()` — walk columns, assign strip-space x positions with gaps, walk windows per column assigning y positions, produce `Rect` per window
- [x] 3.4 Add `LayoutResult` struct containing: per-window `Rect` map (in strip-space), total strip extent, scroll offset, list of visible windows with viewport-local Rects
- [x] 3.5 Add tests: horizontal strip with mixed fixed/proportional columns, vertical strip, single-column, empty strip, constraint clamping

## 4. Viewport & Scrolling

- [x] 4.1 Implement viewport clipping: given scroll offset and viewport size, compute which windows intersect, produce clipped viewport-local Rects
- [x] 4.2 Implement focus-driven scroll: compute offset that centers the focused window, clamp to [0, max_scroll], skip adjustment when window is already visible
- [x] 4.3 Implement manual scroll offset override with re-enable flag for focus tracking
- [x] 4.4 Add tests: window fully visible, partially visible, off-screen, centering at strip edges, manual override

## 5. Window Management Operations

- [x] 5.1 Implement `insert_window(column_idx, stack_pos, constraints)` → WindowId, with auto-creation of intervening columns
- [x] 5.2 Implement `remove_window(id)` with empty-column cleanup and focus fallback to nearest neighbor
- [x] 5.3 Implement `move_window(id, target_column, target_stack_pos)` as atomic remove+insert
- [x] 5.4 Implement `resize_window(id, new_constraints)` and `resize_column(idx, new_constraint)`
- [x] 5.5 Implement `insert_column(idx, initial_window)` with shift-right semantics
- [x] 5.6 Add tests: insert into existing/new column, remove middle/last window, move across columns, move within column, resize

## 6. Focus Navigation

- [x] 6.1 Implement `focus_set(id)`, `focus_clear()`, `focused() -> Option<WindowId>`
- [x] 6.2 Implement `focus_left()` / `focus_right()` with cross-axis positional affinity (find closest window by vertical center)
- [x] 6.3 Implement `focus_up()` / `focus_down()` within column, no-op at boundaries
- [x] 6.4 Implement `focus_first()` / `focus_last()` jumping to strip endpoints
- [x] 6.5 Implement cross-axis affinity memory: store affinity on left/right move, reset on up/down move
- [x] 6.6 Implement focus fallback on window removal: same column preferred, then adjacent column, then None
- [x] 6.7 Add tests: navigation across columns with varying stack sizes, affinity preservation, removal fallback, empty strip

## 7. Public API & Docs

- [x] 7.1 Define the public module structure: `lib.rs` re-exports, module split (types, layout, viewport, nav, strip)
- [x] 7.2 Write rustdoc for all public types and methods with usage examples in lib.rs doc comment
- [x] 7.3 Add `crates/rat-scrolltile/examples/basic.rs` — create a strip, add windows, compute layout, print Rects

## 8. Showcase Integration

- [x] 8.1 Add a "Scroll Tiler" page to the showcase app demonstrating strip navigation with a few colored panels
