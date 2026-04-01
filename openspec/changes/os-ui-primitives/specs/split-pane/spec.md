## ADDED Requirements

### Requirement: Split pane divides area into two child regions
The split pane SHALL divide a given Rect into two non-overlapping child regions separated by a 1-cell-wide divider. The orientation (horizontal or vertical) determines the split axis. Horizontal splits produce left and right children. Vertical splits produce top and bottom children.

#### Scenario: Horizontal split
- **WHEN** the area is 80x24 and orientation is horizontal with divider at ratio 0.5
- **THEN** the first child is Rect(0, 0, 39, 24), the divider occupies column 39, and the second child is Rect(40, 0, 40, 24)

#### Scenario: Vertical split
- **WHEN** the area is 80x24 and orientation is vertical with divider at ratio 0.5
- **THEN** the first child is Rect(0, 0, 80, 11), the divider occupies row 11, and the second child is Rect(0, 12, 80, 12)

### Requirement: Divider position as ratio
When `DividerPos::Ratio(f)` is set, the divider position SHALL be computed as `f * available_space` (available space = total extent minus 1 for the divider itself). The ratio SHALL be clamped to 0.0..=1.0.

#### Scenario: Ratio 0.5
- **WHEN** available width is 79 (80 minus 1 for divider) and ratio is 0.5
- **THEN** the first child is approximately 39 cells wide

#### Scenario: Ratio 0.0
- **WHEN** ratio is 0.0
- **THEN** the first child has width 0 (or min_size if configured) and the second child gets all available space

### Requirement: Divider position as fixed offset
When `DividerPos::FromStart(n)` is set, the first child SHALL be exactly `n` cells along the split axis. When `DividerPos::FromEnd(n)` is set, the second child SHALL be exactly `n` cells along the split axis.

#### Scenario: Fixed from start
- **WHEN** orientation is horizontal and divider is FromStart(20) and area width is 80
- **THEN** the first child is 20 cells wide, divider at column 20, second child is 59 cells wide

#### Scenario: Fixed from end
- **WHEN** orientation is horizontal and divider is FromEnd(15) and area width is 80
- **THEN** the second child is 15 cells wide, divider at column 64, first child is 64 cells wide

### Requirement: Minimum size constraints
Each child region SHALL have a configurable minimum size (default: 1). If the divider position would make either child smaller than its minimum, the divider SHALL be clamped to respect the constraint.

#### Scenario: Divider clamped by first child minimum
- **WHEN** min_first is 10 and divider is FromStart(5) and area width is 80
- **THEN** the first child is 10 cells wide (clamped from 5)

#### Scenario: Divider clamped by second child minimum
- **WHEN** min_second is 20 and divider is FromEnd(10) and area width is 80
- **THEN** the second child is 20 cells wide (clamped from 10)

#### Scenario: Both minimums conflict
- **WHEN** min_first is 40, min_second is 50, and area width is 80 (only 79 available after divider)
- **THEN** the first child gets min_first=40 and the second child gets the remaining 39 (first child's minimum takes precedence)

### Requirement: Divider renders as a visible line
The divider SHALL render as a line character (vertical line for horizontal splits, horizontal line for vertical splits) spanning the full cross-axis extent. The divider style SHALL be configurable.

#### Scenario: Horizontal split divider
- **WHEN** orientation is horizontal
- **THEN** the divider is a vertical line character spanning the full height of the area

#### Scenario: Vertical split divider
- **WHEN** orientation is vertical
- **THEN** the divider is a horizontal line character spanning the full width of the area

### Requirement: Split pane model tracks divider position
The `SplitPaneModel` SHALL store the current `DividerPos`, orientation, and minimum sizes. It SHALL provide methods to move the divider by a delta (positive = toward second child, negative = toward first child), respecting minimum constraints.

#### Scenario: Move divider right
- **WHEN** divider is at ratio 0.5 and move_divider(+5) is called on horizontal orientation with area width 80
- **THEN** the ratio updates to approximately 0.5 + 5/79 ≈ 0.563

#### Scenario: Move clamped by minimum
- **WHEN** divider is at FromStart(12), min_first is 10, and move_divider(-5) is called
- **THEN** the divider moves to FromStart(10) (clamped, not 7)

### Requirement: Split pane render returns child regions
The split pane render function SHALL return a struct containing the two child `Rect` values so the caller can render content into each region.

#### Scenario: Caller receives regions
- **WHEN** the split pane is rendered with area 80x24
- **THEN** the return value contains `first: Rect` and `second: Rect` that together with the divider cover the full area
