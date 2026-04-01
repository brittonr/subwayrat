## ADDED Requirements

### Requirement: Dialog frame renders as a centered overlay
The dialog frame SHALL render as a bordered box centered within the viewport. The dialog size SHALL be configurable as a fixed width/height or as a percentage of the viewport. The area behind the dialog SHALL be cleared before rendering.

#### Scenario: Fixed size centered
- **WHEN** dialog width is 40, height is 15, and viewport is 80x24
- **THEN** the dialog box top-left is at approximately (20, 4) and the background within the dialog area is cleared

#### Scenario: Percentage size
- **WHEN** dialog width is 60% and height is 50% and viewport is 80x24
- **THEN** the dialog is 48 columns wide and 12 rows tall, centered

### Requirement: Dialog frame has title bar region
The top row of the dialog (inside the border) SHALL be reserved for a title bar. The title bar renders the dialog title text using the title-bar component. The title bar occupies exactly 1 row.

#### Scenario: Title rendered
- **WHEN** the dialog has title "Confirm Delete"
- **THEN** the first row inside the border shows "Confirm Delete" using title bar rendering rules (alignment, truncation)

### Requirement: Dialog frame has button row region
The bottom row of the dialog (inside the border) SHALL be reserved for action buttons. Buttons are rendered horizontally, right-aligned by default. Each button has a label and an optional accelerator key hint.

#### Scenario: Two buttons
- **WHEN** the dialog has buttons ["Cancel", "OK"]
- **THEN** the bottom row inside the border shows "Cancel" and "OK" right-aligned with spacing between them

#### Scenario: Single button
- **WHEN** the dialog has buttons ["Close"]
- **THEN** the bottom row shows "Close" right-aligned

### Requirement: Dialog model tracks focused button
The `DialogModel` SHALL track which button is focused by index. Focus SHALL be movable left and right. The focused button SHALL be rendered with a distinct style (highlight or brackets).

#### Scenario: Focus on OK
- **WHEN** buttons are ["Cancel", "OK"] and focused_button is 1
- **THEN** "OK" renders with the focused button style and "Cancel" renders with the normal button style

#### Scenario: Move focus left
- **WHEN** focused_button is 1 and move_left is called
- **THEN** focused_button becomes 0

#### Scenario: Focus clamps at boundaries
- **WHEN** focused_button is 0 and move_left is called
- **THEN** focused_button remains 0

### Requirement: Dialog activate returns button
When the user activates the focused button, the model SHALL return the button index and label so the consumer can handle the action.

#### Scenario: Activate OK
- **WHEN** focused_button is 1 with label "OK" and activate is called
- **THEN** the model returns (1, "OK")

### Requirement: Dialog frame returns content region
The dialog render function SHALL return the inner content `Rect` — the area between the title bar row and the button row, inside the border. The caller renders dialog-specific content into this rect.

#### Scenario: Content region computed
- **WHEN** dialog is 40x15 with 1-row border, 1-row title, and 1-row buttons
- **THEN** the content rect is approximately 38 wide and 11 tall (15 - 2 border rows - 1 title - 1 buttons)

### Requirement: Dialog background dimming
The dialog frame SHALL optionally render a dimmed overlay across the entire viewport before rendering the dialog box. Dimming uses a semi-transparent or dark style applied to all cells outside the dialog. Dimming is enabled by default and can be disabled.

#### Scenario: Dimming enabled
- **WHEN** dimming is enabled (default)
- **THEN** cells outside the dialog border are overwritten with a dim/dark style

#### Scenario: Dimming disabled
- **WHEN** dimming is explicitly disabled
- **THEN** only the dialog border and interior are rendered, leaving surrounding content untouched
