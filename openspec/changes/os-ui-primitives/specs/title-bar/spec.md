## ADDED Requirements

### Requirement: Title bar renders label text
The title bar SHALL render a single-line text label within the given Rect. The label SHALL be truncated with an ellipsis character if it exceeds the available width after accounting for indicators.

#### Scenario: Label fits
- **WHEN** the title bar has label "My Window" and the Rect is 40 columns wide with no indicators
- **THEN** the full label "My Window" is rendered

#### Scenario: Label truncated
- **WHEN** the title bar has label "A Very Long Window Title That Exceeds Bounds" and the Rect is 20 columns wide
- **THEN** the label is truncated to fit with a trailing ellipsis character

### Requirement: Title bar label alignment
The title bar SHALL support left, center, and right alignment for the label text. The alignment SHALL be relative to the space remaining after indicators are placed.

#### Scenario: Center alignment
- **WHEN** the title bar has label "Title" with center alignment and the Rect is 40 columns wide with no indicators
- **THEN** the label is horizontally centered within the Rect

#### Scenario: Center alignment with indicators
- **WHEN** the title bar has label "Title" with center alignment and 3 right-side indicators each 1 cell wide with 1-cell spacing
- **THEN** the label is centered within the space left of the indicators

### Requirement: Title bar indicator symbols
The title bar SHALL render zero or more indicator symbols on the right side. Each indicator has a glyph (single character or short string) and an individual style. Indicators are rendered right-to-left with a configurable gap between them.

#### Scenario: Three indicators
- **WHEN** the title bar has indicators [close="x", maximize="□", minimize="_"] with gap 1
- **THEN** the rightmost cells show "_ □ x" (or the configured glyphs) with the specified gap between each

#### Scenario: No indicators
- **WHEN** the title bar has no indicators configured
- **THEN** the full Rect width is available for the label

### Requirement: Title bar model tracks focused indicator
The `TitleBarModel` SHALL track which indicator (if any) is focused, identified by index. Focus SHALL be movable left and right through indicators. When no indicator is focused, the focus index SHALL be `None`.

#### Scenario: Move focus right
- **WHEN** indicator focus is at index 0 and move_right is called with 3 indicators present
- **THEN** focus moves to index 1

#### Scenario: Focus wraps or clamps at boundary
- **WHEN** indicator focus is at the last index and move_right is called
- **THEN** focus stays at the last index (no wrap)

#### Scenario: No focus
- **WHEN** indicator focus is None and the consumer queries focused_indicator
- **THEN** the result is None

### Requirement: Title bar style configuration
The title bar SHALL accept separate styles for background, label text, indicator glyphs, and the focused indicator highlight. Styles SHALL be configurable via builder methods.

#### Scenario: Custom styles applied
- **WHEN** the title bar is configured with a blue background style and white label style
- **THEN** the rendered title bar row uses the blue background and white foreground for the label
