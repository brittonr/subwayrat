## ADDED Requirements

### Requirement: Breadcrumb renders path segments with separators
The breadcrumb SHALL render a sequence of text segments separated by a configurable separator character (default: "/"). Each segment is rendered inline on a single row.

#### Scenario: Three segments
- **WHEN** segments are ["home", "user", "documents"] with separator "/"
- **THEN** the rendered output is "home / user / documents"

#### Scenario: Custom separator
- **WHEN** segments are ["src", "lib"] with separator ">"
- **THEN** the rendered output is "src > lib"

### Requirement: Breadcrumb truncates from the left
When the full breadcrumb path exceeds available width, segments SHALL be dropped from the left (earliest segments removed first). A configurable ellipsis prefix (default: "...") SHALL replace the dropped segments.

#### Scenario: Truncation
- **WHEN** segments are ["home", "user", "projects", "subwayrat", "crates", "rat-chrome"] and available width is 30 columns
- **THEN** the rendered output starts with "..." followed by the rightmost segments that fit

#### Scenario: No truncation needed
- **WHEN** the full path fits within available width
- **THEN** all segments render without ellipsis

#### Scenario: Single segment exceeds width
- **WHEN** segments are ["a-very-long-directory-name-that-is-enormous"] and available width is 20 columns
- **THEN** the single segment is truncated at the character level with trailing ellipsis

### Requirement: Breadcrumb tracks active segment
The `BreadcrumbModel` SHALL track which segment index is "active" (the current location). The active segment SHALL be rendered with a distinct style. By default, the last segment is active.

#### Scenario: Last segment active
- **WHEN** segments are ["home", "user", "docs"] and no explicit active is set
- **THEN** "docs" renders with the active segment style

#### Scenario: Explicit active
- **WHEN** segments are ["home", "user", "docs"] and active is set to index 1
- **THEN** "user" renders with the active segment style

### Requirement: Breadcrumb navigation
The `BreadcrumbModel` SHALL support moving the active segment left and right. Moving left from the first segment SHALL be a no-op. Moving right from the last segment SHALL be a no-op.

#### Scenario: Move left
- **WHEN** active is at index 2 and move_left is called
- **THEN** active becomes index 1

#### Scenario: Move left at start
- **WHEN** active is at index 0 and move_left is called
- **THEN** active remains at index 0

### Requirement: Breadcrumb select returns segment
When the user activates the current active segment, the model SHALL return the segment index and label so the consumer can navigate to that path.

#### Scenario: Select active segment
- **WHEN** active is at index 1 with label "user" and select is called
- **THEN** the model returns (1, "user")
