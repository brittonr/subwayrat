## MODIFIED Requirements

### Requirement: Enter edit mode
The system SHALL enter edit mode on the current cell when the user presses Enter or starts typing. In edit mode, an inline text input appears in the cell, pre-filled with the cell's current display value (or empty for empty cells). For formula cells, the edit buffer SHALL show the formula expression (e.g., "=A1+1"), not the computed value. `EditState` SHALL be defined in a dedicated `edit_state` module, not in `render.rs`.

#### Scenario: Enter key activates editing
- **WHEN** the cursor is on cell B2 containing "hello" and the user presses Enter
- **THEN** the cell enters edit mode with the buffer containing "hello"

#### Scenario: Typing activates editing
- **WHEN** the cursor is on cell A1 (empty) and the user types "x"
- **THEN** the cell enters edit mode with the buffer containing "x"

#### Scenario: Formula cell shows expression
- **WHEN** the cursor is on a formula cell showing "42" (formula "=6*7") and the user presses Enter
- **THEN** the edit buffer contains "=6*7"

#### Scenario: EditState defined in edit_state module
- **WHEN** the source tree is inspected
- **THEN** `EditState` struct and its methods are defined in `crates/rat-spreadsheet/src/edit_state.rs`
