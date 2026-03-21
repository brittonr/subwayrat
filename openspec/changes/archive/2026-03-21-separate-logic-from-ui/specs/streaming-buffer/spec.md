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
