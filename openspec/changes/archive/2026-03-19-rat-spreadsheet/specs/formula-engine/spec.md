## ADDED Requirements

### Requirement: Formula detection
The system SHALL treat any cell value starting with "=" as a formula expression. The expression text after "=" is parsed and evaluated.

#### Scenario: Equals prefix triggers formula
- **WHEN** the user commits the value "=1+2"
- **THEN** the cell is stored as a formula with expression "1+2" and cached value 3.0

#### Scenario: Text starting with equals-space is not a formula
- **WHEN** the user commits the value "= note"
- **THEN** the cell is stored as text "= note"

### Requirement: Arithmetic operators
The formula engine SHALL support `+`, `-`, `*`, `/`, and `%` (modulo) operators with standard numeric precedence (multiplicative before additive). Parentheses SHALL override precedence.

#### Scenario: Operator precedence
- **WHEN** the formula "=2+3*4" is evaluated
- **THEN** the result is 14.0

#### Scenario: Parentheses override
- **WHEN** the formula "=(2+3)*4" is evaluated
- **THEN** the result is 20.0

#### Scenario: Division by zero
- **WHEN** the formula "=1/0" is evaluated
- **THEN** the result is a `#DIV/0!` error

### Requirement: Cell references
The formula engine SHALL resolve A1-style cell references to their current values. References to empty cells SHALL evaluate to 0.0 in numeric context.

#### Scenario: Simple reference
- **WHEN** cell A1 contains 10, and cell B1 contains formula "=A1+5"
- **THEN** B1 evaluates to 15.0

#### Scenario: Empty cell reference
- **WHEN** cell A1 is empty and cell B1 contains formula "=A1+1"
- **THEN** B1 evaluates to 1.0

#### Scenario: Reference to text cell
- **WHEN** cell A1 contains text "hello" and cell B1 contains formula "=A1+1"
- **THEN** B1 evaluates to a `#VALUE!` error

### Requirement: Range references in functions
The formula engine SHALL support range references (e.g., A1:B3) as arguments to aggregate functions. Range references SHALL NOT be valid outside of function arguments.

#### Scenario: Range in SUM
- **WHEN** A1=1, A2=2, A3=3, and cell B1 contains formula "=SUM(A1:A3)"
- **THEN** B1 evaluates to 6.0

#### Scenario: Range outside function
- **WHEN** a formula is "=A1:A3+1"
- **THEN** the result is a `#VALUE!` error

### Requirement: Built-in functions
The formula engine SHALL provide: SUM (sum of range), AVG (arithmetic mean), MIN (minimum), MAX (maximum), COUNT (count of non-empty cells), and IF (conditional: IF(condition, true_val, false_val)).

#### Scenario: AVG function
- **WHEN** A1=10, A2=20, A3=30, and cell B1 contains "=AVG(A1:A3)"
- **THEN** B1 evaluates to 20.0

#### Scenario: COUNT skips empty
- **WHEN** A1=1, A2 is empty, A3=3, and cell B1 contains "=COUNT(A1:A3)"
- **THEN** B1 evaluates to 2.0

#### Scenario: IF true branch
- **WHEN** A1=10 and cell B1 contains "=IF(A1>5, 1, 0)"
- **THEN** B1 evaluates to 1.0

#### Scenario: IF false branch
- **WHEN** A1=3 and cell B1 contains "=IF(A1>5, 1, 0)"
- **THEN** B1 evaluates to 0.0

### Requirement: Custom function registration
The system SHALL allow users to register custom functions by name. A custom function receives a slice of evaluated arguments and returns a `CellValue` or error.

#### Scenario: Register and use custom function
- **WHEN** a function "DOUBLE" is registered that multiplies its argument by 2, and a cell contains "=DOUBLE(21)"
- **THEN** the cell evaluates to 42.0

### Requirement: Dependency tracking
The formula engine SHALL maintain a dependency graph of cell references. When a cell's value changes, all cells that reference it (directly or transitively) SHALL be marked for recalculation.

#### Scenario: Transitive dependency
- **WHEN** A1=1, B1="=A1+1", C1="=B1+1", and A1 is changed to 10
- **THEN** B1 recalculates to 11 and C1 recalculates to 12

#### Scenario: Unrelated cells not recalculated
- **WHEN** A1=1, B1="=A1", C1=99, and A1 changes to 2
- **THEN** B1 recalculates to 2 and C1 is NOT recalculated

### Requirement: Circular reference detection
The formula engine SHALL detect circular references and set all cells in the cycle to a `#CYCLE!` error value.

#### Scenario: Direct cycle
- **WHEN** A1 contains "=B1" and B1 contains "=A1"
- **THEN** both A1 and B1 display `#CYCLE!`

#### Scenario: Indirect cycle
- **WHEN** A1="=B1", B1="=C1", C1="=A1"
- **THEN** all three cells display `#CYCLE!`
