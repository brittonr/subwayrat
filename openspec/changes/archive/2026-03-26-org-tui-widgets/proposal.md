## Why

SubwayRat has a solid set of general-purpose TUI widgets (editor, table, spreadsheet, tree, streaming output) but nothing purpose-built for structured document workflows — the kind org-mode users depend on daily. Org2 provides a portable CLI/LSP backend for org-like files, but it ships zero terminal UI. By building the missing TUI widgets, SubwayRat becomes the rendering layer for org-style workflows in the terminal: outlining, agenda planning, date picking, quick capture, fuzzy navigation, and backlink browsing. The backend logic (parsing, agenda computation, roam graph) stays in org2; SubwayRat owns the interactive surface.

## What Changes

- New `rat-outline` crate: a folding structured editor that understands heading hierarchy, visibility cycling, and structural operations (promote/demote/move subtree). Built on top of `rat-editor`'s buffer primitives.
- New `rat-agenda` crate: agenda/calendar view widget that renders day/week/month grids, time-block items, and filter/sort controls. Consumes pre-computed agenda data (from org2 CLI or any source).
- New `rat-datepicker` crate: calendar popup and time input widget for selecting dates, time-of-day, and repeater intervals.
- New `rat-capture` crate: transient overlay mini-buffer for quick note/task capture with template selection and target picker.
- New `rat-fuzzy` crate: incremental fuzzy finder widget for searching headings, files, tags, or any ranked item list. Used for refile targets, link insertion, and general navigation.
- New `rat-backlinks` crate: side panel or popup widget that displays incoming references to the current heading, grouped by file with context snippets.
- New `rat-tags` crate: tag completion popup and property drawer editor for heading metadata (tags, properties, priority cookies).
- Adapter module in `rat-spreadsheet` for reading/writing org pipe-table format and bridging org field-formula syntax to the existing formula engine.

## Capabilities

### New Capabilities
- `outline-editing`: Folding structured editor — heading hierarchy, visibility cycling (folded/children/all), promote/demote, move-subtree, TODO state cycling, inline heading markup
- `agenda-view`: Calendar/agenda widget — day/week/month grid layout, time-block rendering, multi-source item display, filter bar, sort/group controls
- `date-picker`: Date and time selection — month-grid calendar popup, time input, repeater interval input, keyboard-driven navigation
- `capture-overlay`: Quick capture mini-buffer — template selector, title+body input, target file/heading picker, transient overlay lifecycle
- `fuzzy-finder`: Incremental fuzzy search widget — ranked match display, file+heading context, keyboard navigation, pluggable data sources
- `backlinks-panel`: Incoming reference display — grouped-by-file list, context snippets around each link, jump-to-source navigation
- `tag-property-editor`: Heading metadata editing — tag completion with `:tag:` syntax, property key-value editor, priority cookie cycling
- `org-table-bridge`: Org pipe-table adapter for rat-spreadsheet — parse `| a | b |` format, serialize back, map org field-formula syntax to existing formula engine

### Modified Capabilities
<!-- No existing spec requirements change. The new crates are additive. -->

## Impact

- **New crates**: 8 new workspace members under `crates/`
- **Existing crates touched**: `rat-spreadsheet` gains an optional `org-table` module; `rat-editor` may gain trait hooks that `rat-outline` builds on
- **Dependencies**: No new external dependencies beyond what the workspace already uses (ratatui, crossterm, unicode-width). The org2 CLI integration is the caller's responsibility, not baked into these widgets.
- **API surface**: Each new crate exposes a ratatui `StatefulWidget` + state struct + action/event enum, following the same pattern as `rat-spreadsheet` and `rat-tree`
- **Testing**: Each crate gets unit tests for state logic and snapshot tests for rendering. The `showcase` binary gains new tabs for each widget.
