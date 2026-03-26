## ADDED Requirements

### Requirement: Parse org pipe-table to Grid
The system SHALL provide a function `from_org_table(text: &str) -> Result<Grid, ParseError>` that parses org-mode pipe-table syntax into a `rat_spreadsheet::Grid`. Data rows (`| a | b |`) SHALL map to grid cells. Separator rows (`|---+---|`) SHALL be skipped (not stored as data). Leading and trailing whitespace in cells SHALL be trimmed. Empty cells SHALL become `CellValue::Empty`.

#### Scenario: Simple table
- **WHEN** the input is `"| Name | Age |\n|------+-----|\n| Alice | 30 |\n| Bob | 25 |"`
- **THEN** the grid has 2 columns and 3 rows (header + 2 data rows), with cells ["Name","Age"], ["Alice","30"], ["Bob","25"]

#### Scenario: Numeric detection
- **WHEN** a cell contains "42" or "3.14"
- **THEN** the cell value is `CellValue::Number(42.0)` or `CellValue::Number(3.14)`

#### Scenario: Empty cells
- **WHEN** a cell is `|  |` or `||`
- **THEN** the cell value is `CellValue::Empty`

#### Scenario: No separator row
- **WHEN** the table has no `|---` separator rows
- **THEN** all rows are parsed as data rows (no error)

#### Scenario: Invalid input
- **WHEN** the input has lines not starting with `|`
- **THEN** those lines are ignored and the valid pipe-table rows are parsed

### Requirement: Serialize Grid to org pipe-table
The system SHALL provide a function `to_org_table(grid: &Grid) -> String` that serializes a grid back to org pipe-table format. Columns SHALL be padded to equal width within each column. Numbers SHALL be right-aligned within their cell padding. Text SHALL be left-aligned. A separator row SHALL be inserted after the first row (treated as header).

#### Scenario: Round-trip simple table
- **WHEN** a table is parsed with `from_org_table` and then serialized with `to_org_table`
- **THEN** the output is a valid pipe-table with consistent column widths

#### Scenario: Column alignment
- **WHEN** a grid has column 0 = ["Name", "Alice", "Bob"] and column 1 = [10, 3, 150]
- **THEN** column 0 cells are left-aligned and column 1 cells are right-aligned within their padding

#### Scenario: Empty grid
- **WHEN** the grid has 0 rows
- **THEN** `to_org_table` returns an empty string

### Requirement: Org field-formula translation
The system SHALL provide functions to translate between org-style column formulas and the existing `rat_spreadsheet` formula syntax. Org uses `$N` for column references (1-indexed) and `@N` for row references. The translator SHALL map: `$1` → `A` (column A), `$2` → `B`, `@2$3` → `C2`, and common functions like `vmean`, `vsum` to their spreadsheet equivalents `AVERAGE`, `SUM`.

#### Scenario: Column reference
- **WHEN** the org formula is "$1 + $2"
- **THEN** the translated formula is "A{row} + B{row}" where {row} is the current row context

#### Scenario: Cell reference
- **WHEN** the org formula is "@2$3"
- **THEN** the translated formula is "C2"

#### Scenario: Function translation
- **WHEN** the org formula is "vsum($1..$3)"
- **THEN** the translated formula maps to a SUM over columns A through C in the current row

#### Scenario: Passthrough for standard formulas
- **WHEN** the formula is already in A1-style (e.g., "=A1+B1")
- **THEN** it is returned unchanged

### Requirement: Feature-gated module
The org table bridge SHALL be in a module `rat_spreadsheet::org_table` gated behind an `org-compat` feature flag. When the feature is not enabled, the module is not compiled. The feature SHALL not add any new external dependencies.

#### Scenario: Feature disabled
- **WHEN** `rat-spreadsheet` is compiled without the `org-compat` feature
- **THEN** `rat_spreadsheet::org_table` does not exist and no additional code is compiled

#### Scenario: Feature enabled
- **WHEN** `rat-spreadsheet` is compiled with `features = ["org-compat"]`
- **THEN** `from_org_table`, `to_org_table`, and formula translation functions are available
