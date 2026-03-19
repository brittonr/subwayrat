## ADDED Requirements

### Requirement: Arrow key navigation
The system SHALL move the cell cursor one cell in the pressed direction (Up, Down, Left, Right) when not in edit mode. The cursor SHALL NOT move beyond grid boundaries.

#### Scenario: Move right
- **WHEN** the cursor is at B2 and the user presses Right
- **THEN** the cursor moves to C2

#### Scenario: Boundary clamp
- **WHEN** the cursor is at A1 and the user presses Left
- **THEN** the cursor stays at A1

### Requirement: Tab navigation
The system SHALL move the cursor one cell to the right on Tab and one cell to the left on Shift+Tab. At the end of a row, Tab SHALL wrap to the first column of the next row. At the start of a row, Shift+Tab SHALL wrap to the last column of the previous row.

#### Scenario: Tab wraps to next row
- **WHEN** the grid has 3 columns, the cursor is at C1, and the user presses Tab
- **THEN** the cursor moves to A2

#### Scenario: Shift+Tab wraps to previous row
- **WHEN** the cursor is at A2 and the user presses Shift+Tab
- **THEN** the cursor moves to C1 (last column of previous row)

### Requirement: Jump navigation
The system SHALL support Home (first column in row), End (last column in row), Ctrl+Home (cell A1), and Ctrl+End (last cell with content).

#### Scenario: Home key
- **WHEN** the cursor is at D5 and the user presses Home
- **THEN** the cursor moves to A5

#### Scenario: Ctrl+End
- **WHEN** the farthest-right, lowest cell with content is E10 and the user presses Ctrl+End
- **THEN** the cursor moves to E10

### Requirement: Page navigation
The system SHALL move the cursor by one viewport height on PageUp/PageDown. The viewport height is the number of visible rows.

#### Scenario: PageDown
- **WHEN** the viewport shows 20 rows, the cursor is at A1, and the user presses PageDown
- **THEN** the cursor moves to A21

#### Scenario: PageDown clamps to last row
- **WHEN** the grid has 25 rows, the viewport shows 20 rows, the cursor is at A10, and the user presses PageDown
- **THEN** the cursor moves to A25 (clamped to grid boundary)

### Requirement: Scroll follows cursor
The system SHALL scroll the viewport to keep the cursor visible. When the cursor moves beyond the visible area, the viewport SHALL scroll the minimum amount needed to bring the cursor cell fully into view.

#### Scenario: Cursor moves below viewport
- **WHEN** the viewport shows rows 1-20 and the cursor moves to row 21
- **THEN** the viewport scrolls down so row 21 is visible

#### Scenario: Cursor within viewport
- **WHEN** the cursor moves from B5 to B6 and both are visible
- **THEN** no scrolling occurs

### Requirement: Mouse click navigation
The system SHALL move the cursor to the clicked cell when the user left-clicks on a cell within the grid area. Clicking on column or row headers SHALL NOT move the cursor.

#### Scenario: Click on cell
- **WHEN** the user clicks on the area corresponding to cell C4
- **THEN** the cursor moves to C4

#### Scenario: Click on header ignored
- **WHEN** the user clicks on the column header area
- **THEN** the cursor position does not change

### Requirement: Cell range selection
The system SHALL support rectangular range selection via Shift+arrow keys or Shift+click. The selection anchor is the cell where selection started. The selection extent is the current cursor position. The selected range is the rectangle defined by anchor and extent.

#### Scenario: Shift+Right extends selection
- **WHEN** the cursor is at B2 with no selection and the user presses Shift+Right
- **THEN** the selection covers B2:C2

#### Scenario: Shift+click selects range
- **WHEN** the cursor is at A1 and the user Shift+clicks on C3
- **THEN** the selection covers A1:C3

#### Scenario: Arrow without Shift clears selection
- **WHEN** a range B2:D4 is selected and the user presses Right (without Shift)
- **THEN** the selection is cleared and the cursor moves one cell right from its current position
