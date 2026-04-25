## 1. Crate Scaffolding

- [x] 1.1 Create `crates/rat-tree/` with `Cargo.toml` (deps: ratatui, rat-keymap, unicode-width; edition 2024, MIT)
- [x] 1.2 Add `crates/rat-tree` to workspace `Cargo.toml` members
- [x] 1.3 Create `src/lib.rs` with module declarations and public re-exports

## 2. Tree Data Model

- [x] 2.1 Define `TreeData` trait with `root_count`, `root`, `child_count`, `child`, `node_label`, `node_icon`
- [x] 2.2 Implement `SimpleTree` adapter struct (from flat `(id, Option<parent_id>, label)` tuples)
- [x] 2.3 Define `VisibleRow` struct (node id, depth, has_children, is_expanded, is_last_sibling)
- [x] 2.4 Implement `compute_visible_rows` function that walks `TreeData` + expanded set → `Vec<VisibleRow>`
- [x] 2.5 Tests: visible rows with all collapsed, single expand, nested expand, collapse hides descendants

## 3. Tree State

- [x] 3.1 Define `TreeState` struct (cursor index, scroll_offset, expanded: `BTreeSet<usize>`, cached visible rows)
- [x] 3.2 Implement expand/collapse/toggle methods that mutate expanded set and recompute visible rows
- [x] 3.3 Implement cursor-adjust-on-collapse (move cursor to collapsed node if on a descendant)
- [x] 3.4 Tests: expand/collapse state transitions, cursor adjustment on collapse

## 4. Tree Navigation

- [x] 4.1 Implement `move_up` / `move_down` with clamping at boundaries
- [x] 4.2 Implement `jump_first` / `jump_last`
- [x] 4.3 Implement `navigate_parent` (find parent's visible row, move cursor)
- [x] 4.4 Implement `navigate_first_child` (auto-expand if collapsed, move cursor to first child)
- [x] 4.5 Implement `next_sibling` / `prev_sibling` (skip expanded subtrees)
- [x] 4.6 Implement `page_up` / `page_down` with page_size parameter
- [x] 4.7 Implement scroll-follows-cursor logic (adjust scroll_offset to keep cursor in viewport)
- [x] 4.8 Tests: all navigation methods including edge cases (root, leaf, last sibling, empty tree)

## 5. Keymap Integration

- [x] 5.1 Define `TreeAction` enum with all variants (Up, Down, Expand, Collapse, Toggle, Parent, FirstChild, NextSibling, PrevSibling, First, Last, PageUp, PageDown, Select)
- [x] 5.2 Implement `parse_tree_action(s: &str) -> Option<TreeAction>` for override support
- [x] 5.3 Implement `default_keymap() -> Keymap<TreeAction, ()>` with vim-style bindings
- [x] 5.4 Implement `TreeState::apply(action: TreeAction, data: &impl TreeData, viewport_height: usize)` dispatch method
- [x] 5.5 Tests: default keymap resolves expected keys, parse_tree_action round-trips, apply dispatches correctly

## 6. Rendering

- [x] 6.1 Define `TreeStyle` struct with defaults (indent width, guide chars, indicator chars, styles)
- [x] 6.2 Implement builder methods (`with_indent_width`, `with_guide_chars`, `with_selected_style`, etc.)
- [x] 6.3 Define `TreeInfo` struct and `TreeState::info()` method
- [x] 6.4 Implement `Tree` widget struct with builder for style, data reference, and block
- [x] 6.5 Implement `StatefulWidget` for `Tree` — render visible rows with indent guides, indicators, icons, selection highlight
- [x] 6.6 Tests: guide line generation (tee vs corner for last sibling), indicator selection (expand/collapse/leaf), info snapshot accuracy

## 7. Integration

- [x] 7.1 Verify `cargo check` passes for the crate and workspace
- [x] 7.2 Verify `cargo test` passes for rat-tree
- [x] 7.3 Add doc comments and crate-level documentation with usage example
