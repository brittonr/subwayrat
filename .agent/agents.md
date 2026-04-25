# Agent Instructions

## Showcase

When adding a new widget crate to the workspace, always add a demo to
`crates/showcase/`. Add a tab or section that exercises the widget's
core interactions so it's visible alongside everything else. Wire up
keybindings and show relevant state in an info panel when it makes sense.

## Build

`.cargo-check.sh` hard-codes `/home/brittonr/git/subwayrat`; inside
`.pi/worktrees/...`, run `cargo` directly with the PATH from that script so
checks run against the active worktree. If cargo waits on the shared build
lock, set `CARGO_TARGET_DIR=/tmp/subwayrat-target-tests` for validation.
Before requesting done-review, make sure the checkout being reviewed contains
the actual code diff; changes left only under `.pi/worktrees/...` may not be
visible to reviewers pointed at the main repo.
