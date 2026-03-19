## ADDED Requirements

### Requirement: Cell addressing
The system SHALL support A1-style cell addressing where columns are labeled A-Z, AA-AZ, etc. and rows are 1-indexed integers. A `CellAddr` type SHALL represent a (column, row) coordinate.

#### Scenario: Parse A1 address
- **WHEN** the string "B3" is parsed as a cell address
- **THEN** the result is column 1 (0-indexed), row 2 (0-indexed)

#### Scenario: Parse multi-letter column
- **WHEN** the string "AA1" is parsed as a cell address
- **THEN** the result is column 26, row 0

#### Scenario: Invalid address
- **WHEN** the string "123" is parsed as a cell address
- **THEN** the parse returns an error

### Requirement: Cell value types
The system SHALL support the following cell value types: empty, text (String), number (f64), boolean, error (with error kind), and formula (expression string + cached value).

#### Scenario: Empty cell default
- **WHEN** a new grid is created with dimensions 5x5
- **THEN** all 25 cells contain the empty value

#### Scenario: Number cell
- **WHEN** a cell is set to the number 42.5
- **THEN** reading that cell returns `CellValue::Number(42.5)`

#### Scenario: Formula cell stores expression and cache
- **WHEN** a cell is set to formula "=A1+1"
- **THEN** the cell stores both the expression string and a cached computed value

### Requirement: Grid data structure
The system SHALL store cells in a row-major `Vec<Vec<CellValue>>` that grows dynamically. The grid SHALL track its current dimensions (row count, column count).

#### Scenario: Dynamic growth on set
- **WHEN** a value is set at address C10 on a 2x2 grid
- **THEN** the grid expands to at least 3 columns and 10 rows, filling new cells with empty values

#### Scenario: Row and column count
- **WHEN** a grid has 5 columns and 20 rows
- **THEN** `row_count()` returns 20 and `col_count()` returns 5

### Requirement: Cell range
The system SHALL support rectangular cell ranges specified by a start and end `CellAddr`. A `CellRange` SHALL iterate over all cells in the rectangle in row-major order.

#### Scenario: Range iteration
- **WHEN** iterating over range A1:B2
- **THEN** the cells are visited in order: A1, B1, A2, B2

#### Scenario: Single cell range
- **WHEN** a range is created with start and end both equal to C3
- **THEN** iteration yields exactly one cell: C3
