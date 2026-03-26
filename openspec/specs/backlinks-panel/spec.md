## ADDED Requirements

### Requirement: Backlink data type
The system SHALL define a `Backlink` struct containing: `source_file` (String), `source_heading` (Option<String>), `source_line` (usize), `context_before` (String, the line before the link), `context_line` (String, the line containing the link), `context_after` (String, the line after the link), `link_text` (String, the visible link text). These are provided by the caller — the widget does not resolve links.

#### Scenario: Backlink with full context
- **WHEN** a Backlink is created with source_file="notes.org", source_line=42, and 3 context lines
- **THEN** all fields are accessible and the link text is extractable for highlighting

#### Scenario: Backlink at start of file
- **WHEN** source_line is 0 and context_before is empty
- **THEN** the backlink is valid; context_before renders as blank

### Requirement: BacklinkSource trait
The system SHALL define a trait `BacklinkSource` with method `fn backlinks(&self, target_id: &str) -> Vec<Backlink>`. The widget calls this to fetch backlinks for the currently viewed heading/node. The trait SHALL be object-safe.

#### Scenario: Query backlinks
- **WHEN** `backlinks("heading-uuid-123")` is called and 3 references exist
- **THEN** a Vec of 3 Backlink structs is returned

#### Scenario: No backlinks
- **WHEN** `backlinks("orphan-id")` is called for a heading with no incoming links
- **THEN** an empty Vec is returned

### Requirement: Grouped display
The system SHALL group backlinks by `source_file`. Each group SHALL show a file header line (file name, link count) followed by the individual backlink entries. Groups SHALL be collapsible — clicking/pressing Enter on a file header toggles visibility of its entries.

#### Scenario: Two files with backlinks
- **WHEN** backlinks come from "notes.org" (2 links) and "projects.org" (1 link)
- **THEN** the panel shows two group headers, each with their link count, and entries underneath

#### Scenario: Collapse a group
- **WHEN** Enter is pressed on the "notes.org" group header
- **THEN** the 2 entries under "notes.org" are hidden; the header shows a collapsed indicator

### Requirement: Context snippet rendering
Each backlink entry SHALL display the `context_line` with the `link_text` portion highlighted. Optionally (configurable), `context_before` and `context_after` lines SHALL be shown in a dimmed style for surrounding context. The source line number SHALL be shown as a prefix.

#### Scenario: Render with context
- **WHEN** a backlink has context_before="previous line", context_line="see [[Ship docs]] for details", context_after="next line"
- **THEN** the entry renders 3 lines: dimmed before, highlighted link in the middle line, dimmed after

#### Scenario: Render without extra context
- **WHEN** the style is configured with context_lines=0
- **THEN** only the context_line is rendered for each backlink

### Requirement: Navigation and jump
The system SHALL support arrow up/down to move between backlink entries (skipping collapsed groups). Enter on a backlink entry SHALL emit a `Jump` action containing the source file and line number. The caller handles the actual file navigation.

#### Scenario: Navigate entries
- **WHEN** the panel has 5 visible entries and down-arrow is pressed 3 times from the top
- **THEN** the selection is on the 4th entry

#### Scenario: Jump to source
- **WHEN** Enter is pressed on an entry from "notes.org" line 42
- **THEN** `handle_action` returns `ActionResult::Jump { file: "notes.org", line: 42 }`

### Requirement: BacklinksState and Action enum
The system SHALL expose `BacklinksState` containing: target ID, backlinks list, group collapse states, selected index, scroll offset, and source reference. An `Action` enum SHALL include: `SetTarget(String)`, `Refresh`, `SelectNext`, `SelectPrev`, `ToggleGroup`, `Jump`, `ScrollUp`, `ScrollDown`. A `handle_action(state, action)` function SHALL process actions.

#### Scenario: SetTarget re-queries
- **WHEN** `handle_action(state, Action::SetTarget("new-id".into()))` is called
- **THEN** the source is queried with "new-id", the backlinks list updates, and selection resets to 0

### Requirement: Backlinks rendering
The system SHALL implement `StatefulWidget` for ratatui. The widget SHALL render as a bordered panel with a title showing the target heading name and total backlink count. A `BacklinksStyle` struct SHALL configure: file header style, line number style, context dimmed style, link highlight style, selected entry style, and collapsed/expanded indicators.

#### Scenario: Panel title
- **WHEN** the target heading is "Ship docs" with 5 backlinks
- **THEN** the panel title reads "Backlinks: Ship docs (5)"
