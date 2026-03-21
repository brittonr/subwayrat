## 1. rat-streaming: Extract buffer logic from rendering

- [x] 1.1 Remove `render_lines()` and `render_stats()` methods from `StreamingOutput` in `streaming_output.rs`. Remove all `ratatui` imports from that file (Style, Color, Modifier, Line, Span).
- [x] 1.2 Make `get_display_line()` and `DisplayLine` enum public so render functions can access display data.
- [x] 1.3 Create `streaming_render.rs` with free functions `render_streaming_lines(&StreamingOutput, visible_height, Style) -> Vec<Line>` and `render_streaming_stats(&StreamingOutput, Style) -> Line`. Move the rendering logic from the old methods here.
- [x] 1.4 Move render-related tests (test_render_lines_basic, test_render_lines_with_truncation, test_render_stats_footer, test_empty_buffer_render) from `streaming_output.rs` to `streaming_render.rs`.
- [x] 1.5 Update `lib.rs` to declare `mod streaming_render` and re-export the new render functions.

## 2. rat-streaming: Extract search state from rendering

- [x] 2.1 Remove the `render()` method from `OutputSearch` in `output_search.rs`. Remove ratatui imports (Frame, Rect, Color, Modifier, Style, Line, Span, Block, Borders, Clear, Paragraph) from that file.
- [x] 2.2 Move `apply_search_highlights()` out of `output_search.rs` into a new `search_render.rs` file.
- [x] 2.3 Create `render_search_overlay(search: &OutputSearch, frame: &mut Frame, area: Rect)` free function in `search_render.rs` with the popup rendering logic.
- [x] 2.4 Move render-related tests to `search_render.rs`. Keep match-logic tests in `output_search.rs`.
- [x] 2.5 Update `lib.rs` to declare `mod search_render` and re-export `render_search_overlay` and `apply_search_highlights`.

## 3. rat-nodegraph: Intent-only input handlers

- [x] 3.1 In `view.rs`, refactor `handle_mouse_click` to NOT call `self.graph.add_edge()`. Instead, check compatibility via `self.graph.port()` reads and return `GraphAction::EdgeCreated` as an intent. Track wiring cancellation as before (view-layer state).
- [x] 3.2 Refactor `handle_key` to NOT call `self.graph.remove_edge()` or mutate `self.graph.node_mut().x/y`. Return `GraphAction::EdgeDeleted` and `GraphAction::NodeMoved` intents instead.
- [x] 3.3 Refactor `handle_mouse_drag` to NOT mutate `self.graph.node_mut().x/y`. Return `GraphAction::NodeMoved` intents.
- [x] 3.4 Add `apply_action(&mut self, action: &GraphAction)` method on `NodeGraphState` that performs the actual graph mutations: `add_edge`, `remove_edge`, and node position updates.
- [x] 3.5 Update all tests in `view.rs` to call `apply_action` after input handlers where graph mutations are expected. Verify that graph state is unchanged between handler return and apply.

## 4. rat-widgets: Split SelectList model from rendering

- [x] 4.1 Create `SelectListModel` struct in `select_list.rs` with fields `items: Vec<String>`, `selected: usize`, `visible: bool` and methods `new()`, `move_up()`, `move_down()`, `select() -> Option<String>`.
- [x] 4.2 Refactor `SelectList` to hold a `SelectListModel` plus `title: String`. Change `render()` and `render_themed()` to read from `self.model`.
- [x] 4.3 Expose `model` field as `pub` so callers can access `SelectListModel` directly. Keep `move_up()`, `move_down()`, `select()` delegation methods on `SelectList` for ergonomics.
- [x] 4.4 Re-export `SelectListModel` from `lib.rs`.

## 5. rat-spreadsheet: Move EditState, reorganize SpreadsheetState

- [x] 5.1 Create `crates/rat-spreadsheet/src/edit_state.rs` and move the `EditState` struct, its `impl` block, and `impl Default` from `render.rs` into it.
- [x] 5.2 In `render.rs`, replace the `EditState` definition with `use crate::edit_state::EditState;`.
- [x] 5.3 In `lib.rs`, add `pub mod edit_state;` and add `pub use edit_state::EditState;` to the re-exports.
- [x] 5.4 Reorganize `SpreadsheetState` fields with comment headers: `// -- Data model --` for `grid`, `dep_graph`, `fn_registry`, `validators`, `last_undo`, `edit`; and `// -- Visual layout --` for `col_widths`, `default_col_width`, `min_col_width`, `frozen_rows`, `frozen_cols`, `style_callback`.
- [x] 5.5 Move `EditState` tests from `render.rs` `mod tests` into `edit_state.rs` `mod tests`.

## 6. Verify and clean up

- [x] 6.1 Run `cargo check --workspace` and fix any compilation errors from the refactor.
- [x] 6.2 Run `cargo test --workspace` and verify all existing tests pass.
- [x] 6.3 Update `showcase` example if it references any moved types or changed function signatures.
