## 1. Workspace Setup

- [x] 1.1 Add new `crates/rat-chrome` crate to the workspace with crate docs and public module structure
- [x] 1.2 Add `ratatui` and any needed workspace dependencies to `crates/rat-chrome/Cargo.toml`
- [x] 1.3 Add a minimal showcase integration or demo surface for the overlay primitive

## 2. Overlay Model and Layout

- [x] 2.1 Define anchor and size enums covering center, edges, corners, fixed sizes, and percentage sizes
- [x] 2.2 Define `OverlayModel` with builder-style configuration for anchor, width, height, offsets, clear, dimming, and optional title/chrome settings
- [x] 2.3 Implement overlay outer-rect computation from viewport, anchor, size, and offsets
- [x] 2.4 Implement clamping so oversized overlay requests stay within viewport bounds
- [x] 2.5 Define a returned layout struct containing at least outer and inner rects

## 3. Rendering and Styling

- [x] 3.1 Define `OverlayStyle` with border, title, backdrop, and fill styling
- [x] 3.2 Implement overlay rendering that optionally dims the viewport outside the overlay rect
- [x] 3.3 Implement overlay rendering that optionally clears the overlay region before drawing chrome
- [x] 3.4 Implement border/title chrome rendering and compute the final inner content rect from the rendered frame
- [x] 3.5 Export a render API that callers can use to draw the overlay and then render body content into the returned inner rect

## 4. Validation

- [x] 4.1 Add unit tests for fixed-size centering, percentage sizing, edge anchoring, and viewport clamping
- [x] 4.2 Add unit tests for returned inner rect behavior with and without border chrome
- [x] 4.3 Add unit tests for clear and backdrop behavior
- [x] 4.4 Run `cargo check` for the workspace and `cargo test -p rat-chrome`

## 5. Follow-up Adoption Notes

- [x] 5.1 Document intended migration targets in existing popup-heavy crates (`rat-widgets`, `rat-branches`, `rat-leaderkey`, `rat-streaming`, `rat-capture`)
- [x] 5.2 Note deferred work explicitly: animation, hit testing, and broader popup refactors remain follow-up changes
