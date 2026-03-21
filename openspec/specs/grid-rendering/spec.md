## ADDED Requirements

### Requirement: Grid layout
The system SHALL render the spreadsheet as a grid with column headers (A, B, C, ...) across the top and row numbers (1, 2, 3, ...) along the left side. The intersection of the header row and row-number column SHALL be empty or display a select-all indicator.

#### Scenario: Column headers
- **WHEN** the viewport shows columns A through E
- **THEN** the top row displays "A", "B", "C", "D", "E" in the header cells

#### Scenario: Row numbers
- **WHEN** the viewport shows rows 1 through 10
- **THEN** the leftmost column displays "1" through "10"

### Requirement: Cell rendering
Each cell SHALL render its display value within its allocated rectangle. Numbers SHALL be right-aligned. Text SHALL be left-aligned. Booleans SHALL display as "TRUE" or "FALSE". Errors SHALL display their error code (e.g., "#DIV/0!"). Formula cells SHALL display their cached computed value, not the expression.

#### Scenario: Number alignment
- **WHEN** cell A1 contains the number 42
- **THEN** "42" is rendered right-aligned in the cell

#### Scenario: Formula displays result
- **WHEN** cell B1 contains formula "=1+2" with cached value 3
- **THEN** "3" is rendered in the cell, not "=1+2"

#### Scenario: Error display
- **WHEN** cell C1 has a #DIV/0! error
- **THEN** "#DIV/0!" is rendered in the cell

### Requirement: Cursor indicator
The system SHALL visually distinguish the current cursor cell from other cells using a distinct background color or border style. The cursor style SHALL be configurable.

#### Scenario: Cursor visible
- **WHEN** the cursor is at cell B3
- **THEN** cell B3 is rendered with the cursor style (distinct from normal cells)

### Requirement: Selection highlight
The system SHALL render selected cells with a highlight style distinct from both normal cells and the cursor cell. The selection style SHALL be configurable.

#### Scenario: Range highlight
- **WHEN** cells A1:B2 are selected
- **THEN** all four cells render with the selection highlight style

### Requirement: Edit mode indicator
The system SHALL render the cell differently when in edit mode. The edit buffer text SHALL be displayed in place of the cell value, with a visible text cursor.

#### Scenario: Edit mode display
- **WHEN** cell A1 is in edit mode with buffer "hel" and cursor after "l"
- **THEN** the cell displays "hel" with a cursor indicator after the "l"

### Requirement: Column width configuration
Each column SHALL have a configurable width in character units. The system SHALL provide a default width and allow per-column overrides. Column widths SHALL have a minimum of 3 characters.

#### Scenario: Default width
- **WHEN** no column widths are configured
- **THEN** all columns use the default width (e.g., 10 characters)

#### Scenario: Custom column width
- **WHEN** column B is configured with width 20
- **THEN** column B renders 20 characters wide while other columns use the default

#### Scenario: Minimum width enforced
- **WHEN** column A is set to width 1
- **THEN** column A renders at the minimum width of 3

### Requirement: Frozen panes
The system SHALL support freezing rows and/or columns. Frozen rows stay visible at the top when scrolling vertically. Frozen columns stay visible on the left when scrolling horizontally.

#### Scenario: Frozen header row
- **WHEN** 1 row is frozen and the user scrolls down to row 50
- **THEN** row 1 remains visible at the top, and rows 50+ appear below it

#### Scenario: Frozen columns
- **WHEN** 2 columns are frozen and the user scrolls right to column J
- **THEN** columns A and B remain visible on the left, with column J+ appearing after them

### Requirement: Cell styling
The system SHALL support per-cell style overrides (foreground color, background color, bold, italic). A style callback function SHALL receive the cell address and value, returning an optional style override.

#### Scenario: Conditional styling via callback
- **WHEN** a style callback is registered that makes negative numbers red, and cell A1 contains -5
- **THEN** cell A1 renders with red foreground

#### Scenario: No override uses default
- **WHEN** no style callback is registered or the callback returns None for a cell
- **THEN** the cell uses the default style

### Requirement: StatefulWidget implementation
The spreadsheet SHALL implement ratatui's `StatefulWidget` trait. The widget struct holds configuration (column widths, frozen panes, styles). The state struct holds mutable data (grid, cursor, selection, scroll offset, edit state). `EditState` SHALL be imported from the `edit_state` module, not defined in `render.rs`. `SpreadsheetState` fields SHALL be organized into documented groups (data model vs. visual layout) but the struct SHALL remain a single type.

#### Scenario: Render via StatefulWidget
- **WHEN** the application calls `frame.render_stateful_widget(spreadsheet, area, &mut state)`
- **THEN** the spreadsheet grid is rendered into the given area using the current state

#### Scenario: EditState imported from edit_state module
- **WHEN** `render.rs` references `EditState`
- **THEN** it uses `use crate::edit_state::EditState` rather than defining the type inline
