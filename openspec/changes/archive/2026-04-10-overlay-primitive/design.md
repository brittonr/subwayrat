## Context

`subwayrat` already has several overlay-like renderers, but each crate recomputes centered popup rectangles, clears the region, draws a border, and manually derives an inner content area. The duplication is visible in `rat-widgets`, `rat-branches`, `rat-leaderkey`, `rat-streaming`, and `rat-capture`. At the same time, the existing `os-ui-primitives` change identified overlay rendering as a shared concern for dialogs, tooltips, and context menus, but that change is broad and not yet implementation-ready.

This change narrows the problem to a single primitive: draw an overlay frame over an existing viewport and return the child content area. The workspace prefers focused crates, builder-style APIs, explicit layout outputs, Rust 2024, MIT licensing, and `ratatui` as the rendering dependency.

## Goals / Non-Goals

**Goals:**
- Introduce a reusable overlay/frame primitive in a new `rat-chrome` crate.
- Support anchored placement at center, edges, and corners.
- Support fixed and percentage sizing for width and height.
- Support optional backdrop dimming, optional clearing/fill, and optional border/title chrome.
- Return computed outer and inner rects so callers render their own body content.
- Provide a foundation that existing popup-style widgets can migrate to incrementally.

**Non-Goals:**
- Animation, easing, or slide transitions.
- Mouse hit testing or click-outside dismissal.
- Owning dialog, menu, tooltip, or toast content.
- Solving global z-ordering across multiple overlays.
- Refactoring all existing popup users in the same change.

## Decisions

### Create `rat-chrome` as a small structural crate

The primitive belongs in `rat-chrome`, not `rat-widgets`. `rat-widgets` holds leaf widgets that mostly render their own contents, while an overlay frame is structural: it defines where other content should render. Starting `rat-chrome` with one small primitive also de-risks the larger `os-ui-primitives` effort.

**Alternative considered:** add the primitive to `rat-widgets`. Rejected because it mixes framing/layout responsibilities into the leaf-widget crate and makes the future crate split harder.

### Use explicit model + render output instead of `StatefulWidget`

The render API should return layout information such as `outer` and `inner` rects. That matches the existing `os-ui-primitives` design direction and avoids hiding layout output inside mutable state. A typical call shape is:

```rust
let layout = overlay_frame(frame, viewport, &model, &style);
frame.render_widget(dialog_body, layout.inner);
```

**Alternative considered:** model the primitive after `tui-overlay` with `StatefulWidget` and stored inner area. Rejected for v1 because this workspace already prefers explicit return values for structural components, and animation state is out of scope.

### Use narrow sizing enums instead of fully generic ratatui constraints

The primitive only needs fixed and percentage sizing in v1. A dedicated size enum keeps the API honest and spec-aligned, instead of accepting unrelated `Constraint` variants such as `Min` or `Fill` that would need extra interpretation.

**Alternative considered:** use `ratatui::layout::Constraint` directly. Rejected because it widens the contract beyond the behavior this change promises.

### Separate structural configuration from visual styling

`OverlayModel` carries behavioral configuration: anchor, width, height, offsets, dimming enabled, clear enabled, and optional title text. `OverlayStyle` carries border, title, backdrop, and fill styles. This follows the workspace’s state/style split and keeps rendering testable.

**Alternative considered:** a single overlay struct containing both behavior and style. Rejected because it couples runtime state to theme choices and makes reuse harder.

### Clamp overlay rectangles to the viewport and compute inner area from rendered chrome

The overlay rect is computed from the requested size and anchor, then clamped to the viewport. If border/title chrome is enabled, the returned inner rect is derived from the actual rendered frame, not from the requested size, so callers can trust it even on small terminals.

**Alternative considered:** reject oversized requests or return invalid rects. Rejected because terminal UIs need graceful degradation when space is tight.

## Risks / Trade-offs

- **[Another small crate in the workspace]** → Adds one more crate to maintain. Mitigation: keep the first version tiny and immediately useful, with a single exported primitive.
- **[Spec overlap with `os-ui-primitives`]** → The broader change also discusses dialogs and overlays. Mitigation: treat this change as the reusable substrate that `os-ui-primitives` can depend on or absorb later.
- **[Existing widgets keep duplicating logic for a while]** → This change does not migrate every popup caller. Mitigation: document intended follow-up adopters and leave their current behavior unchanged.
- **[No animation support]** → The result will not match `tui-overlay` feature-for-feature. Mitigation: keep the API extensible so animation can be layered on later without changing the basic layout contract.

## Migration Plan

1. Add `crates/rat-chrome` and expose the overlay primitive.
2. Add unit tests for placement, sizing, clamping, and returned inner rect computation.
3. Optionally migrate one showcase/demo usage as proof of composition.
4. Follow up with targeted refactors in popup-heavy crates after the primitive has stabilized.

## Open Questions

- Should the first version expose title rendering directly, or should callers compose a border-only frame and render titles themselves?
- Should fill/clear be separate toggles, or is clear-always with optional backdrop enough for v1?
- Should future dialog/context-menu helpers live in the same crate or re-export this primitive from a higher-level module?
