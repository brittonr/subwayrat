## Why

irohscii (collaborative ASCII art editor) has ~3,400 lines of hand-rolled TUI
code — leader menus, confirm dialogs, text inputs, selection popups, list
browsers, keybinding dispatch. subwayrat already has reusable widgets for most
of these patterns (`rat-leaderkey`, `rat-keymap`, `rat-widgets`) but is missing
two things irohscii needs: filesystem path completion and 2D grid selection.
Additionally, irohscii's viewport/pan/zoom canvas and layer system are generic
patterns with no subwayrat equivalent.

Adding the missing widgets and abstractions to subwayrat, then migrating
irohscii to use them, cuts ~1,900 lines from irohscii while making subwayrat
more capable for any TUI app that needs canvas navigation or structured
overlays.

## What Changes

- Add tab-completing path input to `rat-widgets::TextInput` via a `Completer`
  callback, plus a bundled `path_completer` function.
- Add `GridSelect` widget to `rat-widgets` for 2D grid popup selection with
  arrow/hjkl navigation and optional color swatches.
- Add `rat-canvas` crate: generic infinite-canvas viewport with pan, zoom,
  coordinate mapping (screen↔canvas), and cell-based rendering. Extracted from
  irohscii's `irohscii-geometry::Viewport` + `Position`.
- Add `rat-layers` crate: ordered layer stack with visibility/lock toggles, a
  layer panel widget, and layer-aware item ownership. Extracted from
  irohscii's `irohscii-core::Layer` / `LayerId`.
- Migrate irohscii's TUI modes to use `rat-leaderkey`, `rat-keymap`,
  `rat-widgets`, `rat-canvas`, and `rat-layers`, deleting the hand-rolled
  equivalents.

## Capabilities

### New Capabilities
- `path-completion`: Tab-completing filesystem path input for TextInput.
- `grid-select`: 2D grid popup selection widget with arrow/hjkl navigation.
- `canvas`: Generic infinite-canvas viewport with pan/zoom and coordinate
  mapping for cell-based TUI applications.
- `layers`: Ordered layer stack with visibility/lock state, ownership tracking,
  and a layer panel widget.

### Modified Capabilities

## Impact

- **subwayrat crates**: New files in `rat-widgets` (`path_complete.rs`,
  `grid_select.rs`), new crates `rat-canvas` and `rat-layers`.
- **irohscii binary**: `modes/leader.rs`, `modes/confirm.rs`,
  `modes/popup.rs` deleted. `modes/path_input.rs`, `modes/label_input.rs`,
  `modes/layer_rename.rs`, `modes/session.rs`, `modes/recent_files.rs`
  reduced to thin wrappers. `modes/normal.rs` restructured around
  `rat-keymap`. Canvas rendering in `ui.rs` switches to `rat-canvas` API.
  Layer management in `app/mod.rs` switches to `rat-layers`.
- **irohscii library crates**: `irohscii-geometry::Viewport` and
  `irohscii-core::Layer`/`LayerId` become thin re-exports of `rat-canvas`
  and `rat-layers` types, or are replaced entirely.
- **Dependencies**: irohscii gains workspace deps on `rat-keymap`,
  `rat-leaderkey`, `rat-widgets`, `rat-canvas`, `rat-layers`.
