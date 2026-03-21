## ADDED Requirements

### Requirement: Pure selection model
`SelectListModel` SHALL hold `items: Vec<String>`, `selected: usize`, and `visible: bool`. It SHALL provide `move_up()`, `move_down()`, and `select() -> Option<String>` methods. It SHALL have no ratatui dependency.

#### Scenario: Model compiles without ratatui
- **WHEN** `SelectListModel` is compiled
- **THEN** it compiles without the `ratatui` crate in scope

#### Scenario: Navigation clamps to bounds
- **WHEN** `selected` is 0 and `move_up()` is called
- **THEN** `selected` remains 0

#### Scenario: Select returns current item
- **WHEN** items are `["a", "b", "c"]` and `selected` is 1
- **THEN** `select()` returns `Some("b".to_string())`

### Requirement: Separate rendering widget
A `SelectListWidget` or free function SHALL accept `&SelectListModel`, a title, and an optional theme, and render the popup overlay. The rendering code SHALL live in a separate type or function from `SelectListModel`.

#### Scenario: Render from model reference
- **WHEN** rendering is called with `&model`, title "Pick one", and a theme
- **THEN** a popup is rendered showing the items with the selected item highlighted

### Requirement: Convenience constructor preserved
A convenience function or type alias SHALL exist so that callers who want a combined model+title can construct one in a single call, matching the ergonomics of the current `SelectList::new(title, items)`.

#### Scenario: Simple construction still works
- **WHEN** caller creates a select list with title and items
- **THEN** both model and rendering config are available without separate construction steps
