## Why

The workspace has widgets for flat lists (rat-table), text editing (rat-editor), and tree algorithms (rat-branches), but no widget for navigating hierarchical data interactively. File browsers, outlines, dependency graphs, and config editors all need a tree view with expand/collapse, cursor movement, and keyboard-driven navigation. Building this on top of rat-keymap gives consumers configurable, modal keybindings out of the box.

## What Changes

- New crate `rat-tree` providing a generic, stateful tree navigation widget for ratatui.
- Tree state tracks which nodes are expanded/collapsed, which node has cursor focus, and vertical scroll position.
- Renders indented rows with expand/collapse indicators, guide lines, and per-node icons.
- Integrates with `rat-keymap` so consumers define their own action enum and bind tree actions to keys, including modal bindings.
- Follows the workspace conventions: builder pattern, style/state/info structs, `StatefulWidget` trait, Rust 2024 edition.

## Capabilities

### New Capabilities
- `tree-model`: Generic tree data model — trait for nodes, flattened visible-row computation, expand/collapse state tracking.
- `tree-navigation`: Cursor movement (up/down/parent/first-child/next-sibling/prev-sibling), expand/collapse/toggle, jump-to-first/last, scroll management.
- `tree-rendering`: Indented row rendering with guide lines, expand/collapse indicators, icons, selection highlight, and configurable styles.
- `tree-keymap`: Integration with `rat-keymap` — a `TreeAction` enum, default key bindings, and the wiring to resolve key events into tree operations.

### Modified Capabilities

(none)

## Impact

- New crate `crates/rat-tree` added to workspace `Cargo.toml`.
- Depends on `ratatui` (workspace), `rat-keymap` (path dep), and `unicode-width` (workspace).
- No breaking changes to existing crates.
