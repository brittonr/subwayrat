## ADDED Requirements

### Requirement: Columns arranged along primary axis
The layout engine SHALL arrange columns sequentially along a configurable primary axis (horizontal or vertical). Each column SHALL occupy a contiguous region along the primary axis. Columns SHALL be separated by a configurable gap in cells.

#### Scenario: Horizontal strip with three columns
- **WHEN** a strip has three columns with widths 20, 30, 20 and gap 1 in horizontal mode
- **THEN** column 0 occupies x=[0,19], column 1 occupies x=[21,50], column 2 occupies x=[52,71]

#### Scenario: Vertical strip with two columns
- **WHEN** a strip has two columns with heights 10, 15 and gap 1 in vertical mode
- **THEN** column 0 occupies y=[0,9], column 1 occupies y=[11,25]

#### Scenario: Zero gap
- **WHEN** gap is 0 and two columns have widths 10 and 20
- **THEN** column 0 occupies x=[0,9], column 1 occupies x=[10,29]

### Requirement: Windows stacked within columns
Windows within a column SHALL be stacked along the cross axis (vertical when the primary axis is horizontal, horizontal when vertical). Each window SHALL occupy the full column width along the primary axis. Windows SHALL be separated by a configurable cross-axis gap.

#### Scenario: Two windows stacked in a horizontal-mode column
- **WHEN** a column in horizontal mode contains windows A and B, viewport height is 20, both have equal proportion constraints, and cross gap is 0
- **THEN** window A occupies the top 10 rows and window B occupies the bottom 10 rows, both spanning the full column width

#### Scenario: Single window fills column
- **WHEN** a column contains one window with no explicit height constraint
- **THEN** the window occupies the full column height (viewport height minus any cross-axis gaps)

### Requirement: Fixed size constraints
A window or column SHALL accept a Fixed(u16) size constraint that sets its extent along the relevant axis to exactly that many cells.

#### Scenario: Fixed-width column
- **WHEN** a column has constraint Fixed(25) in a horizontal strip
- **THEN** the column is exactly 25 cells wide regardless of viewport size

#### Scenario: Fixed-height window
- **WHEN** a window has height constraint Fixed(8) in a column of height 20
- **THEN** the window is exactly 8 cells tall

### Requirement: Proportional size constraints
A window or column SHALL accept a Proportion(f32) constraint that allocates a fraction of the remaining space after fixed-size items are placed. Proportions SHALL be normalized across siblings.

#### Scenario: Two proportional columns
- **WHEN** two columns have Proportion(1.0) and Proportion(2.0) in a viewport of width 90 with gap 0
- **THEN** column 0 gets 30 cells and column 1 gets 60 cells

#### Scenario: Mix of fixed and proportional
- **WHEN** column A is Fixed(20), column B is Proportion(1.0), viewport width is 81, gap is 1
- **THEN** column A gets 20 cells, column B gets 60 cells (81 - 20 - 1 gap = 60)

### Requirement: Min and MinMax size constraints
A window or column SHALL accept Min(u16) and MinMax(u16, u16) constraints. Min sets a lower bound, allowing growth beyond it. MinMax clamps the computed size to the given range.

#### Scenario: Min constraint with excess space
- **WHEN** a column has Min(10) and is the only column in a 40-wide viewport
- **THEN** the column expands to fill available space (40 cells)

#### Scenario: MinMax clamps proportional
- **WHEN** a column has MinMax(10, 30) and Proportion would assign 50
- **THEN** the column gets 30 cells

### Requirement: Layout output as Rects
The layout engine SHALL produce a `Rect` (x, y, width, height in u16 cells) for every window in the strip. Rects SHALL be in strip-space coordinates (not viewport-space). Rects SHALL not overlap.

#### Scenario: Non-overlapping rects
- **WHEN** compute_layout is called on any valid strip configuration
- **THEN** no two window Rects overlap

#### Scenario: Rect coordinates are strip-space
- **WHEN** a strip has total content width 200 and viewport width 80
- **THEN** window Rects MAY have x values from 0 to 199, independent of the viewport scroll position
