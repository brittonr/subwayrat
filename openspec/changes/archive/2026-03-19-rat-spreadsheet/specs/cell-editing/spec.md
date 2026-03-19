## ADDED Requirements

### Requirement: Enter edit mode
The system SHALL enter edit mode on the current cell when the user presses Enter or starts typing. In edit mode, an inline text input appears in the cell, pre-filled with the cell's current display value (or empty for empty cells). For formula cells, the edit buffer SHALL show the formula expression (e.g., "=A1+1"), not the computed value.

#### Scenario: Enter key activates editing
- **WHEN** the cursor is on cell B2 containing "hello" and the user presses Enter
- **THEN** the cell enters edit mode with the buffer containing "hello"

#### Scenario: Typing activates editing
- **WHEN** the cursor is on cell A1 (empty) and the user types "x"
- **THEN** the cell enters edit mode with the buffer containing "x"

#### Scenario: Formula cell shows expression
- **WHEN** the cursor is on a formula cell showing "42" (formula "=6*7") and the user presses Enter
- **THEN** the edit buffer contains "=6*7"

### Requirement: Commit edit
The system SHALL commit the edit buffer to the cell when the user presses Enter while in edit mode. If the buffer starts with "=", it SHALL be stored as a formula. Otherwise, the system SHALL attempt numeric parsing first, then fall back to text.

#### Scenario: Commit text
- **WHEN** the user types "hello" and presses Enter
- **THEN** the cell value becomes `CellValue::Text("hello")`

#### Scenario: Commit number
- **WHEN** the user types "3.14" and presses Enter
- **THEN** the cell value becomes `CellValue::Number(3.14)`

#### Scenario: Commit formula
- **WHEN** the user types "=A1+B1" and presses Enter
- **THEN** the cell value becomes a formula with expression "=A1+B1" and the computed result is cached

### Requirement: Cancel edit
The system SHALL discard the edit buffer and restore the previous cell value when the user presses Escape while in edit mode.

#### Scenario: Escape discards changes
- **WHEN** cell A1 contains "old" and the user enters edit mode, types "new", and presses Escape
- **THEN** cell A1 still contains "old" and edit mode is exited

### Requirement: Cell validation
The system SHALL support pluggable validation functions per column. When a validation function is registered for a column, committing an edit SHALL run the validator. If validation fails, the edit is rejected and the cell remains in edit mode with an error indicator.

#### Scenario: Numeric-only column rejects text
- **WHEN** column B has a numeric-only validator and the user tries to commit "abc" in B3
- **THEN** the commit is rejected, the cell stays in edit mode, and an error is indicated

#### Scenario: Valid input passes
- **WHEN** column B has a numeric-only validator and the user commits "42" in B3
- **THEN** the value is accepted and the cell exits edit mode

### Requirement: Single-level undo
The system SHALL store the previous value of a cell before each edit commit. Pressing Ctrl+Z while not in edit mode SHALL restore the last edited cell to its previous value.

#### Scenario: Undo restores previous value
- **WHEN** cell A1 was "old", the user edits it to "new", then presses Ctrl+Z
- **THEN** cell A1 is restored to "old"

#### Scenario: Undo only applies once
- **WHEN** the user presses Ctrl+Z a second time without any intervening edit
- **THEN** nothing changes (single-level undo, not a stack)
