# canvas Specification

## Purpose
TBD - created by archiving change irohscii-tui-migration. Update Purpose after archive.
## Requirements
### Requirement: Position type

`rat-canvas` SHALL provide a `Position { x: i32, y: i32 }` type representing
a cell coordinate on an infinite canvas. Negative coordinates are valid.

#### Scenario: Position construction

- **WHEN** `Position::new(-5, 10)` is called
- **THEN** a position with x=-5 y=10 is returned

#### Scenario: Position equality

- **WHEN** two positions with the same x and y are compared
- **THEN** they are equal

### Requirement: Viewport type

`rat-canvas` SHALL provide a `Viewport` type that maps between screen
coordinates (u16, visible terminal area) and canvas coordinates (i32,
infinite space). The viewport tracks an offset (pan position), dimensions
(terminal size), and zoom level.

#### Scenario: Viewport construction

- **WHEN** `Viewport::new(width, height)` is called
- **THEN** offset is (0,0), zoom is 1.0, and dimensions match the arguments

#### Scenario: Screen to canvas at default zoom

- **WHEN** `screen_to_canvas(10, 5)` is called with offset (0,0) and zoom 1.0
- **THEN** the result is `Position { x: 10, y: 5 }`

#### Scenario: Screen to canvas with offset

- **WHEN** `screen_to_canvas(10, 5)` is called with offset (100, 50) and zoom 1.0
- **THEN** the result is `Position { x: 110, y: 55 }`

#### Scenario: Screen to canvas with zoom

- **WHEN** `screen_to_canvas(10, 5)` is called with offset (0,0) and zoom 2.0
- **THEN** the result is `Position { x: 5, y: 3 }` (screen coords divided by zoom, rounded)

### Requirement: Canvas to screen mapping

The viewport SHALL convert canvas positions to screen coordinates, returning
`None` for positions outside the visible area.

#### Scenario: Visible position

- **WHEN** `canvas_to_screen(Position { x: 5, y: 3 })` is called with offset (0,0), zoom 1.0, size 80x24
- **THEN** the result is `Some((5, 3))`

#### Scenario: Position outside viewport

- **WHEN** `canvas_to_screen(Position { x: 100, y: 3 })` is called with size 80x24
- **THEN** the result is `None`

#### Scenario: Negative canvas position with matching offset

- **WHEN** `canvas_to_screen(Position { x: -10, y: 0 })` is called with offset (-20, 0), size 80x24
- **THEN** the result is `Some((10, 0))`

### Requirement: Pan

The viewport SHALL support panning by a delta in canvas units.

#### Scenario: Pan shifts offset

- **WHEN** `pan(5, -3)` is called on a viewport with offset (10, 20)
- **THEN** offset becomes (15, 17)

### Requirement: Zoom

The viewport SHALL support zoom in, zoom out, and reset, clamped to a
configurable range.

#### Scenario: Zoom in

- **WHEN** `zoom_in()` is called with zoom at 1.0 and step 0.25
- **THEN** zoom becomes 1.25

#### Scenario: Zoom clamped to max

- **WHEN** `zoom_in()` is called with zoom at max (4.0)
- **THEN** zoom stays at 4.0

#### Scenario: Zoom out clamped to min

- **WHEN** `zoom_out()` is called with zoom at min (0.25)
- **THEN** zoom stays at 0.25

#### Scenario: Reset zoom

- **WHEN** `reset_zoom()` is called
- **THEN** zoom returns to 1.0

### Requirement: Resize

The viewport SHALL support resizing when the terminal dimensions change.

#### Scenario: Resize updates dimensions

- **WHEN** `resize(120, 40)` is called
- **THEN** width becomes 120 and height becomes 40
- **THEN** offset and zoom are preserved

### Requirement: Visible canvas size

The viewport SHALL report how many canvas cells are visible at the current
zoom level.

#### Scenario: Visible size at zoom 1.0

- **WHEN** `visible_canvas_size()` is called with width=80, height=24, zoom=1.0
- **THEN** the result is (80, 24)

#### Scenario: Visible size at zoom 2.0

- **WHEN** `visible_canvas_size()` is called with width=80, height=24, zoom=2.0
- **THEN** the result is (40, 12)

### Requirement: No ratatui dependency

`rat-canvas` SHALL depend only on `serde` (optional, for serialization) and
have no dependency on `ratatui`. Rendering integration is the caller's
responsibility — the crate provides coordinate math only.

#### Scenario: Crate compiles without ratatui

- **WHEN** `rat-canvas` is compiled with default features
- **THEN** it does not pull in `ratatui` as a dependency
