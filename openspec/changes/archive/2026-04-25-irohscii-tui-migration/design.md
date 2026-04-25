## Context

subwayrat is a workspace of reusable ratatui widget crates. irohscii is a
collaborative ASCII art editor with ~30k lines across 6 library crates and a
~14k line binary. irohscii's TUI layer duplicates patterns subwayrat already
solves (leader menus, text inputs, confirm dialogs, list selection) and also
contains two generic abstractions (infinite canvas viewport, layer stack) that
have no subwayrat equivalent.

Both projects use ratatui 0.30 and Rust edition 2024. irohscii currently has
no dependency on subwayrat.

## Goals / Non-Goals

**Goals:**

- Add path completion and grid selection to `rat-widgets` so irohscii (and
  other TUI apps) can use them.
- Extract the canvas viewport and layer stack patterns into `rat-canvas` and
  `rat-layers` as standalone crates with no ratatui dependency in their core.
- Migrate irohscii's TUI modes to use subwayrat crates, deleting ~1,900
  lines of hand-rolled code.
- Keep irohscii's domain logic (shapes, CRDT document, sync, export)
  untouched.

**Non-Goals:**

- Rewriting irohscii's shape rendering or geometry algorithms — those stay
  in `irohscii-geometry` and `irohscii-core`.
- Adding a generic "shape system" to subwayrat — shapes are domain-specific.
- Making `rat-canvas` or `rat-layers` depend on automerge or any CRDT —
  irohscii's CRDT layer sits above these crates.
- Changing irohscii's P2P sync protocol or session management.

## Decisions

### 1. Completer as a callback, not a trait

**Choice:** `TextInput::with_completer(Box<dyn Fn(&str) -> Vec<String>>)`

**Rationale:** A closure is simpler than a trait for a single method. The
caller doesn't need to define a struct — `Box::new(path_completer)` or
`Box::new(|s| my_custom_complete(s))` works inline. The `TextInput` already
owns its state; adding one more `Option<Box<...>>` field is cheap.

**Alternative:** A `Completer` trait with `fn complete(&self, input: &str)
-> Vec<String>`. More extensible (stateful completers) but adds ceremony for
the common case. Can be added later if needed — the callback signature is
compatible.

### 2. GridSelect as a separate widget, not a mode on SelectList

**Choice:** New `GridSelect` struct alongside `SelectList`.

**Rationale:** `SelectList` is 1D — its API (`move_up`/`move_down`) and
rendering (single column list) don't map to 2D grids. Adding a `columns`
field to `SelectList` would complicate every method. A separate widget with
`move_left`/`move_right`/`move_up`/`move_down` is cleaner.

**Alternative:** Add `columns: Option<usize>` to `SelectList` and branch in
navigation/rendering. Rejected because it muddies the existing simple API.

### 3. rat-canvas is coordinate math only, no ratatui dep

**Choice:** `rat-canvas` provides `Position`, `Viewport`, pan/zoom/resize.
No widget, no `Frame`, no ratatui. Rendering is the caller's job.

**Rationale:** The viewport is useful in contexts beyond ratatui (testing,
headless export, WASM). irohscii's `ui.rs` already has a custom render loop
that iterates canvas cells — wrapping that in a ratatui `Widget` trait would
fight the grain. The crate stays tiny (~300 lines) and dependency-free.

**Alternative:** Include a `CanvasWidget` that implements `ratatui::Widget`.
Rejected because canvas rendering varies wildly per application (ASCII art
vs. tilemap vs. diagram). The abstraction boundary is the coordinate math,
not the rendering.

### 4. rat-layers is generic over item ID

**Choice:** `LayerStack<I: Eq + Hash + Copy>` where `I` is the item
identifier type. irohscii uses `ShapeId`, other apps use their own.

**Rationale:** Layers are a generic pattern (image editors, map editors,
diagram tools). The stack needs to track item→layer assignments but shouldn't
know what items are. A type parameter keeps it generic without trait objects.

**Alternative:** Use `u64` as the item ID and have callers convert.
Rejected — forces lossy conversions for UUID-based IDs.

### 5. irohscii-geometry re-exports rat-canvas types

**Choice:** After migration, `irohscii-geometry` depends on `rat-canvas`
and re-exports `Position`, `Viewport`. Shape-specific geometry functions
(`rect_points`, `ellipse_points`, etc.) stay in `irohscii-geometry`.

**Rationale:** Avoids a flag-day rewrite of every import in irohscii. The
re-export layer is zero-cost. Over time, direct `rat-canvas` imports can
replace the re-exports.

**Alternative:** Delete `irohscii-geometry` and use `rat-canvas` directly
everywhere. Higher risk — touches every file in irohscii.

### 6. Migration order: subwayrat first, irohscii second

**Choice:** Build and test all new subwayrat crates/widgets before touching
irohscii. Then migrate irohscii mode by mode.

**Rationale:** Subwayrat additions are self-contained and testable in
isolation. Migrating irohscii with proven widgets reduces debugging surface.
Each irohscii mode can be migrated in a separate commit.

## Risks / Trade-offs

**[Path completer tests depend on filesystem]** → Tests use `tempdir` with
known file structure. No reliance on system paths.

**[GridSelect popup sizing]** → Grid cells have variable label widths. Risk
of overflow on narrow terminals. Mitigation: clamp popup width to
`area.width - 4`, truncate labels.

**[rat-canvas duplicates irohscii-geometry::Position]** → During migration
both types exist. Mitigation: `irohscii-geometry` re-exports `rat-canvas`
types, so downstream code sees one type. Aliasing, not duplication.

**[rat-layers generic parameter adds complexity]** → The type parameter
propagates to `LayerStack<I>` everywhere. Mitigation: most consumers use
a single concrete `I` and never think about the generic.

**[irohscii regression during migration]** → Each mode replacement changes
input handling. Mitigation: irohscii has mode-level unit tests for every
key handler. Preserve and adapt these tests during migration.
