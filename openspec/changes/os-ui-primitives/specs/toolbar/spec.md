## ADDED Requirements

### Requirement: Toolbar renders items in a strip
The toolbar SHALL render items as a horizontal or vertical strip within the given Rect. Items are rendered sequentially along the primary axis. Each item occupies one cell height (horizontal) or one cell width (vertical) for its label row.

#### Scenario: Horizontal toolbar with three buttons
- **WHEN** the toolbar has items [Button("New"), Button("Open"), Button("Save")] in horizontal orientation with gap 1
- **THEN** the items render left-to-right with 1 cell gap between each

#### Scenario: Vertical toolbar
- **WHEN** the toolbar has items [Button("New"), Button("Open")] in vertical orientation
- **THEN** the items render top-to-bottom

### Requirement: Toolbar item types
The toolbar SHALL support three item types: `Button` (label, triggers action on activate), `Toggle` (label, on/off state, toggles on activate), and `Separator` (visual divider with no interaction).

#### Scenario: Toggle item renders on state
- **WHEN** a Toggle item "Bold" has state on=true
- **THEN** it renders with the active toggle style

#### Scenario: Toggle item renders off state
- **WHEN** a Toggle item "Bold" has state on=false
- **THEN** it renders with the inactive toggle style

#### Scenario: Separator renders divider
- **WHEN** the toolbar has [Button("A"), Separator, Button("B")] in horizontal orientation
- **THEN** a vertical line character appears between A and B

### Requirement: Toolbar items support icon and label
Each Button and Toggle item SHALL accept an optional icon string and a label string. When both are present, the icon renders before the label. When only icon is present, only the icon renders (compact mode). When only label is present, only the label renders.

#### Scenario: Icon and label
- **WHEN** a Button has icon "📁" and label "Open"
- **THEN** the rendered item shows "📁 Open"

#### Scenario: Icon only
- **WHEN** a Button has icon "📁" and no label
- **THEN** the rendered item shows "📁"

### Requirement: Toolbar model tracks focus and toggle states
The `ToolbarModel` SHALL track which item is focused (by index, skipping separators) and the on/off state of each Toggle item. Focus movement SHALL skip Separator items.

#### Scenario: Move focus past separator
- **WHEN** items are [Button, Separator, Toggle] and focus is on index 0 and move_next is called
- **THEN** focus moves to index 2, skipping the separator at index 1

#### Scenario: Toggle state change
- **WHEN** a Toggle item at index 2 is focused with state on=false and activate is called
- **THEN** the Toggle state becomes on=true

### Requirement: Toolbar activate returns item identity
When the user activates a focused Button, the model SHALL return the item index so the consumer can dispatch the action. For Toggle items, the model SHALL return the item index and the new toggle state.

#### Scenario: Activate button
- **WHEN** focus is on Button at index 0 and activate is called
- **THEN** the model returns Activated(0)

#### Scenario: Activate toggle
- **WHEN** focus is on Toggle at index 2 (currently off) and activate is called
- **THEN** the model returns Toggled(2, true)

### Requirement: Toolbar overflow indicator
When items exceed the available space along the primary axis, the toolbar SHALL render an overflow indicator (e.g., ">>") at the trailing edge. Items that do not fit SHALL not be rendered. Focus movement into overflow items SHALL scroll the visible window.

#### Scenario: Items overflow
- **WHEN** the toolbar has 10 items but only 5 fit in the available width
- **THEN** the first 4 items render plus an overflow indicator at the right edge

#### Scenario: Focus scrolls into overflow
- **WHEN** focus moves to an item that is currently in the overflow region
- **THEN** the visible window shifts to include the focused item
