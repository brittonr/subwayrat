# Napkin

## Corrections
| Date | Source | What Went Wrong | What To Do Instead |
|------|--------|----------------|-------------------|
| 2026-03-19 | self | Tried multiple nix store Rust toolchains before finding one that works | Use `/nix/store/yx30bd3n73w18d3r6pdxw1ira1sdfq12-rust-default-1.95.0-nightly-2026-02-06/bin` + `/nix/store/a245z3cvf9x9sn0xlk6k8j9xhxbhda1z-gcc-wrapper-15.2.0/bin` (x86_64 gcc, not i686) |
| 2026-03-19 | self | Used `ref expr` in pattern match with Rust 2024 edition | Edition 2024 has implicit borrowing in patterns - drop the `ref` |
| 2026-03-19 | self | Undo test assumed Enter-edit replaces content | Enter-edit appends to existing; use `EnterEdit(Some('c'))` for fresh edit |
| 2026-04-11 | self | Refactored `rat-widgets::Loader` into a state/widget split without a compatibility shim | Keep the old public API working and layer new stateful/stateless types underneath it; preserve old default styling semantics in `render()` |
| 2026-04-11 | self | Claimed archive/test verification without evidence in hand | Do not mark OpenSpec verification done or report tests unless command output was actually captured |

## User Preferences
- Workspace of ratatui widget crates under `crates/rat-*`
- Rust 2024 edition, MIT license
- Builder pattern APIs, ratatui StatefulWidget conventions
- openspec for change management

## Patterns That Work
- Each widget is its own crate with focused scope
- DataTable pattern: state struct + style struct + info struct + render method
- Builder pattern with `with_*` methods
- Delegating parallel crate modules to separate workers speeds up implementation
- `.cargo-check.sh` script wraps toolchain PATH for builds
- Navigation modules: separate cursor/selection/scroll state with pure functions
- Using `get_selection(&cursor)` function instead of field access for computed state

## Patterns That Don't Work
- Storing computed selection as a field instead of computing it on-demand
- Using `HashMap` for ID-keyed collections that get iterated — nondeterministic order causes flaky tests. Use `BTreeMap` for anything with monotonic IDs.

## Domain Notes
- subwayrat: a collection of ratatui TUI widgets
- rat-table exists as a read-only scrollable data table
- rat-editor exists for text editing
- rat-selection handles selection models
- rat-tree exists for interactive tree navigation with keymap integration
- openspec schema is `spec-driven`, config at `openspec/changes/<name>/.openspec.yaml`
- Doc-test examples referencing `rat_tree` (or any workspace crate) from within the crate need `ignore` or `no_run` — the doctest harness can't resolve `use rat_tree::...` as an external crate
