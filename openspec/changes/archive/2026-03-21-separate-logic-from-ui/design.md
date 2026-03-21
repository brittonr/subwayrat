## Context

The subwayrat workspace contains 17 ratatui widget crates. Several mix business logic (buffer management, search algorithms, graph mutation) directly into rendering structs. The crate boundaries are already well-drawn — the problem is within-crate entanglement, not cross-crate coupling. All affected types are workspace-internal with no published API guarantees.

The `rat-spreadsheet` crate already demonstrates good separation (cell.rs, formula.rs, nav.rs are pure logic), proving the pattern works. The goal is to bring the remaining crates to that standard.

## Goals / Non-Goals

**Goals:**
- Pure data structs for streaming buffers, search state, selection models — testable without ratatui types
- Graph view state borrows the graph model instead of owning it
- Input handlers return pure intents; callers apply mutations
- Each crate's `lib.rs` re-exports stay the same where possible (minimize downstream churn)
- `EditState` lives in its own module, not in `render.rs`

**Non-Goals:**
- Rewriting rendering logic — only moving it to separate functions/widgets
- Splitting `SpreadsheetState` into two structs (breaks `StatefulWidget`). Fields get reorganized, not separated.
- Changing any crate's public dependency surface
- Performance optimization — this is a structural refactor only
- Touching crates that are already clean (`rat-keymap`, `rat-canvas`, `rat-diff`, `rat-markdown`, etc.)

## Decisions

### 1. Streaming buffer: extract `StreamingOutput` core into render-free struct

The current `StreamingOutput` has `render_lines()` and `render_stats()` that produce `Line<'a>` with hardcoded styles. Extract the data layer:

- `StreamingOutput` keeps: `push_line`, `push_text`, scroll methods, `get_display_line`, `display_line_count`, byte/line counters. No ratatui imports.
- New free functions `render_streaming_lines()` and `render_streaming_stats()` take `&StreamingOutput` plus style params and return `Vec<Line<'a>>` / `Line<'a>`.
- `StreamingOutputManager` stays as-is (it's already a pure HashMap wrapper around buffers).

**Why not a separate widget struct?** The streaming output isn't rendered via `StatefulWidget` — it produces `Line` fragments that get composed into a larger chat view. Free functions match the existing usage pattern.

### 2. Search state: extract `OutputSearch` matching from rendering

- `OutputSearch` keeps: query, matches, mode, navigation, `update_matches()`, `find_substring_matches()`, `find_fuzzy_matches()`. No ratatui imports.
- New `render_search_overlay()` free function takes `&OutputSearch`, `Frame`, `Rect`.
- `apply_search_highlights()` stays as a standalone function (it already is) but moves to a `render` submodule or stays in the same file with the rendering functions.

### 3. Node graph: borrow graph instead of owning it

Current `NodeGraphState` has `pub graph: Graph`. Input handlers call `self.graph.add_edge()` directly, then also return `GraphAction` events — double bookkeeping.

New approach:
- `NodeGraphState` drops the `graph` field. Keeps: `viewport`, `selected`, `focused`, `focused_port`, `mode`, `selected_edge`, `tab_order`, `area`.
- `handle_mouse_click(&mut self, graph: &mut Graph, ...)` takes graph as param.
- `handle_key(&mut self, graph: &mut Graph, ...)` same.
- `NodeGraphWidget::render()` signature becomes `render(self, area, buf, state: &mut NodeGraphState, graph: &Graph)` — but this breaks `StatefulWidget` trait. So instead: `NodeGraphState` gets a `set_graph(&mut self, graph: &Graph)` pattern... No.

**Revised decision:** Keep `NodeGraphState` holding the graph BUT change input handlers to only return `GraphAction` intents, not mutate the graph. Add an `apply_action(&mut self, action: &GraphAction)` method that performs the actual mutation. This preserves `StatefulWidget` compatibility and gives callers the option to intercept/reject actions.

**Alternative considered:** Borrowing `&mut Graph` in every method. Rejected because `StatefulWidget::render` takes `&mut Self::State` — we can't split the lifetime.

### 4. SelectList: split model from widget

- `SelectListModel`: `items: Vec<String>`, `selected: usize`, `visible: bool`, `move_up()`, `move_down()`, `select() -> Option<String>`.
- `SelectListWidget<'a>`: borrows `&'a SelectListModel`, holds title, optional theme. Implements rendering.
- Keep a convenience `SelectList` type alias or constructor that bundles both for simple cases.

### 5. SpreadsheetState: reorganize, move EditState

- Move `EditState` from `render.rs` to `edit_state.rs`. Re-export from `lib.rs`.
- Group `SpreadsheetState` fields with doc comments separating concerns: data model fields (`grid`, `dep_graph`, `fn_registry`, `validators`, `last_undo`) vs. visual layout fields (`col_widths`, `default_col_width`, `min_col_width`, `frozen_rows`, `frozen_cols`, `style_callback`). No struct split — just documentation and module relocation.

## Risks / Trade-offs

**[Internal API churn]** → All changes are workspace-internal. The `showcase` binary and examples will need updating. Mitigated by keeping re-exports stable at `lib.rs` level.

**[NodeGraphState still owns Graph]** → We chose to keep ownership for `StatefulWidget` compatibility. The trade-off is that `Graph` can't be shared or borrowed from elsewhere. This is acceptable — the graph editor is the sole owner in all current usage. Future work could introduce a `GraphRef` pattern if sharing is needed.

**[Render function signatures grow]** → Free functions like `render_streaming_lines(&StreamingOutput, visible_height, style)` have more params than the current `self.render_lines()`. Mitigated by using style/config structs to bundle params.

**[Test migration]** → Tests in `streaming_output.rs` that call `render_lines()` move to the render module. The data-layer tests stay in place. Small risk of missing a test during migration.
