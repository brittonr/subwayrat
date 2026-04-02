## MODIFIED Requirements

### Requirement: InlineWidget trait implementation
`StreamingOutput` SHALL implement the `InlineWidget` trait so it can participate as a leaf node in inline view trees. The implementation SHALL use the existing `render_streaming_lines` function for rendering and SHALL compute height from the visible line count.

#### Scenario: StreamingOutput in inline tree
- **WHEN** a `StreamingOutput` is used as a node in an `inline!` view tree
- **THEN** it renders its visible lines into the allocated buffer region using existing rendering logic

#### Scenario: Height measurement
- **WHEN** `InlineWidget::height()` is called on a `StreamingOutput` with `visible_lines: 16`
- **THEN** it returns `min(total_display_lines, 16)` as the measured height

#### Scenario: State preserved across rebuilds
- **WHEN** a keyed `StreamingOutput` node is matched by the reconciler across frames
- **THEN** its scroll offset, auto-follow mode, and buffer contents are preserved
