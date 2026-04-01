## ADDED Requirements

### Requirement: Menu bar renders top-level labels
The menu bar SHALL render a horizontal row of top-level menu labels. Each label SHALL be separated by a configurable gap. The active (open) menu label SHALL be rendered with a distinct style.

#### Scenario: Three top-level menus
- **WHEN** the menu bar has labels ["File", "Edit", "View"] with gap 2
- **THEN** the rendered row shows "File  Edit  View" with the specified gap between each

#### Scenario: Active menu highlighted
- **WHEN** the menu bar has "Edit" open
- **THEN** the "Edit" label is rendered with the active menu style

### Requirement: Menu bar tracks open state
The `MenuBarModel` SHALL track whether any menu is open and which top-level menu index is active. Opening a menu SHALL set the active index and mark the bar as open. Closing SHALL clear the open state but preserve the active index for re-opening.

#### Scenario: Open menu
- **WHEN** no menu is open and open is called on index 1
- **THEN** the bar reports is_open=true and active_index=1

#### Scenario: Close menu
- **WHEN** menu at index 1 is open and close is called
- **THEN** the bar reports is_open=false and active_index=1

#### Scenario: Switch menu
- **WHEN** menu at index 0 is open and move_right is called
- **THEN** active_index becomes 1 and the bar remains open

### Requirement: Dropdown renders menu items
When a menu is open, the dropdown SHALL render below the active top-level label. Each `MenuItem::Action` SHALL display its label and optional accelerator hint (right-aligned). `MenuItem::Separator` SHALL render as a horizontal line. `MenuItem::Submenu` SHALL display its label with a right-pointing indicator.

#### Scenario: Action with accelerator
- **WHEN** the dropdown contains an Action with label "Save" and accel "Ctrl+S"
- **THEN** the rendered row shows "Save" left-aligned and "Ctrl+S" right-aligned

#### Scenario: Separator between items
- **WHEN** the dropdown contains [Action("Cut"), Separator, Action("Paste")]
- **THEN** a horizontal divider line appears between "Cut" and "Paste"

#### Scenario: Submenu indicator
- **WHEN** the dropdown contains a Submenu with label "Recent Files"
- **THEN** the rendered row shows "Recent Files" with a right-pointing arrow on the right side

### Requirement: Disabled items are visually distinct and non-selectable
A `MenuItem::Action` with `enabled: false` SHALL be rendered with a dimmed style. Navigation SHALL skip disabled items when moving up or down.

#### Scenario: Disabled item rendered dim
- **WHEN** the dropdown contains an Action with label "Undo" and enabled=false
- **THEN** "Undo" is rendered with the disabled item style

#### Scenario: Navigation skips disabled item
- **WHEN** the focused item is at index 0 and item at index 1 is disabled and item at index 2 is enabled and move_down is called
- **THEN** focus moves to index 2, skipping index 1

### Requirement: Submenu opens on navigation
When a `MenuItem::Submenu` is focused and the right-arrow action is invoked, the submenu's children SHALL be displayed as a new dropdown adjacent to the parent item. Left-arrow from a submenu SHALL close it and return focus to the parent item.

#### Scenario: Open submenu
- **WHEN** a Submenu item is focused and enter_submenu is called
- **THEN** a child dropdown appears to the right of the parent dropdown, and focus moves to the first enabled item in the child

#### Scenario: Close submenu
- **WHEN** focus is inside a submenu and leave_submenu is called
- **THEN** the child dropdown is hidden and focus returns to the parent Submenu item

### Requirement: Menu navigation wraps within dropdown
Moving down past the last enabled item SHALL wrap to the first enabled item. Moving up past the first enabled item SHALL wrap to the last enabled item. Separators are always skipped.

#### Scenario: Wrap at bottom
- **WHEN** focus is on the last enabled item in the dropdown and move_down is called
- **THEN** focus wraps to the first enabled item

#### Scenario: Wrap at top
- **WHEN** focus is on the first enabled item in the dropdown and move_up is called
- **THEN** focus wraps to the last enabled item

### Requirement: Menu bar returns selected action
When the user activates a focused `MenuItem::Action` (via enter/select), the model SHALL return the item's label (or a caller-provided identifier) so the consumer can dispatch the action. If the activated item is disabled, no action SHALL be returned.

#### Scenario: Activate enabled item
- **WHEN** focus is on an enabled Action with label "Save" and activate is called
- **THEN** the model returns Some("Save") and the menu closes

#### Scenario: Activate disabled item
- **WHEN** focus is on a disabled Action and activate is called
- **THEN** the model returns None and focus does not change

### Requirement: Dropdown width fits content
The dropdown width SHALL be the maximum of: the widest item label + accelerator + padding, and a configurable minimum width. The dropdown SHALL not exceed the viewport width.

#### Scenario: Width from longest item
- **WHEN** the dropdown has items ["Open" (accel "Ctrl+O"), "Save As..." (no accel)] with padding 2
- **THEN** the dropdown width accommodates "Open    Ctrl+O" or "Save As..." + padding, whichever is wider

#### Scenario: Minimum width
- **WHEN** all items are short (e.g., "Cut") and minimum width is 15
- **THEN** the dropdown is 15 columns wide
