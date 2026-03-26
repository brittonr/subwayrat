## ADDED Requirements

### Requirement: CalendarGrid widget
The system SHALL provide a `CalendarGrid` widget that renders a single month as a 7-column grid. Column headers SHALL show abbreviated day names (Mo Tu We Th Fr Sa Su, configurable start day). Each cell SHALL show the day number. The selected date SHALL be highlighted. Today SHALL have a distinct style.

#### Scenario: Render current month
- **WHEN** CalendarGrid is rendered for March 2026
- **THEN** the grid shows "March 2026" header, day-name columns, and 31 day cells in the correct weekday positions

#### Scenario: Navigate with arrow keys
- **WHEN** the user presses right-arrow with March 15 selected
- **THEN** March 16 becomes selected

#### Scenario: Navigate across month boundary
- **WHEN** the user presses right-arrow with March 31 selected
- **THEN** the grid advances to April 2026 and April 1 is selected

#### Scenario: Navigate with up/down
- **WHEN** the user presses down-arrow
- **THEN** the selection moves to the same weekday in the next week (7 days forward)

### Requirement: TimeInput widget
The system SHALL provide a `TimeInput` widget for entering time as HH:MM in 24-hour format. The widget SHALL show two numeric fields separated by a colon. Tab or colon SHALL move focus between hours and minutes. Arrow up/down SHALL increment/decrement the focused field.

#### Scenario: Type time directly
- **WHEN** the user types "14" then ":" then "30"
- **THEN** the TimeInput displays "14:30"

#### Scenario: Increment hours
- **WHEN** hours field is focused showing "09" and up-arrow is pressed
- **THEN** hours becomes "10"

#### Scenario: Hours wrap
- **WHEN** hours shows "23" and up-arrow is pressed
- **THEN** hours wraps to "00"

#### Scenario: Minutes wrap
- **WHEN** minutes shows "59" and up-arrow is pressed
- **THEN** minutes wraps to "00"

### Requirement: RepeaterInput widget
The system SHALL provide a `RepeaterInput` for entering org-style repeater intervals. The widget SHALL accept a count (integer) and a unit (d=day, w=week, m=month, y=year) and a mode (+, ++, .+). The display format SHALL be e.g. `+1w`, `++2m`, `.+3d`.

#### Scenario: Select repeater
- **WHEN** the user sets count=1, unit=w, mode=+
- **THEN** the widget displays "+1w" and `value()` returns `Repeater { mode: Plus, count: 1, unit: Week }`

#### Scenario: Cycle unit
- **WHEN** the user presses a cycle key on the unit field
- **THEN** the unit advances d→w→m→y→d

#### Scenario: No repeater
- **WHEN** the repeater is cleared/disabled
- **THEN** `value()` returns None

### Requirement: DatePickerState and Action enum
The system SHALL expose `CalendarGridState` with: selected date, displayed month/year, and week start day config. `TimeInputState` with: hours, minutes, focused field. `RepeaterInputState` with: mode, count, unit, enabled flag. Each SHALL have a corresponding `Action` enum and `handle_action()` function.

#### Scenario: Confirm date selection
- **WHEN** the user presses Enter on a selected date in CalendarGrid
- **THEN** `handle_action` returns `ActionResult::Confirmed(date)` that the caller can use

#### Scenario: Cancel without selection
- **WHEN** the user presses Escape in CalendarGrid
- **THEN** `handle_action` returns `ActionResult::Cancelled`

### Requirement: DatePicker style
The system SHALL accept a `CalendarStyle` struct with: selected day style, today style, weekday header style, weekend day style, out-of-month day style, and month/year title style.

#### Scenario: Weekend days styled differently
- **WHEN** the calendar renders Saturday and Sunday cells
- **THEN** they use the `weekend` style from CalendarStyle
