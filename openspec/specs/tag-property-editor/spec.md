## ADDED Requirements

### Requirement: Tag model
The system SHALL represent tags as a `Vec<String>`. The display format SHALL be colon-delimited: `:tag1:tag2:tag3:`. Tags SHALL be case-sensitive and consist of alphanumeric characters, underscores, and hyphens (validated on entry).

#### Scenario: Format tags for display
- **WHEN** tags are ["work", "urgent"]
- **THEN** the display string is ":work:urgent:"

#### Scenario: Parse tags from string
- **WHEN** the string ":work:urgent:" is parsed
- **THEN** the result is vec!["work", "urgent"]

#### Scenario: Empty tags
- **WHEN** tags list is empty
- **THEN** the display string is "" (no colons)

### Requirement: Tag completion popup
The system SHALL provide a `TagEditor` widget that shows the current tag list and a completion popup. When the user begins typing a tag, a popup SHALL appear with matching tags from a provided vocabulary list. The vocabulary is supplied by the caller as `Vec<String>`. Arrow up/down navigates the popup. Enter selects the highlighted tag. Tab accepts the top suggestion.

#### Scenario: Type to filter vocabulary
- **WHEN** the user types "wo" and the vocabulary contains ["work", "personal", "workout"]
- **THEN** the popup shows "work" and "workout"

#### Scenario: Select from popup
- **WHEN** "work" is highlighted in the popup and Enter is pressed
- **THEN** "work" is added to the tag list, the input clears, and the popup closes

#### Scenario: Add tag not in vocabulary
- **WHEN** the user types "newtag" (not in vocabulary) and presses Enter
- **THEN** "newtag" is added to the tag list (vocabulary is not restrictive)

#### Scenario: Remove tag
- **WHEN** Backspace is pressed with empty input and tags ["work", "urgent"] exist
- **THEN** the last tag "urgent" is removed from the list

### Requirement: Priority cookie cycling
The system SHALL provide a `PriorityCycler` that cycles through priority values. The default cycle is: None → 'A' → 'B' → 'C' → None. The cycle order SHALL be configurable via `Vec<char>`. The display format SHALL be `[#A]`.

#### Scenario: Cycle from None
- **WHEN** current priority is None and cycle is called
- **THEN** priority becomes Some('A')

#### Scenario: Cycle through
- **WHEN** priority is 'A' and cycle is called
- **THEN** priority becomes 'B'

#### Scenario: Cycle past last
- **WHEN** priority is 'C' (last in default list) and cycle is called
- **THEN** priority becomes None

#### Scenario: Display format
- **WHEN** priority is Some('A')
- **THEN** the display string is "[#A]"

### Requirement: Property drawer editor
The system SHALL provide a `PropertyEditor` widget that displays a list of key-value pairs. Each row shows a key (left-aligned) and a value (editable TextInput). The user SHALL navigate between rows with arrow up/down and edit values inline. New properties can be added with an "add" action. Properties can be deleted with a "delete" action.

#### Scenario: Display properties
- **WHEN** properties are [("ID", "abc-123"), ("EFFORT", "45")]
- **THEN** the editor renders two rows: "ID: abc-123" and "EFFORT: 45"

#### Scenario: Edit a value
- **WHEN** the user navigates to the "EFFORT" row and types "60"
- **THEN** the value updates to "60"

#### Scenario: Add a property
- **WHEN** the add action is triggered
- **THEN** a new empty row appears at the bottom with focus on the key field

#### Scenario: Delete a property
- **WHEN** delete is triggered on the "EFFORT" row
- **THEN** the "EFFORT" property is removed and selection moves to the adjacent row

### Requirement: TagEditorState and Action enum
The system SHALL expose `TagEditorState` containing: current tags, input text, vocabulary list, popup filtered list, popup selected index, popup visible flag. An `Action` enum SHALL include: `TypeChar(char)`, `Backspace`, `SelectNext`, `SelectPrev`, `AcceptSuggestion`, `AcceptInput`, `RemoveLast`, `Close`. A `handle_action(state, action)` function SHALL process actions.

#### Scenario: Handle AcceptSuggestion
- **WHEN** `handle_action(state, Action::AcceptSuggestion)` is called with "work" highlighted
- **THEN** "work" is appended to tags, input clears, popup closes

### Requirement: PropertyEditorState and Action enum
The system SHALL expose `PropertyEditorState` containing: properties (Vec<(String, String)>), selected row, edit mode (key/value/none), inner TextInput states. An `Action` enum SHALL include: `SelectNext`, `SelectPrev`, `EditKey`, `EditValue`, `AddProperty`, `DeleteProperty`, `Confirm`, `Cancel`. A `handle_action(state, action)` function SHALL process actions.

#### Scenario: Handle AddProperty
- **WHEN** `handle_action(state, Action::AddProperty)` is called
- **THEN** a new ("", "") entry is appended and selection moves to the new row in key-edit mode

### Requirement: Tag and property rendering
The system SHALL implement `StatefulWidget` for both `TagEditor` and `PropertyEditor`. A `TagStyle` SHALL configure: tag chip style, input style, popup style, popup selected style. A `PropertyStyle` SHALL configure: key style, value style, separator style, selected row style, edit highlight style.

#### Scenario: Tag chips rendered inline
- **WHEN** tags are ["work", "urgent"] and the input is empty
- **THEN** the widget renders two styled chips followed by the cursor-ready input area
