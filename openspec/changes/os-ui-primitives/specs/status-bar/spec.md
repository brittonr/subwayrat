## ADDED Requirements

### Requirement: Status bar renders three segments
The status bar SHALL render a single row divided into left, center, and right segments. The left segment is left-aligned, the center segment is centered in the remaining space, and the right segment is right-aligned.

#### Scenario: All three segments populated
- **WHEN** left="NORMAL", center="main.rs", right="Ln 42, Col 8" and the bar is 80 columns wide
- **THEN** "NORMAL" appears at the left edge, "main.rs" appears centered, and "Ln 42, Col 8" appears at the right edge

#### Scenario: Only left and right
- **WHEN** left="INSERT", center is empty, right="UTF-8"
- **THEN** "INSERT" appears at the left edge and "UTF-8" at the right edge with empty space between

#### Scenario: Empty status bar
- **WHEN** all three segments are empty
- **THEN** the bar renders as a full-width row with the background style only

### Requirement: Segments accept styled spans
Each segment SHALL accept a `Vec<Span>` (ratatui styled text spans), not plain strings. This allows mixed styling within a single segment (e.g., a mode indicator with distinct background color followed by a file path in normal color).

#### Scenario: Mixed styles in left segment
- **WHEN** the left segment contains [Span("NORMAL", blue_bg), Span(" main.rs", default)]
- **THEN** the "NORMAL" text renders with blue background and "main.rs" renders with default styling, concatenated

### Requirement: Center segment truncation
When the center segment text is wider than the space remaining after the left and right segments, the center text SHALL be truncated with an ellipsis. Left and right segments SHALL never be truncated by the center segment.

#### Scenario: Center truncated
- **WHEN** left is 20 chars, right is 20 chars, center is 50 chars, and the bar is 60 columns wide
- **THEN** left and right render fully (40 chars total), and center is truncated to fit the remaining 20 columns with an ellipsis

#### Scenario: No space for center
- **WHEN** left + right together exceed the bar width
- **THEN** the center segment is not rendered at all

### Requirement: Status bar is non-interactive
The status bar SHALL have no model or mutable state. It is a pure rendering primitive that accepts content and style configuration and produces output. No focus tracking, no navigation.

#### Scenario: No model struct
- **WHEN** a status bar is constructed
- **THEN** it takes content (left/center/right spans) and style directly at render time, with no separate model object

### Requirement: Status bar background fills full width
The status bar background style SHALL fill the entire Rect width, including space not occupied by text. This produces the continuous background bar typical of OS status bars.

#### Scenario: Background fills gaps
- **WHEN** the bar is 80 columns wide and total text content is 40 columns
- **THEN** all 80 columns have the background style applied
