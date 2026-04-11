## ADDED Requirements

### Requirement: Spinner state is reusable outside rat-widgets
The workspace SHALL provide a standalone `rat-spinner` crate that owns spinner presets, custom frame sets, playback direction, interval configuration, and animation state without depending on `ratatui`.

#### Scenario: Preset spinner playback
- **WHEN** a caller constructs a preset spinner spec and advances a spinner state
- **THEN** the caller receives the expected frame sequence from `rat-spinner` without importing `rat-widgets`

#### Scenario: Custom spinner playback
- **WHEN** a caller constructs a spinner spec from custom frames
- **THEN** `rat-spinner` SHALL advance through those frames safely, including the empty-frame case

### Requirement: Loader renders with external spinner state
The `rat-widgets::Loader` widget SHALL render from an external `rat_spinner::SpinnerState` and SHALL accept a `rat_spinner::SpinnerSpec` for both preset and custom spinners.

#### Scenario: Loader renders preset spinner
- **WHEN** a caller creates a loader with a preset `SpinnerSpec` and passes a `SpinnerState`
- **THEN** the widget SHALL render the current spinner frame and message using loader styling

#### Scenario: Loader renders custom spinner
- **WHEN** a caller creates a loader with a custom-frame `SpinnerSpec`
- **THEN** the widget SHALL render those custom frames without requiring a loader-owned animation wrapper
