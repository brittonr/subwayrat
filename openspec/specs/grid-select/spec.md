# grid-select Specification

## Purpose
TBD - created by archiving change irohscii-tui-migration. Update Purpose after archive.
## Requirements
### Requirement: GridSelect widget

`rat-widgets` SHALL provide a `GridSelect` widget that displays items in a 2D
grid popup with configurable column count, arrow/hjkl navigation, and themed
rendering.

#### Scenario: Construction with column count

- **WHEN** `GridSelect::new(title, items, columns)` is called with 8 items and 3 columns
- **THEN** the grid has 3 rows (3, 3, 2 items) and selection starts at index 0

#### Scenario: Empty items list

- **WHEN** `GridSelect::new(title, vec![], columns)` is called
- **THEN** the widget does not panic and `selected_index()` returns 0

### Requirement: GridSelect navigation

Navigation SHALL move a single selection cursor through a 2D grid laid out
left-to-right, top-to-bottom.

#### Scenario: Move right within row

- **WHEN** `move_right()` is called and the selection is not at the last item
- **THEN** selection advances by 1

#### Scenario: Move right at end of items

- **WHEN** `move_right()` is called and the selection is at the last item
- **THEN** selection stays at the last item (no wrap)

#### Scenario: Move left within row

- **WHEN** `move_left()` is called and the selection is not at index 0
- **THEN** selection decreases by 1

#### Scenario: Move left at index 0

- **WHEN** `move_left()` is called and selection is at index 0
- **THEN** selection stays at 0 (no wrap)

#### Scenario: Move down by column width

- **WHEN** `move_down()` is called with selection at index 1 in a 3-column grid
- **THEN** selection moves to index 4 (1 + 3)

#### Scenario: Move down clamped to last item

- **WHEN** `move_down()` is called and `selected + columns >= items.len()`
- **THEN** selection moves to `items.len() - 1`

#### Scenario: Move up by column width

- **WHEN** `move_up()` is called with selection at index 4 in a 3-column grid
- **THEN** selection moves to index 1 (4 - 3)

#### Scenario: Move up clamped to zero

- **WHEN** `move_up()` is called and `selected < columns`
- **THEN** selection moves to the same column in row 0 (i.e. `selected % columns` or just stays via `saturating_sub`)

### Requirement: GridSelect rendering

The widget SHALL render as a centered popup overlay with items in a grid,
the selected cell highlighted, and optional color swatches.

#### Scenario: Popup rendering

- **WHEN** `render()` or `render_themed()` is called with `visible == true`
- **THEN** a bordered popup is drawn centered in the area with grid cells

#### Scenario: Hidden widget

- **WHEN** `render()` is called with `visible == false`
- **THEN** nothing is drawn

#### Scenario: Color swatch display

- **WHEN** an item has `color: Some(Color::Red)`
- **THEN** a colored `█` block is rendered before the item's label

### Requirement: GridItem type

Each grid item SHALL be a `GridItem` struct with a `label: String` and
optional `color: Option<Color>` for rendering a color swatch.

#### Scenario: Item with label only

- **WHEN** a GridItem is created with label "Rectangle" and color None
- **THEN** only the label text is rendered in the cell

#### Scenario: Item with color swatch

- **WHEN** a GridItem is created with label "Red" and color Some(Color::Red)
- **THEN** a red `█` glyph appears before the label text
