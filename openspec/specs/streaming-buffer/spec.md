## ADDED Requirements

### Requirement: Render-free streaming buffer
`StreamingOutput` SHALL have no ratatui dependency. It SHALL store lines in a head/tail ring buffer, track total lines/bytes, manage scroll offset and auto-follow state, and expose display lines via `get_display_line(index)`. All rendering (producing `Line`/`Span` types) SHALL be in separate free functions outside the struct.

#### Scenario: Buffer has no ratatui imports
- **WHEN** `StreamingOutput` is compiled
- **THEN** it compiles without the `ratatui` crate in scope

#### Scenario: Display lines accessible without rendering
- **WHEN** 10 lines are pushed into a buffer
- **THEN** `display_line_count()` returns 10 and `get_display_line(0..10)` returns each line's text without any styling information

### Requirement: Standalone render functions for streaming output
Free functions `render_streaming_lines()` and `render_streaming_stats()` SHALL accept a `&StreamingOutput` reference plus style parameters and return ratatui `Line` values. These functions SHALL live in a separate module or file from the buffer data struct.

#### Scenario: Render lines from buffer reference
- **WHEN** `render_streaming_lines(&buffer, visible_height, border_style)` is called
- **THEN** it returns `Vec<Line>` with styled spans matching the buffer's visible window

#### Scenario: Render stats from buffer reference
- **WHEN** `render_streaming_stats(&buffer, border_style)` is called
- **THEN** it returns a `Line` showing total lines, bytes, follow indicator, and omission count

### Requirement: StreamingOutputManager unchanged
`StreamingOutputManager` SHALL remain a HashMap wrapper around `StreamingOutput` buffers. Its API (`add_line`, `add_text`, `get`, `get_mut`, `remove`, `focus`, `unfocus_all`) SHALL not change.

#### Scenario: Manager API stable
- **WHEN** existing code calls `manager.add_line("call-1", "hello")`
- **THEN** the call compiles and behaves identically to before the refactor

### Requirement: InlineWidget trait implementation
`StreamingOutput` SHALL implement the `InlineWidget` trait (from `rat-inline`) behind an `inline` feature flag so it can participate as a leaf node in inline view trees. The implementation SHALL compute height from the visible line count and render the tail of the output into the allocated buffer region.

#### Scenario: StreamingOutput in inline tree
- **WHEN** a `StreamingOutput` is used as a node in an inline view tree with the `inline` feature enabled
- **THEN** it renders its visible lines into the allocated buffer region

#### Scenario: Height measurement
- **WHEN** `InlineWidget::height()` is called on a `StreamingOutput` with `visible_lines: 16`
- **THEN** it returns `min(total_display_lines, 16)` as the measured height

#### Scenario: State preserved across rebuilds
- **WHEN** a keyed `StreamingOutput` node is matched by the reconciler across frames
- **THEN** its scroll offset, auto-follow mode, and buffer contents are preserved
