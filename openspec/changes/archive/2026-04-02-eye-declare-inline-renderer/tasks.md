## 1. ratcore inline module

- [x] 1.1 Add `inline` module to ratcore with `ViewTree`, `ViewNode`, `NodeKey` types (key, type_tag via `TypeId`, opaque `Box<dyn Any>` state slot)
- [x] 1.2 Implement `reconcile(old: &[ViewNode], new: Vec<ViewNode>) -> Vec<ViewNode>` — key-based matching via HashMap, then positional matching by type_tag
- [x] 1.3 Implement `compute_commits(node_heights: &[u16], viewport_height: u16, scroll_offset: u16) -> Vec<usize>` — returns indices of fully evicted nodes
- [x] 1.4 Write tests: reordered keyed nodes preserve state
- [x] 1.5 Write tests: type mismatch at position creates new node, appended nodes get fresh state
- [x] 1.6 Write tests: compute_commits returns correct indices for various height/viewport combinations

## 2. rat-inline crate scaffold

- [x] 2.1 Create `crates/rat-inline/` with Cargo.toml (deps: ratcore, ratatui, crossterm, unicode-width), add to subwayrat workspace members
- [x] 2.2 Create `src/lib.rs` with module declarations and public re-exports
- [x] 2.3 Verify `cargo check -p rat-inline` passes

## 3. InlineWidget trait and basic widgets

- [x] 3.1 Define `InlineWidget` trait: `height(&self, width: u16) -> u16` and `render(&self, area: Rect, buf: &mut Buffer)`
- [x] 3.2 Implement `InlineText` widget: styled text with word wrapping, implements `InlineWidget`
- [x] 3.3 Implement `InlineWidget` wrapper for `rat-markdown` rendered lines (takes markdown source + style, computes height from line count)

## 4. Builder API

- [x] 4.1 Implement `InlineView` builder: `.push(widget)` adds an unkeyed node, `.keyed(key, widget)` adds a keyed node
- [x] 4.2 Implement `.text(string)` shorthand — wraps string in `InlineText`
- [x] 4.3 Implement `.when(bool, |builder| ...)` conditional
- [x] 4.4 Implement `.each(iter, |builder, item| ...)` loop
- [x] 4.5 Implement `.build() -> ViewTree` — converts builder state to ratcore `ViewTree`
- [x] 4.6 Write tests for each builder method

## 5. Inline renderer core

- [x] 5.1 Implement `InlineRenderer` struct: holds current/previous ratatui `Buffer`, ratcore `ViewTree`, tracks `claimed_rows` and terminal width
- [x] 5.2 Implement `rebuild(view: InlineView)` — builds `ViewTree`, calls `ratcore::inline::reconcile` against previous tree, stores result
- [x] 5.3 Implement height measurement pass: iterate nodes, call `InlineWidget::height(width)`, sum for total buffer height
- [x] 5.4 Implement render pass: allocate `Rect` per node (stacked vertically), call `InlineWidget::render()` into buffer
- [x] 5.5 Implement frame diffing: compare current vs previous buffer cell-by-cell, emit ANSI only for changed cells
- [x] 5.6 Implement terminal growth: emit `\n` to claim new rows when content height increases, clear excess rows on shrink
- [x] 5.7 Wrap diff output in DEC synchronized output sequences (`?2026h`/`l`)
- [x] 5.8 Implement width change detection: re-measure and rebuild when `crossterm::terminal::size()` width changes

## 6. Scrollback commit

- [x] 6.1 Add `on_commit` callback field to `InlineRenderer`
- [x] 6.2 After each render, call `ratcore::inline::compute_commits` with node heights and terminal height
- [x] 6.3 Fire `on_commit` with evicted node keys, remove committed nodes from the active tree
- [x] 6.4 Write test: content exceeding terminal height triggers commit callback with correct keys

## 7. StreamingOutput integration

- [x] 7.1 Add `rat-inline` as optional dependency of `rat-streaming` behind an `inline` feature flag
- [x] 7.2 Implement `InlineWidget` for `StreamingOutput`: height from `min(total_display_lines, visible_lines)`, render via existing `render_streaming_lines`
- [x] 7.3 Write test: `StreamingOutput` renders correctly through the inline renderer

## 8. Integration tests and example

- [x] 8.1 Write end-to-end test: build a view tree with markdown + text, render two frames, verify diff output is minimal
- [x] 8.2 Write end-to-end test: rebuild with reordered keyed nodes, verify state preservation and correct output
- [x] 8.3 Add `cargo run --example inline_demo` in rat-inline: streams styled agent-like output to terminal scrollback
