## ADDED Requirements

### Requirement: Context menu renders at a given position
The context menu SHALL render as a floating box at a specified (x, y) anchor position. The menu SHALL reposition to stay within viewport bounds — shifting left if it would overflow the right edge, shifting up if it would overflow the bottom edge.

#### Scenario: Menu fits at anchor
- **WHEN** anchor is (10, 5) and the menu is 15 wide, 8 tall, and the viewport is 80x24
- **THEN** the menu renders with top-left at (10, 5)

#### Scenario: Overflow right edge
- **WHEN** anchor is (70, 5) and the menu is 15 wide and the viewport is 80 wide
- **THEN** the menu renders with top-left at (65, 5) so the right edge aligns with column 80

#### Scenario: Overflow bottom edge
- **WHEN** anchor is (10, 20) and the menu is 8 tall and the viewport is 24 tall
- **THEN** the menu renders with bottom at row 24, shifting upward

### Requirement: Context menu uses shared MenuItem model
The context menu SHALL accept a `Vec<MenuItem>` using the same `MenuItem` enum as the menu-system (Action, Submenu, Separator). Rendering and navigation rules for item types SHALL be identical to the dropdown behavior.

#### Scenario: Mixed item types
- **WHEN** the context menu has [Action("Copy"), Separator, Submenu("Paste Special", [Action("Plain Text"), Action("HTML")])]
- **THEN** the menu renders with the same visual treatment as a menu bar dropdown

### Requirement: Context menu navigation
The context menu SHALL support up/down navigation through enabled items, submenu entry/exit via right/left actions, wrapping at boundaries, and skipping separators and disabled items. Behavior SHALL match menu-system dropdown navigation.

#### Scenario: Navigate down
- **WHEN** focus is on the first item and move_down is called
- **THEN** focus moves to the next enabled, non-separator item

#### Scenario: Enter submenu
- **WHEN** focus is on a Submenu item and enter_submenu is called
- **THEN** a child menu appears to the right and focus moves into it

### Requirement: Context menu dismissal
The context menu SHALL be dismissable via an escape/close action. Dismissal SHALL close the menu and any open submenus. The model SHALL report whether the menu is visible.

#### Scenario: Dismiss open menu
- **WHEN** the context menu is visible with a submenu open and dismiss is called
- **THEN** the entire menu tree closes and is_visible returns false

#### Scenario: Activate and dismiss
- **WHEN** focus is on an enabled Action and activate is called
- **THEN** the model returns the selected item and the menu closes

### Requirement: Context menu clears underlying content
The context menu SHALL render a `Clear` widget behind its content to erase whatever was previously drawn in that region. The menu border and items render on top of the cleared area.

#### Scenario: Content behind menu is erased
- **WHEN** the context menu renders at (10, 5) with size 15x8
- **THEN** the rectangular area (10, 5, 15, 8) is cleared before menu content is drawn
