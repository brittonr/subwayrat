## ADDED Requirements

### Requirement: Windows identified by opaque ID
Each window in the strip SHALL have a unique opaque identifier assigned at insertion time. All operations (remove, resize, query) SHALL reference windows by this ID. IDs SHALL NOT be reused after removal.

#### Scenario: Insert and query by ID
- **WHEN** a window is inserted and returns ID 7
- **THEN** querying the strip for ID 7 returns that window's column index and position within the column

#### Scenario: ID not reused after removal
- **WHEN** window with ID 3 is removed and a new window is inserted
- **THEN** the new window receives an ID different from 3

### Requirement: Insert window into column
The consumer SHALL be able to insert a window into a specific column at a specific stack position. If the column index is beyond the current column count, new empty columns SHALL be created. If the stack position is beyond the current window count in the column, the window SHALL be appended.

#### Scenario: Insert into existing column
- **WHEN** column 1 has windows [A, B] and a new window C is inserted at stack position 1
- **THEN** column 1 contains [A, C, B]

#### Scenario: Insert creates new column
- **WHEN** the strip has 2 columns and a window is inserted into column 5
- **THEN** columns 2, 3, 4 are created empty, and column 5 contains the new window

#### Scenario: Append to end of column
- **WHEN** column 0 has 2 windows and a window is inserted at stack position 99
- **THEN** the window is appended as the third window in column 0

### Requirement: Remove window
The consumer SHALL be able to remove a window by ID. If removal leaves a column empty, that column SHALL be removed from the strip. The layout SHALL be recomputed on next call to compute_layout.

#### Scenario: Remove from multi-window column
- **WHEN** column 0 has [A, B, C] and window B is removed
- **THEN** column 0 contains [A, C]

#### Scenario: Remove last window in column
- **WHEN** column 2 has only window D and D is removed
- **THEN** column 2 is removed, columns after it shift down by one index

#### Scenario: Remove non-existent ID
- **WHEN** remove is called with an ID that doesn't exist
- **THEN** the operation returns an error or is a no-op

### Requirement: Move window between columns
The consumer SHALL be able to move a window from its current position to a different column and stack position. This SHALL be atomic — the window is removed from its source and inserted at the destination in one operation.

#### Scenario: Move window to different column
- **WHEN** window A is in column 0 and is moved to column 2 at stack position 0
- **THEN** window A is removed from column 0 and appears at the top of column 2

#### Scenario: Move within same column
- **WHEN** column 0 has [A, B, C] and B is moved to stack position 0
- **THEN** column 0 contains [B, A, C]

### Requirement: Resize window constraints
The consumer SHALL be able to change a window's size constraints after insertion. The new constraints SHALL take effect on the next compute_layout call.

#### Scenario: Change from fixed to proportional
- **WHEN** a window has Fixed(20) width and is resized to Proportion(1.0)
- **THEN** the next layout computation uses the proportional constraint

#### Scenario: Change column width constraint
- **WHEN** a column's width constraint is changed from Fixed(30) to Fixed(50)
- **THEN** subsequent layouts allocate 50 cells for that column

### Requirement: Insert new column
The consumer SHALL be able to insert an empty column at a specific index, optionally with an initial window. Existing columns at and after that index SHALL shift right.

#### Scenario: Insert column in the middle
- **WHEN** the strip has columns [A, B, C] and a new column is inserted at index 1
- **THEN** the strip becomes [A, new, B, C]
