## ADDED Requirements

### Requirement: Tooltip renders floating text at anchor position
The tooltip SHALL render a bordered box containing text content at a specified (x, y) anchor position. The box renders on top of existing content by clearing its footprint first.

#### Scenario: Single line tooltip
- **WHEN** anchor is (20, 10) and content is "Save file"
- **THEN** a bordered box containing "Save file" renders with its top-left near (20, 10)

#### Scenario: Multi-line tooltip
- **WHEN** content is "Line 1\nLine 2\nLine 3"
- **THEN** the tooltip box is 3 rows tall (plus border) and wide enough for the longest line

### Requirement: Tooltip vertical repositioning
If the tooltip would extend below the viewport bottom, it SHALL render above the anchor point instead. If it would extend above the viewport top (unlikely but possible), it SHALL clamp to row 0.

#### Scenario: Tooltip flips above anchor
- **WHEN** anchor is (10, 22) and the tooltip is 5 rows tall and the viewport is 24 rows
- **THEN** the tooltip renders above the anchor, ending at or near row 22

#### Scenario: Tooltip fits below anchor
- **WHEN** anchor is (10, 5) and the tooltip is 5 rows tall and the viewport is 24 rows
- **THEN** the tooltip renders below the anchor starting near row 6

### Requirement: Tooltip width from content
The tooltip width SHALL be determined by the longest line of content plus border and padding. A configurable maximum width SHALL cause line wrapping when content exceeds it.

#### Scenario: Width from content
- **WHEN** content is "Short" with no max width
- **THEN** the tooltip is 5 characters wide plus border/padding

#### Scenario: Max width wraps
- **WHEN** content is "This is a long line of text that should wrap" and max width is 20
- **THEN** the tooltip is 20 columns wide and the text wraps across multiple rows

### Requirement: Tooltip border and style configuration
The tooltip SHALL accept a border style (ratatui `BorderType` or no border) and styles for the text content and background. Defaults SHALL produce a visible tooltip (single-line border, contrasting colors).

#### Scenario: Custom border
- **WHEN** the tooltip is configured with BorderType::Rounded
- **THEN** the tooltip box uses rounded corner characters

#### Scenario: No border
- **WHEN** the tooltip is configured with no border
- **THEN** the tooltip renders as plain text with background fill and no border characters

### Requirement: Tooltip is non-interactive
The tooltip SHALL have no model, no focus state, and no navigation. It is a pure rendering primitive. The caller controls when to render it and with what content.

#### Scenario: No mutable state
- **WHEN** a tooltip is rendered
- **THEN** it takes anchor position, content, and style as direct parameters with no persistent state object
