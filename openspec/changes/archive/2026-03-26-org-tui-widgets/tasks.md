## 1. rat-outline — Folding structured editor

- [x] 1.1 Create `crates/rat-outline` crate scaffold: Cargo.toml (depends on rat-editor, ratatui), src/lib.rs with module declarations
- [x] 1.2 Implement `HeadingSyntax` enum (Org, Markdown) and `HeadingParser` trait with default parsers that extract level, TODO state, priority, tags, title from a line
- [x] 1.3 Implement `HeadingInfo` struct and heading index builder: scan buffer lines, produce `Vec<HeadingInfo>` with line numbers, levels, fold states
- [x] 1.4 Implement `OutlineState` wrapping `rat_editor::Editor` + heading index + fold states + TODO keyword config + scroll offset
- [x] 1.5 Implement visibility cycling logic: folded→children→all→folded per heading, plus global fold/unfold
- [x] 1.6 Implement visible-line computation: filter buffer lines through fold states, produce the set of line indices to render
- [x] 1.7 Implement structural editing: promote, demote (adjust heading markers for subtree), move-subtree-up, move-subtree-down (swap buffer regions)
- [x] 1.8 Implement TODO state cycling: advance keyword on heading line, wrap past last to None, insert first keyword from None
- [x] 1.9 Define `Action` enum (#[non_exhaustive]) and `handle_action()` function dispatching outline actions and delegating text edits to inner Editor
- [x] 1.10 Implement `StatefulWidget` rendering: gutter with fold indicators (▶/▼), heading styles, TODO/priority/tag styles via `OutlineStyle`
- [x] 1.11 Write unit tests: heading parsing for org and markdown syntax, fold state cycling, promote/demote, move subtree, TODO cycling
- [x] 1.12 Add rat-outline tab to showcase binary (deferred to group 9)

## 2. rat-agenda — Calendar/agenda view

- [x] 2.1 Create `crates/rat-agenda` crate scaffold: Cargo.toml (depends on ratatui), src/lib.rs
- [x] 2.2 Implement `AgendaItem` struct and `AgendaDataSource` trait with `fn items(&self, range: DateRange) -> Vec<AgendaItem>`
- [x] 2.3 Implement `ViewMode` enum (Day, Week, Month) and `AgendaState` with selected date, view mode, item list, filter state, scroll offset
- [x] 2.4 Implement day view rendering: time-sorted timed items first, then untimed items by priority, with time slot / priority / status / title / tags columns
- [x] 2.5 Implement week view rendering: 7-column layout with day headers, item lists per column, selected day highlight
- [x] 2.6 Implement month view rendering: 7×6 grid with day numbers, item counts per cell, today highlight, out-of-month dimming
- [x] 2.7 Implement filter bar: status filter, tag filter, priority filter, free-text search; AND combination; horizontal rendering above content
- [x] 2.8 Define `Action` enum and `handle_action()`: navigation (next/prev day/week/month), view switching, item selection, filter toggle/set/clear, refresh
- [x] 2.9 Implement `StatefulWidget` with `AgendaStyle` (day headers, time slots, priority colors, today highlight, selection highlight)
- [x] 2.10 Write unit tests: day/week/month navigation, filter logic, item ordering
- [x] 2.11 Add rat-agenda tab to showcase with sample static data source (deferred to group 9)

## 3. rat-datepicker — Date, time, and repeater input

- [x] 3.1 Create `crates/rat-datepicker` crate scaffold: Cargo.toml (depends on ratatui), src/lib.rs
- [x] 3.2 Implement `CalendarGridState`: selected date, displayed month/year, week start day config
- [x] 3.3 Implement `CalendarGrid` StatefulWidget: month header, day-name columns, day cells in correct weekday positions, today/selected/weekend styles
- [x] 3.4 Implement calendar navigation: arrow keys (left/right ±1 day, up/down ±7 days), month boundary transitions
- [x] 3.5 Implement `TimeInputState`: hours field, minutes field, focused field; `TimeInput` widget rendering HH:MM
- [x] 3.6 Implement time input interaction: direct digit typing, colon/tab to switch fields, up/down increment/decrement with wrapping
- [x] 3.7 Implement `RepeaterInputState`: mode (+/++/.+), count, unit (d/w/m/y); `RepeaterInput` widget rendering "+1w" format
- [x] 3.8 Implement repeater interaction: cycle unit, cycle mode, increment/decrement count, enable/disable
- [x] 3.9 Define Action enums and handle_action() for each sub-widget, including Confirmed/Cancelled result variants
- [x] 3.10 Implement `CalendarStyle` with selected/today/weekend/out-of-month/header styles
- [x] 3.11 Write unit tests: month grid layout correctness (Feb leap year, month lengths), navigation across boundaries, time wrapping, repeater formatting
- [x] 3.12 Add rat-datepicker tab to showcase (deferred to group 9)

## 4. rat-capture — Quick capture overlay

- [x] 4.1 Create `crates/rat-capture` crate scaffold: Cargo.toml (depends on rat-editor, rat-widgets, ratatui), src/lib.rs
- [x] 4.2 Implement `CaptureTemplate` struct and `CaptureResult` struct with timestamp
- [x] 4.3 Implement `CaptureState`: phase enum (Closed/TemplateSelect/Editing), template list, selected index, title TextInput, body Editor, result slot
- [x] 4.4 Implement lifecycle: open(templates) → template-select (or skip if single) → editing → confirmed/cancelled → closed
- [x] 4.5 Implement editing phase: tab between title/body focus, Ctrl+Enter to confirm, Enter on non-empty title to confirm, Escape to cancel, empty title rejection
- [x] 4.6 Define `Action` enum and `handle_action()` covering all phase transitions and input delegation
- [x] 4.7 Implement `StatefulWidget` rendering: centered popup (60%×50% configurable), bordered, template list in select phase, title input + body editor in editing phase
- [x] 4.8 Implement `CaptureStyle` with border, title bar, input, template list styles
- [x] 4.9 Write unit tests: lifecycle transitions, empty title rejection, single template skip, take_result after confirm/cancel
- [x] 4.10 Add rat-capture tab to showcase

## 5. rat-fuzzy — Incremental fuzzy finder

- [x] 5.1 Create `crates/rat-fuzzy` crate scaffold: Cargo.toml (depends on ratatui), src/lib.rs
- [x] 5.2 Implement `FuzzyCandidate` struct and `FuzzySource` trait with blanket impl for Vec<FuzzyCandidate>
- [x] 5.3 Implement fuzzy scoring algorithm: consecutive match bonus, word boundary bonus, prefix bonus, gap penalty; score 0 = no match
- [x] 5.4 Implement match position tracking: record which character indices matched so they can be highlighted during rendering
- [x] 5.5 Implement `FuzzyState`: query string, source ref, scored/sorted results, selected index, scroll offset, open/closed flag, result slot
- [x] 5.6 Implement incremental search: on each TypeChar/Backspace, re-score candidates, sort by score desc, reset selection to 0
- [x] 5.7 Implement result navigation: up/down selection, scroll to keep selection visible, Enter to confirm, Escape to clear-or-close
- [x] 5.8 Define `Action` enum and `handle_action()` for all input and navigation
- [x] 5.9 Implement `StatefulWidget` rendering: input line with prompt, result count, scrollable candidate list with match character highlighting
- [x] 5.10 Implement `FuzzyStyle` with prompt, query, match highlight, selected, context, border, count styles
- [x] 5.11 Write unit tests: scoring (prefix beats mid-word, consecutive beats scattered), empty query returns all, match position tracking, navigation edge cases
- [x] 5.12 Add rat-fuzzy tab to showcase

## 6. rat-backlinks — Incoming reference panel

- [x] 6.1 Create `crates/rat-backlinks` crate scaffold: Cargo.toml (depends on ratatui), src/lib.rs
- [x] 6.2 Implement `Backlink` struct and `BacklinkSource` trait
- [x] 6.3 Implement `BacklinksState`: target ID, backlinks list, group-by-file structure, collapse states per group, selected index, scroll offset
- [x] 6.4 Implement grouped display: file headers with link count, collapsible entries under each, navigation skips collapsed groups
- [x] 6.5 Implement context snippet rendering: context_line with link_text highlighted, optional context_before/after in dimmed style
- [x] 6.6 Implement jump action: Enter on entry emits ActionResult::Jump { file, line }
- [x] 6.7 Define `Action` enum and `handle_action()` for navigation, group toggle, target change, refresh, jump
- [x] 6.8 Implement `StatefulWidget` rendering: bordered panel with title "Backlinks: {heading} ({count})", group headers, entries
- [x] 6.9 Implement `BacklinksStyle` with file header, line number, context, link highlight, selected, collapse indicator styles
- [x] 6.10 Write unit tests: grouping logic, collapse/expand, navigation with collapsed groups, SetTarget re-query
- [x] 6.11 Add rat-backlinks tab to showcase

## 7. rat-tags — Tag completion and property editor

- [x] 7.1 Create `crates/rat-tags` crate scaffold: Cargo.toml (depends on rat-widgets, ratatui), src/lib.rs
- [x] 7.2 Implement tag model: Vec<String> with `:tag:` format serialize/parse, validation (alphanumeric + underscore + hyphen)
- [x] 7.3 Implement `TagEditorState`: current tags, input text, vocabulary, popup filtered list, popup selection, popup visible flag
- [x] 7.4 Implement tag completion: type-to-filter vocabulary, popup appears on input, Enter/Tab accepts suggestion, non-vocabulary tags allowed, Backspace-on-empty removes last tag
- [x] 7.5 Implement `PriorityCycler`: configurable cycle list, None→A→B→C→None default, `[#X]` display format
- [x] 7.6 Implement `PropertyEditorState`: Vec<(String,String)>, selected row, edit mode (key/value/none), inner TextInput states
- [x] 7.7 Implement property editor interaction: up/down navigate rows, enter to edit value, add/delete property, tab between key/value
- [x] 7.8 Define Action enums and handle_action() for TagEditor and PropertyEditor
- [x] 7.9 Implement `StatefulWidget` for TagEditor: inline tag chips + input area + completion popup
- [x] 7.10 Implement `StatefulWidget` for PropertyEditor: key-value rows with edit highlight
- [x] 7.11 Implement `TagStyle` and `PropertyStyle` structs
- [x] 7.12 Write unit tests: tag parse/format roundtrip, validation, completion filtering, priority cycling, property add/delete
- [x] 7.13 Add rat-tags tab to showcase

## 8. Org table bridge — rat-spreadsheet adapter

- [x] 8.1 Add `org-compat` feature flag to rat-spreadsheet Cargo.toml, create `src/org_table.rs` module gated behind it
- [x] 8.2 Implement `from_org_table(text: &str) -> Result<Grid, ParseError>`: parse pipe rows, skip separator rows, trim cells, detect numbers
- [x] 8.3 Implement `to_org_table(grid: &Grid) -> String`: serialize to pipe-table with aligned columns, right-align numbers, separator after header row
- [x] 8.4 Implement org formula translator: `$N` → column letter, `@N$M` → cell address, `vsum`/`vmean` → `SUM`/`AVERAGE`, passthrough for A1-style
- [x] 8.5 Write unit tests: round-trip parse→serialize, numeric detection, empty cells, separator handling, formula translation for column refs / cell refs / functions
- [x] 8.6 Add org-table examples to the spreadsheet showcase tab

## 9. Workspace integration and showcase

- [x] 9.1 Add all new crates to workspace Cargo.toml members list
- [x] 9.2 Update showcase Cargo.toml to depend on all new crates
- [x] 9.3 Wire up all new showcase tabs with demo data and keybinding hints
- [x] 9.4 Run `cargo check --workspace` and fix any cross-crate issues
- [x] 9.5 Run `cargo test --workspace` and verify all specs have corresponding test coverage
