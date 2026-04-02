## Why

clankers renders agent conversations in a full-screen ratatui TUI. That works for the interactive dashboard, but it's wrong for the conversation stream itself — messages accumulate, tool outputs grow, and the user needs to scroll back through history. eye-declare (atuinsh/eye-declare) demonstrates that an inline rendering model — content grows into terminal scrollback, frame diffing, reconciliation — is the right approach for this kind of output.

We already split rendering-agnostic logic into ratcore (tree, leaderkey, fuzzy, caldate) and wrap it in subwayrat (ratatui) and meteorite (dioxus). The inline renderer should follow the same pattern: the reconciler and view tree model live in ratcore, subwayrat provides a ratatui terminal backend, and meteorite can provide a dioxus component backend later.

On the dioxus side: meteorite already uses `rsx!` for declarative view composition. Rather than inventing a custom `inline!` proc macro, the subwayrat backend should provide a builder API for constructing inline view trees. A proc macro can come later if the builder is too verbose, but the reconciler and rendering engine are the hard parts — not the syntax.

## What Changes

- New `ratcore::inline` module: framework-agnostic inline view tree, reconciler (key/position matching with state preservation), and commit tracking for scrollback eviction
- New `rat-inline` crate in subwayrat: ratatui terminal backend that renders the ratcore view tree into terminal scrollback with frame diffing, ANSI output, terminal growth tracking, and DEC synchronized output
- Builder API in rat-inline for constructing view trees (no proc macro dependency to start)
- `InlineWidget` trait bridging existing rat-* widgets (rat-markdown, rat-streaming) into the inline view tree
- Integration point in clankers: `clankers -p "..." --inline` streams styled conversation output without the full-screen TUI
- Future: meteorite can wrap `ratcore::inline` in a dioxus component that renders inline content via `rsx!`

## Capabilities

### New Capabilities
- `inline-reconciler`: Framework-agnostic reconciler in ratcore — key/position node matching, state preservation across rebuilds, scrollback commit tracking. No UI dependencies.
- `inline-renderer`: ratatui terminal backend in subwayrat — frame buffer, terminal growth, ANSI diff output, synchronized update wrapping, width tracking on resize
- `inline-builder`: Builder API for constructing inline view trees from rat-* widgets without a proc macro

### Modified Capabilities
- `streaming-buffer`: StreamingOutput gains an `InlineWidget` trait impl so it can participate in inline view trees

## Impact

- ratcore: new `inline` module added (reconciler, view tree types, commit tracking). Zero new dependencies — stays pure logic.
- New crate: `crates/rat-inline` added to subwayrat workspace. Depends on ratcore + ratatui + crossterm.
- `rat-streaming`: minor trait impl addition behind feature flag, no breaking changes
- `rat-markdown`: used as-is inside inline views, no changes needed
- clankers `clankers-tui`: no changes — inline mode is a separate rendering path
- meteorite: no changes now. Future `met-inline` crate can wrap `ratcore::inline` in dioxus components using `rsx!`
