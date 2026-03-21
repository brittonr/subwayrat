## ADDED Requirements

### Requirement: Focus tracks a single window
The strip SHALL maintain a focus state identifying at most one window by ID. Setting focus to a window SHALL update the viewport scroll position via the focus-driven scroll mechanism. Focus MAY be None (no window focused).

#### Scenario: Set focus by ID
- **WHEN** focus is set to window ID 5
- **THEN** the focus state reports window 5 as focused and the viewport adjusts

#### Scenario: Clear focus
- **WHEN** focus is set to None
- **THEN** no window is focused and the viewport offset does not change

#### Scenario: Focus on removed window
- **WHEN** the focused window is removed from the strip
- **THEN** focus moves to the nearest neighbor (same column preferred, then adjacent column) or becomes None if the strip is empty

### Requirement: Move focus left/right across columns
The consumer SHALL be able to move focus to the adjacent column in either direction along the primary axis. Focus SHALL land on the window in the target column whose cross-axis position is closest to the current focused window's position (positional affinity).

#### Scenario: Move focus right
- **WHEN** focus is on window A in column 0 and move-right is invoked
- **THEN** focus moves to the window in column 1 whose vertical center is closest to A's vertical center

#### Scenario: Move focus left at first column
- **WHEN** focus is on a window in column 0 and move-left is invoked
- **THEN** focus does not change (no column to the left)

#### Scenario: Move right into multi-window column
- **WHEN** column 1 has windows [B, C, D] stacked vertically and focus moves right from column 0 where the focused window was vertically centered at row 15
- **THEN** focus lands on whichever of B, C, D has its vertical center closest to row 15

### Requirement: Move focus up/down within column
The consumer SHALL be able to move focus to the adjacent window within the same column along the cross axis. If focus is at the top or bottom of the column, the operation SHALL be a no-op.

#### Scenario: Move focus down
- **WHEN** column has [A, B, C] and A is focused, move-down is invoked
- **THEN** focus moves to B

#### Scenario: Move focus down at bottom
- **WHEN** column has [A, B, C] and C is focused, move-down is invoked
- **THEN** focus remains on C

#### Scenario: Move focus up
- **WHEN** column has [A, B, C] and B is focused, move-up is invoked
- **THEN** focus moves to A

### Requirement: Focus first and last
The consumer SHALL be able to jump focus to the first window in the first column or the last window in the last column.

#### Scenario: Focus first
- **WHEN** focus-first is invoked on a strip with columns [col0, col1, col2]
- **THEN** focus moves to the first window in col0

#### Scenario: Focus last
- **WHEN** focus-last is invoked on a strip with columns [col0, col1, col2]
- **THEN** focus moves to the last window in col2

#### Scenario: Focus first on empty strip
- **WHEN** focus-first is invoked on an empty strip
- **THEN** focus is None

### Requirement: Focus preserves cross-axis affinity
When moving focus left or right across columns, the layout engine SHALL remember the cross-axis position of the window where horizontal movement began. Subsequent left/right moves SHALL use this stored affinity rather than the current window's position. Moving up/down SHALL reset the affinity.

#### Scenario: Affinity preserved across columns
- **WHEN** focus is on a window centered at row 30, moves right to a column where the closest window is at row 10, then moves right again
- **THEN** the second right-move targets row 30 (the original affinity), not row 10

#### Scenario: Vertical move resets affinity
- **WHEN** focus moves right (storing affinity at row 30), then moves down to a window at row 50
- **THEN** the next right-move targets row 50 (affinity was reset by the vertical move)
