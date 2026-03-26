## ADDED Requirements

### Requirement: AgendaItem data type
The system SHALL define an `AgendaItem` struct containing: `id` (String), `title` (String), `status` (Option<String>), `priority` (Option<char>), `tags` (Vec<String>), `scheduled` (Option<Date>), `deadline` (Option<Date>), `time_start` (Option<Time>), `time_end` (Option<Time>), `source_file` (Option<String>), `source_line` (Option<usize>). The type SHALL be generic over the data source.

#### Scenario: Construct a minimal item
- **WHEN** an AgendaItem is created with only id and title
- **THEN** all optional fields are None and the item is valid

#### Scenario: Item with full planning data
- **WHEN** an AgendaItem has scheduled=2026-03-25, deadline=2026-03-28, priority='A', tags=["work"]
- **THEN** all fields are accessible and the item can be rendered in any view mode

### Requirement: AgendaDataSource trait
The system SHALL define a trait `AgendaDataSource` with method `fn items(&self, range: DateRange) -> Vec<AgendaItem>`. The widget SHALL call this trait to obtain items for the visible date range. The trait SHALL be object-safe.

#### Scenario: Static data source
- **WHEN** a Vec<AgendaItem> is wrapped in a struct implementing AgendaDataSource
- **THEN** calling `items(range)` returns items whose scheduled or deadline falls within the range

#### Scenario: Empty range
- **WHEN** `items` is called with a range that contains no items
- **THEN** an empty Vec is returned

### Requirement: Day view
The system SHALL render a single-day view showing all items for that date, ordered by time (timed items first, sorted by start time, then untimed items sorted by priority). Each item line SHALL show: time slot (if timed), priority cookie, TODO status, title, and tags.

#### Scenario: Day with timed and untimed items
- **WHEN** the day has 2 timed items (09:00, 14:00) and 3 untimed items
- **THEN** the timed items appear first in time order, followed by untimed items sorted by priority

#### Scenario: Empty day
- **WHEN** the selected day has no items
- **THEN** the view shows the date header with an empty content area

### Requirement: Week view
The system SHALL render a 7-day columnar view (Mon-Sun or configurable start day). Each column SHALL show the day name, date, and a vertical list of items for that day. The currently selected day SHALL be highlighted.

#### Scenario: Navigate between days
- **WHEN** the user presses right-arrow in week view
- **THEN** the selection moves to the next day column

#### Scenario: Week wrapping
- **WHEN** the user presses right-arrow on the last day of the week
- **THEN** the view advances to the next week and selects the first day

### Requirement: Month view
The system SHALL render a month calendar grid (7 columns × 4-6 rows). Each cell SHALL show the day number and a count or abbreviated list of items. The selected day SHALL be highlighted. Days outside the current month SHALL be dimmed.

#### Scenario: Navigate month grid
- **WHEN** the user presses down-arrow in month view
- **THEN** the selection moves to the same weekday in the next week row

#### Scenario: Month boundary
- **WHEN** the user navigates past the last day of the month
- **THEN** the view advances to the next month

### Requirement: Filter bar
The system SHALL provide a filter bar that narrows visible items by: status (TODO, DONE, etc.), tags (include/exclude), priority (A/B/C), and free-text search on title. Multiple filters SHALL combine with AND logic. The filter bar SHALL render as a horizontal row above the agenda content.

#### Scenario: Filter by tag
- **WHEN** the user activates filter and types tag "work"
- **THEN** only items with the "work" tag are shown in the agenda view

#### Scenario: Clear filters
- **WHEN** the user clears all filters
- **THEN** all items for the visible date range are shown

### Requirement: AgendaState and Action enum
The system SHALL expose `AgendaState` containing: view mode (day/week/month), selected date, selected item index, filter state, data source reference, and scroll offset. An `Action` enum SHALL include: `NextDay`, `PrevDay`, `NextWeek`, `PrevWeek`, `NextMonth`, `PrevMonth`, `SwitchView(ViewMode)`, `SelectItem(usize)`, `ToggleFilter`, `SetFilter(FilterSpec)`, `ClearFilters`, `Refresh`. A `handle_action(state, action)` function SHALL process actions.

#### Scenario: Switch from day to week view
- **WHEN** `handle_action(state, Action::SwitchView(ViewMode::Week))` is called
- **THEN** the view mode changes to week with the current day selected in the week

#### Scenario: Refresh re-queries data source
- **WHEN** `handle_action(state, Action::Refresh)` is called
- **THEN** the state calls `data_source.items()` for the current visible range and updates the item list

### Requirement: Agenda rendering
The system SHALL implement `StatefulWidget` for ratatui. The widget SHALL accept an `AgendaStyle` struct for configuring colors of: day headers, time slots, priority levels (A=red, B=yellow, C=green by default), TODO states, tags, selected item highlight, today highlight, and dimmed days.

#### Scenario: Today is highlighted
- **WHEN** the agenda renders and today's date is visible
- **THEN** today's date header or cell uses the `today` style from AgendaStyle

#### Scenario: Selected item highlighted
- **WHEN** the cursor is on item index 2
- **THEN** item 2 is rendered with the selection highlight style
