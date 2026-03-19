## Phase 1: New subwayrat widgets

- [x] Add `Completer` type alias and `with_completer`/`complete()` to `TextInput` in `rat-widgets/src/text_input.rs`
- [x] Add `path_completer` function in `rat-widgets/src/path_complete.rs`
- [x] Add tests for `TextInput::complete()` (single match, multi match, no match, no completer)
- [x] Add tests for `path_completer` (dir listing, prefix filter, slash append, empty input, nonexistent dir)
- [x] Add `GridSelect` and `GridItem` in `rat-widgets/src/grid_select.rs`
- [x] Add `GridSelect` navigation tests (left/right/up/down clamping, empty list)
- [x] Add `GridSelect` rendering with themed support and color swatches
- [x] Wire `grid_select` and `path_complete` modules into `rat-widgets/src/lib.rs`
- [x] `cargo test -p rat-widgets` passes

## Phase 2: rat-canvas crate

- [x] Create `crates/rat-canvas/Cargo.toml` with `serde` (optional) dep only
- [x] Implement `Position` type with `new`, `Eq`, `Hash`, `Copy`, `Serialize`/`Deserialize`
- [x] Implement `Viewport` type with `new`, `screen_to_canvas`, `canvas_to_screen`, `pan`, `zoom_in`, `zoom_out`, `reset_zoom`, `resize`, `visible_canvas_size`
- [x] Add zoom constants (`MIN_ZOOM`, `MAX_ZOOM`, `ZOOM_STEP`) with compile-time assertions
- [x] Add tests for all coordinate mapping scenarios from the canvas spec
- [x] Add to workspace `Cargo.toml` members list
- [x] `cargo test -p rat-canvas` passes

## Phase 3: rat-layers crate

- [x] Create `crates/rat-layers/Cargo.toml` with `uuid` and optional `serde` deps
- [x] Implement `LayerId` type (UUID v4, Eq, Hash, Copy, Display)
- [x] Implement `Layer` struct (id, name, visible, locked) with `new` and `with_id`
- [x] Implement `LayerStack<I>` with `new`, `add_layer`, `remove_layer`, `move_layer`, `rename_layer`
- [x] Implement visibility/lock: `set_visible`, `set_locked`, `is_visible`, `is_locked`
- [x] Implement item ownership: `set_item_layer`, `get_item_layer`, reassign on layer delete
- [x] Implement render ordering: `layers_bottom_to_top`, `z_index`
- [x] Add tests for all layer stack scenarios from the layers spec
- [x] Add to workspace `Cargo.toml` members list
- [x] `cargo test -p rat-layers` passes

## Phase 4: irohscii dependency wiring

- [x] Add `rat-keymap`, `rat-leaderkey`, `rat-widgets`, `rat-canvas`, `rat-layers` as workspace deps in irohscii `Cargo.toml`
- [x] Add deps to main binary `[dependencies]` section
- [x] `cargo check` passes with new deps (also fixed iroh 0.95→0.97 bump for aspen-automerge compat)

## Phase 5: irohscii-geometry → rat-canvas bridge

- [x] Add `rat-canvas` dependency to `irohscii-geometry/Cargo.toml`
- [x] Replace `Position` type in `irohscii-geometry` with re-export of `rat_canvas::Position`
- [x] Replace `Viewport` type in `irohscii-geometry` with re-export of `rat_canvas::Viewport`
- [x] Re-export zoom constants from `rat-canvas`
- [x] Keep all shape-specific geometry functions (`rect_points`, `line_points`, etc.) in `irohscii-geometry`
- [x] `cargo test --workspace` passes in irohscii (357 tests, 0 failures)

## Phase 6: irohscii-core → rat-layers bridge

- [x] Add `rat-layers` dependency to `irohscii-core/Cargo.toml`
- [x] Replace `LayerId` type with re-export of `rat_layers::LayerId`
- [x] Replace `Layer` struct with re-export of `rat_layers::Layer`
- [x] Adapt `Document` layer operations to delegate to a `LayerStack<ShapeId>` internally or keep automerge-backed storage and just share the types
- [x] `cargo test --workspace` passes in irohscii (357 tests, 0 failures)

## Phase 7: Replace irohscii leader key

- [x] Create `src/actions.rs` with `Action` enum covering all commands
- [x] Create `src/leader_menu.rs` implementing `MenuContributor<Action>` with all current leader.rs bindings
- [x] Replace `LeaderMenuState` in `modes/mod.rs` to use `rat_leaderkey::LeaderMenu<Action>`
- [x] Update leader key handling in mode dispatch to call `leader_menu.handle_key()`
- [x] Create shared `dispatch_action(Action, ModeContext)` function
- [x] Update `ui.rs` leader overlay rendering to call `leader_menu.render()`
- [x] Delete `modes/leader.rs`
- [x] Verify all leader key bindings work (manual spot check)

## Phase 8: Replace irohscii simple widgets

- [x] Replace `ConfirmDialogState` with `rat_widgets::ConfirmDialog`, update `ui.rs` rendering
- [x] Replace `SelectionPopupState` with `rat_widgets::GridSelect` for tool/color/brush popups
- [x] Replace `PathInputState` tab completion with `rat_widgets::path_completer`
- [x] Update `ui.rs` rendering for path input, confirm, popup to use rat-widgets
- [x] Adapt existing mode tests for the new widget-backed states

## Phase 9: Replace irohscii list modes

- [x] Replace `RecentFilesState` rendering with `rat_widgets::ScrollableList`
- [x] Update `ui.rs` rendering for recent files to use `ScrollableList::render()`

## Phase 10: Replace irohscii normal mode keybindings

- [x] Create `src/keybindings.rs` building `Keymap<Action, InputMode>` with all current normal.rs bindings
- [x] Restructure `modes/normal.rs`: keymap lookup first, then fallback for movement keys
- [x] Wire `dispatch_action` (from phase 7) as the shared handler for both keymap and leader menu results
- [x] `cargo test --workspace` passes in irohscii (336 tests, 0 failures)

## Phase 11: Cleanup

- [x] Run `cargo clippy --fix` on subwayrat — fixed 4 lints
- [x] Verify `cargo test --workspace` passes in both repos (subwayrat: 349 tests, irohscii: 336 tests)
