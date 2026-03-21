## Why

Several crates in the workspace embed business logic (search algorithms, buffer management, graph mutation) directly inside rendering structs or rendering modules. This makes the logic untestable without ratatui types, prevents theming/restyling without touching logic, and creates ownership tangles (e.g., `NodeGraphState` owns the `Graph` model). Separating these concerns now reduces coupling before more widgets are built on top.

## What Changes

- Extract `StreamingOutput` buffer management (head/tail ring, scroll offsets, byte tracking) from its `render_lines()`/`render_stats()` methods into a pure data struct. Rendering becomes a separate function that borrows the buffer.
- Extract `OutputSearch` matching logic (substring, fuzzy, smart-case, match navigation) from its `render()` popup and `apply_search_highlights()` into a pure search state struct. Rendering becomes a separate function.
- Decouple `NodeGraphState` from owning `Graph`. Input handlers should return action intents without directly mutating the graph — the caller applies them. The graph is borrowed for rendering, not stored.
- Extract `SelectList` data model (items, selection index, visibility) from its rendering methods into a standalone model struct.
- Move `EditState` out of `rat-spreadsheet/src/render.rs` into its own module. Split `SpreadsheetState` visual layout fields (`col_widths`, `frozen_rows/cols`, display config) from data model fields (`grid`, `dep_graph`, `fn_registry`, `validators`).

## Capabilities

### New Capabilities
- `streaming-buffer`: Pure data buffer for head/tail truncated output with scroll state, independent of rendering.
- `search-state`: Pure search state machine (query, matches, navigation, modes) independent of rendering.
- `nodegraph-state-split`: Decoupled graph view state that borrows the graph model rather than owning it, with pure-intent input handling.
- `select-list-model`: Pure selection model (items, index, visibility) independent of rendering.
- `spreadsheet-state-split`: Separated `EditState` module and split `SpreadsheetState` into visual layout config vs. data model config.

### Modified Capabilities
- `grid-rendering`: `SpreadsheetState` struct changes — visual fields split from data fields.
- `cell-editing`: `EditState` moves from `render.rs` to its own module; re-exported path changes.
- `graph-interaction`: `NodeGraphState` no longer owns `Graph`; input handlers return intents instead of mutating directly.
- `node-rendering`: Widget borrows graph from caller instead of reading it from state.

## Impact

- **rat-streaming**: `StreamingOutput` and `OutputSearch` public API changes. Callers that call `render_lines()`/`render_stats()` on the buffer or `render()` on the search must switch to standalone render functions.
- **rat-nodegraph**: `NodeGraphState` no longer has a `pub graph: Graph` field. Callers must own the graph separately and pass it to render/input methods. `handle_mouse_click` and `handle_key` signatures change to borrow `&mut Graph`.
- **rat-widgets**: `SelectList` splits into `SelectListModel` + `SelectListWidget`. Callers update construction.
- **rat-spreadsheet**: `EditState` import path moves. `SpreadsheetState` internal field grouping changes but the struct remains one type (fields reorganized, not split into two structs) to preserve `StatefulWidget` compatibility.
- **No external crate API changes** — all affected types are workspace-internal.
