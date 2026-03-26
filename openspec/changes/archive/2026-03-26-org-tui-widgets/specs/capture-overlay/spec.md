## ADDED Requirements

### Requirement: CaptureTemplate type
The system SHALL define a `CaptureTemplate` struct containing: `name` (String), `icon` (Option<char>), `target_file` (Option<String>), `target_heading` (Option<String>), `initial_content` (Option<String>), `include_body` (bool, default true). Templates are provided by the caller — the widget does not manage template storage.

#### Scenario: Template with defaults
- **WHEN** a CaptureTemplate is created with name="Note" and all other fields as None/default
- **THEN** the template is valid and include_body is true

#### Scenario: Template with target
- **WHEN** a CaptureTemplate has target_file="inbox.org" and target_heading="Inbox"
- **THEN** the capture result includes both fields for the caller to use during filing

### Requirement: Capture overlay lifecycle
The system SHALL provide a `CaptureOverlay` widget that renders as a centered popup. The lifecycle SHALL be: **closed** (not rendered) → **template-select** (user picks a template) → **editing** (user fills title and body) → **confirmed** or **cancelled** (overlay closes, result returned). The overlay SHALL be openable via `CaptureState::open(templates)` and closeable via Escape at any phase.

#### Scenario: Open capture
- **WHEN** `state.open(templates)` is called with 3 templates
- **THEN** the state transitions to template-select phase showing 3 options

#### Scenario: Single template skips selection
- **WHEN** `state.open(templates)` is called with exactly 1 template
- **THEN** the state skips template-select and goes directly to editing phase

#### Scenario: Cancel during template selection
- **WHEN** Escape is pressed during template-select phase
- **THEN** the state transitions to closed with no result

#### Scenario: Cancel during editing
- **WHEN** Escape is pressed during editing phase
- **THEN** the state transitions to closed with no result (content is discarded)

### Requirement: Capture editing phase
During the editing phase, the overlay SHALL show: the template name as a title bar, a `TextInput` for the item title, and (if `include_body` is true) a `rat-editor::Editor` for the body text. Tab SHALL move focus between title and body. Enter on the title field with a non-empty title SHALL confirm the capture.

#### Scenario: Fill title and body
- **WHEN** the user types "Meeting notes" in the title and "Discussed Q2 roadmap" in the body and presses Ctrl+Enter
- **THEN** the capture result contains title="Meeting notes", body="Discussed Q2 roadmap", and the selected template

#### Scenario: Empty title rejects confirm
- **WHEN** the user presses Enter/Ctrl+Enter with an empty title
- **THEN** the capture is not confirmed and the title input shows an error indicator

### Requirement: CaptureResult type
The system SHALL define a `CaptureResult` struct containing: `template` (CaptureTemplate used), `title` (String), `body` (Option<String>), `timestamp` (the time of capture). The caller uses this to actually write to a file — the widget does not do I/O.

#### Scenario: Result after confirm
- **WHEN** the user confirms a capture with title "Buy groceries" and no body
- **THEN** `state.take_result()` returns Some(CaptureResult) with title="Buy groceries", body=None, and a timestamp

#### Scenario: Result after cancel
- **WHEN** the user cancels the capture
- **THEN** `state.take_result()` returns None

### Requirement: Capture rendering
The system SHALL implement `StatefulWidget` for ratatui. The overlay SHALL render as a bordered box, centered horizontally and vertically, taking 60% width and 50% height of the available area (configurable). A `CaptureStyle` struct SHALL configure: border style, title bar style, input styles, and template list styles.

#### Scenario: Overlay size
- **WHEN** the overlay renders in an 80×24 terminal
- **THEN** the popup is approximately 48 columns wide and 12 rows tall, centered

#### Scenario: Template select rendering
- **WHEN** the overlay is in template-select phase with 3 templates
- **THEN** the popup shows a bordered list with template names, the selected one highlighted

### Requirement: CaptureState and Action enum
The system SHALL expose `CaptureState` containing: phase (closed/template-select/editing), templates list, selected template index, title input, body editor, and result. An `Action` enum SHALL include: `Open(Vec<CaptureTemplate>)`, `SelectTemplate(usize)`, `ConfirmTemplate`, `SetTitle(String)`, `Confirm`, `Cancel`, `FocusNext`, `FocusPrev`. A `handle_action(state, action)` function SHALL process actions.

#### Scenario: Handle Open action
- **WHEN** `handle_action(state, Action::Open(templates))` is called while closed
- **THEN** the phase transitions to template-select (or editing if single template)

#### Scenario: Handle Confirm action
- **WHEN** `handle_action(state, Action::Confirm)` is called during editing with non-empty title
- **THEN** the phase transitions to closed and `take_result()` returns Some
