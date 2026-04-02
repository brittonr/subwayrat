## ADDED Requirements

### Requirement: Inline frame buffer
The system SHALL maintain a ratatui `Buffer` representing the current inline content. The buffer width SHALL match the terminal width. The buffer height SHALL equal the sum of all rendered node heights.

#### Scenario: Initial render
- **WHEN** `InlineRenderer::new(width)` is called
- **THEN** the renderer creates an empty buffer with the given width and zero height

#### Scenario: Content determines buffer height
- **WHEN** nodes are added via `rebuild(view_tree)` and `render()` is called
- **THEN** the buffer height equals the sum of each node's measured height at the current width

### Requirement: Frame diffing
The system SHALL compare the current frame buffer against the previous frame buffer cell-by-cell. Only cells whose content or style changed SHALL produce ANSI escape output. Unchanged cells SHALL produce no output.

#### Scenario: No changes between frames
- **WHEN** `render()` is called twice with identical state
- **THEN** the second call produces an empty output byte sequence

#### Scenario: Single cell change
- **WHEN** one cell changes content between frames
- **THEN** output contains a cursor-move sequence and the new cell content, with no other cell output

### Requirement: Terminal growth
The system SHALL emit newline characters to claim additional terminal rows when the content height increases. The renderer SHALL track the number of claimed rows and only emit newlines for the difference.

#### Scenario: Content grows by N rows
- **WHEN** a rebuild increases total content height by N rows
- **THEN** the renderer emits exactly N newline characters before rendering the frame

#### Scenario: Content shrinks
- **WHEN** a rebuild decreases total content height
- **THEN** the renderer does NOT emit newlines and clears the excess rows

### Requirement: Synchronized output wrapping
The system SHALL wrap frame output in DEC synchronized output sequences (`\x1b[?2026h` before, `\x1b[?2026l` after) to prevent tearing on terminals that support the protocol.

#### Scenario: Frame output wrapping
- **WHEN** a frame is rendered with changes
- **THEN** the output byte sequence starts with `\x1b[?2026h` and ends with `\x1b[?2026l`

#### Scenario: Empty frame
- **WHEN** a frame is rendered with no changes
- **THEN** no synchronized output sequences are emitted

### Requirement: Scrollback commit callback
The system SHALL accept an optional `on_commit` callback. When content rows scroll above the terminal viewport, the system SHALL invoke `ratcore::inline::compute_commits` and fire the callback with the key of each fully evicted node.

#### Scenario: Node scrolls off screen
- **WHEN** terminal height is 24 rows and content grows to 30 rows, evicting a keyed node
- **THEN** `on_commit` fires with that node's key

#### Scenario: No callback registered
- **WHEN** content scrolls off screen with no `on_commit` set
- **THEN** no error occurs; evicted rows are simply no longer diffed

### Requirement: Width tracking on resize
The system SHALL query terminal width on each render cycle. When the width changes, the system SHALL re-measure all node heights and rebuild the frame buffer at the new width.

#### Scenario: Terminal resize
- **WHEN** terminal width changes between render calls
- **THEN** all nodes are re-measured at the new width and the buffer is reconstructed
