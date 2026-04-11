## Why

The workspace has multiple crates that hand-roll the same popup math and clearing logic for centered dialogs, search bars, switchers, and capture overlays. That duplication makes styling and behavior drift over time, and it blocks the planned `rat-chrome` work from starting with a small reusable primitive instead of a large all-at-once chrome crate.

## What Changes

- Add a new `rat-chrome` crate to the workspace with a small overlay/frame primitive for ratatui.
- Introduce an `OverlayModel` and `OverlayStyle` with builder-style configuration for anchor, size, offsets, clearing, backdrop dimming, and optional border/title chrome.
- Add a render API that draws the overlay and returns the inner content `Rect` so callers render their own body content.
- Support fixed and percentage sizing, plus anchored placement for center, corners, and edges.
- Document this primitive as the shared foundation for popups, modals, drawers, tooltips, and future dialog/context-menu helpers.
- Keep v1 non-animated and keyboard-first; slide transitions and richer mouse behavior stay out of scope.

## Capabilities

### New Capabilities
- `overlay-frame`: composable overlay placement and frame rendering with backdrop, border chrome, and returned child content rect

### Modified Capabilities
- None.

## Impact

- New crate: `crates/rat-chrome`
- Workspace updates in `Cargo.toml`
- New OpenSpec capability spec for overlay framing behavior
- Future consumers include duplicated popup renderers in `rat-widgets`, `rat-branches`, `rat-leaderkey`, `rat-streaming`, and `rat-capture`
- No new external dependency required; implementation uses workspace `ratatui`
