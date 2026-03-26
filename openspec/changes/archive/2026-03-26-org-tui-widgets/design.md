## Context

SubwayRat is a Rust TUI widget toolkit built on ratatui. It follows a consistent architecture: each widget is a separate crate with a `StatefulWidget` impl, a state struct owned by the application, and an action/event enum for input handling. Existing crates like `rat-spreadsheet`, `rat-tree`, and `rat-streaming` demonstrate this pattern at scale.

The goal is to add 8 new crates that cover the UI surface needed for org-mode-style workflows in the terminal. The backend logic (file parsing, agenda computation, roam graphs, export) is explicitly out of scope — that's org2's job. These widgets consume pre-computed data and handle all rendering and interaction.

Existing crates that new code builds on:
- `rat-editor`: multi-line buffer with cursor, history, undo. 831 LoC.
- `rat-tree`: hierarchical tree with expand/collapse, keymap, navigation. 1641 LoC.
- `rat-spreadsheet`: full spreadsheet with formulas and cell editing. 3244 LoC.
- `rat-widgets`: TextInput, SelectList, ScrollableList, TabBar, Confirm, etc.
- `rat-keymap` + `rat-leaderkey`: key combo and leader-key systems.

## Goals / Non-Goals

**Goals:**
- Each new crate is self-contained with no coupling to org2 or any specific document format
- Widgets accept generic data through traits, not org-specific types
- Consistent API pattern: `Widget` + `WidgetState` + `Action` enum + `handle_action()` function
- All widgets work with keyboard-only input; mouse is optional
- Each crate compiles and tests independently

**Non-Goals:**
- File I/O, parsing org/markdown files, or any disk operations
- Network calls to org2 CLI or LSP — the caller handles that
- Full Emacs-style extensibility or elisp-like configuration
- Org export (HTML, PDF, LaTeX) — that's org2's domain
- Undo/redo across widget boundaries — each widget manages its own state

## Decisions

### 1. rat-outline wraps rat-editor's buffer, doesn't fork it

`rat-outline` composes `rat-editor::Editor` for the text buffer and adds a parallel `Vec<HeadingInfo>` index that tracks heading positions, levels, fold state, and metadata (TODO state, tags, priority). When the buffer changes, the heading index is rebuilt incrementally.

**Alternative considered**: Fork `rat-editor` into a new heading-aware editor. Rejected because it duplicates cursor/undo/history logic and creates maintenance burden. Composition keeps the buffer generic.

**Alternative considered**: Extend `rat-tree` with inline editing. Rejected because `rat-tree` renders one line per node — an outline needs multi-line body text between headings.

### 2. Data-in via traits, not concrete types

Every widget that consumes external data uses a trait:
- `rat-agenda`: `AgendaDataSource` trait with `fn items(&self, range: DateRange) -> Vec<AgendaItem>`
- `rat-fuzzy`: `FuzzySource` trait with `fn candidates(&self) -> &[FuzzyCandidate]`
- `rat-backlinks`: `BacklinkSource` trait with `fn backlinks(&self, target_id: &str) -> Vec<Backlink>`

This means the caller can back these with org2 CLI, a SQLite DB, static data, or anything else. The widgets never know.

**Alternative considered**: Accept `Vec<T>` directly in state constructors. Rejected because it requires the caller to rebuild the full list on every update. Traits allow lazy loading and incremental refresh.

### 3. rat-fuzzy uses a scoring algorithm, not substring match

Fuzzy matching uses a Smith-Waterman-style scoring that rewards consecutive character matches and penalizes gaps. This matches the behavior users expect from fzf/telescope. The scorer is internal to the crate — no external fuzzy-match dependency.

**Alternative considered**: Pull in the `fuzzy-matcher` crate. Rejected to avoid adding a dependency for ~200 lines of scoring logic.

### 4. rat-datepicker is two widgets: CalendarGrid + TimeInput

The calendar popup is a `CalendarGrid` that renders a month view with arrow-key navigation. `TimeInput` is a separate widget for HH:MM entry. The caller composes them however they want — side by side, stacked, or only one of them.

**Alternative considered**: Single monolithic `DateTimePicker`. Rejected because many use cases only need a date or only need a time.

### 5. rat-capture is a layered overlay, not a separate screen

Capture uses `rat-layers` to render as a centered popup over the current view. It contains a `TextInput` for the title, the `rat-editor` for the body, and a `SelectList` for the template. The lifecycle is: open → fill → confirm/cancel → close. State is ephemeral.

**Alternative considered**: Full-screen capture view. Rejected because the point of capture is speed — you don't want to lose context of what you're looking at.

### 6. Org table bridge is an optional module in rat-spreadsheet, not a new crate

Adding `rat-spreadsheet::org_table` behind a `org-compat` feature flag keeps the parsing/serialization close to the data structures it operates on. The module provides `from_org_table(text: &str) -> Grid` and `to_org_table(grid: &Grid) -> String`, plus a formula syntax translator.

**Alternative considered**: Separate `rat-org-table` crate. Rejected because the bridge is thin (~300 LoC of parsing) and deeply coupled to `Grid`/`CellValue` internals.

### 7. Tag completion reuses rat-widgets::SelectList with a custom renderer

`rat-tags` wraps `SelectList` for the tag completion popup, adding `:tag:` syntax awareness, multi-select (tags are additive), and a colon-delimited display format. Property editing is a vertical key-value list using `TextInput` pairs.

**Alternative considered**: Build tag completion from scratch. Rejected because `SelectList` already handles filtering, keyboard navigation, and rendering — the tag layer just adds syntax rules.

### 8. Action enums are non-exhaustive

All new `Action` enums use `#[non_exhaustive]` so adding new actions in future versions doesn't break callers who match on them. This follows the pattern already used in `rat-spreadsheet`.

## Risks / Trade-offs

**[Risk] rat-outline heading index gets out of sync with buffer** → The heading index is rebuilt on every buffer mutation. For files under 10K lines this is <1ms. For larger files, incremental updates (re-scan only the changed region) can be added later without API changes.

**[Risk] Fuzzy scoring is too slow for large candidate sets (>100K headings)** → Initial implementation scores synchronously on keystroke. If profiling shows this is a problem, scoring moves to a background thread with debounced input. The `FuzzySource` trait already allows lazy evaluation.

**[Risk] rat-capture overlay interferes with the host app's layer stack** → Capture pushes/pops a single layer. It does not manage other layers. If the host app has its own layer system, it needs to coordinate. Document this clearly.

**[Trade-off] No undo for structural operations in rat-outline** → Promote/demote/move-subtree modify multiple lines at once. The initial implementation does not group these into a single undo step in `rat-editor`. Users can undo character-by-character through the changes. Grouped undo is a future enhancement.

**[Trade-off] rat-agenda has no built-in recurring event expansion** → The widget renders whatever items it's given. Repeater expansion ("+1w" means show every week) is the data source's responsibility. This keeps the widget simple but means the caller must handle recurrence.
