## ADDED Requirements

### Requirement: Completer callback on TextInput

The `TextInput` widget SHALL accept an optional completer function via
`with_completer(Box<dyn Fn(&str) -> Vec<String>>)`. Calling `complete()`
SHALL invoke the completer with the current value and apply common-prefix
logic to the results.

#### Scenario: Single match replaces value

- **WHEN** `complete()` is called and the completer returns exactly one match
- **THEN** the widget's value is set to that match and the cursor moves to end

#### Scenario: Multiple matches fill common prefix

- **WHEN** `complete()` is called and the completer returns multiple matches
- **THEN** the value is set to the longest common prefix of all matches
- **THEN** `complete()` returns the full list of matches for the caller to display

#### Scenario: No matches leaves value unchanged

- **WHEN** `complete()` is called and the completer returns an empty vec
- **THEN** the value and cursor position are unchanged
- **THEN** `complete()` returns an empty vec

#### Scenario: No completer attached

- **WHEN** `complete()` is called on a TextInput with no completer set
- **THEN** `complete()` returns an empty vec and the value is unchanged

### Requirement: Bundled path completer function

`rat-widgets` SHALL provide a `path_completer(input: &str) -> Vec<String>`
function that performs filesystem tab completion.

#### Scenario: Directory listing on trailing slash

- **WHEN** input ends with `/` (e.g. `/tmp/`)
- **THEN** completer returns all entries in that directory

#### Scenario: Prefix filtering

- **WHEN** input is a partial filename (e.g. `/tmp/fo`)
- **THEN** completer returns entries in `/tmp/` whose names start with `fo` (case-insensitive)

#### Scenario: Directories get slash appended

- **WHEN** a matching entry is a directory
- **THEN** the returned string ends with `/`

#### Scenario: Empty input completes from current directory

- **WHEN** input is empty string
- **THEN** completer returns entries from `.`

#### Scenario: Nonexistent directory returns empty

- **WHEN** input refers to a path whose parent directory does not exist
- **THEN** completer returns an empty vec
