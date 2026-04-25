# Napkin

## Corrections
| Date | Source | What Went Wrong | What To Do Instead |
|------|--------|----------------|-------------------|
| 2026-03-19 | self | Tried multiple nix store Rust toolchains before finding one that works | Use `/nix/store/yx30bd3n73w18d3r6pdxw1ira1sdfq12-rust-default-1.95.0-nightly-2026-02-06/bin` + `/nix/store/a245z3cvf9x9sn0xlk6k8j9xhxbhda1z-gcc-wrapper-15.2.0/bin` (x86_64 gcc, not i686) |
| 2026-03-19 | self | Used `ref expr` in pattern match with Rust 2024 edition | Edition 2024 has implicit borrowing in patterns - drop the `ref` |
| 2026-03-19 | self | Undo test assumed Enter-edit replaces content | Enter-edit appends to existing; use `EnterEdit(Some('c'))` for fresh edit |
| 2026-04-11 | self | Refactored `rat-widgets::Loader` into a state/widget split without a compatibility shim | Keep the old public API working and layer new stateful/stateless types underneath it; preserve old default styling semantics in `render()` |
| 2026-04-11 | self | Claimed archive/test verification without evidence in hand | Do not mark OpenSpec verification done or report tests unless command output was actually captured |
| 2026-04-11 | self | Matched `self.frames` by value inside `SpinnerSpec::label(&self)` | Match borrowed enum fields by reference unless type is intentionally `Copy` end-to-end |
| 2026-04-23 | user/self | Updated `ratcore` at the flake layer when the real dependency edge belongs in Cargo | For extracted Rust crates, move the dependency in `Cargo.toml` first; only keep Nix-specific support files that the build actually needs |
| 2026-04-25 | done-review | Let reviewer inspect main checkout while code changes only existed in `.pi/worktrees/...` | Before `request_done_review`, ensure the actual repo root being reviewed contains the code diff, and mark untracked files with intent-to-add so `git diff` shows test/code additions |

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
- `.cargo-check.sh` script wraps toolchain PATH for builds, but it hard-codes `/home/brittonr/git/subwayrat`; in `.pi/worktrees/...` run `cargo` directly with the PATH from the script so tests hit the active worktree
- Navigation modules: separate cursor/selection/scroll state with pure functions
- Using `get_selection(&cursor)` function instead of field access for computed state
- Cargo SSH git deps in this workspace need `.cargo/config.toml` with `git-fetch-with-cli = true`, and unit2nix needs `crate-hashes.json` for the pinned git rev
- Ratcore API drift may break re-export crates: `rat-fuzzy` scoring now takes `FuzzyScoreInput`, and `rat-tree` node ids use `ratcore::tree::NodeId` (`u32`) rather than `usize`
- Shared `/home/brittonr/.cargo-target` can block on unrelated builds; use `CARGO_TARGET_DIR=/tmp/subwayrat-target-tests` for final worktree validation when cargo waits on the build-directory lock

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

## Promotion Backlog
- [class=pattern scope=proposal route=spec-rule promoted=yes] Promotion candidate (2x omission/proposal/spec-rule): Strengthen OpenSpec proposal gate/template to block repeated omission findings. Example trigger: Completed OpenSpec change remains active
- [class=pattern scope=review route=deterministic-check promoted=yes] Promotion candidate (2x omission/review/prompt): Tighten reviewer/prompt guidance for repeated omission findings, then promote to a spec rule or deterministic check if it persists. Example trigger: Claimed validation is not evidenced
- [class=pattern scope=test route=deterministic-check promoted=yes] Promotion candidate (2x omission/test/prompt): Add deterministic rail for repeated omission findings in test: test, lint, TigerStyle audit, compile-time check, or proof. Example trigger: Showcase exporter was not verified by a successful build
