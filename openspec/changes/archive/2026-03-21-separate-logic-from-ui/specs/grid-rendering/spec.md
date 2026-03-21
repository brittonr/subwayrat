## MODIFIED Requirements

### Requirement: StatefulWidget implementation
The spreadsheet SHALL implement ratatui's `StatefulWidget` trait. The widget struct holds configuration (column widths, frozen panes, styles). The state struct holds mutable data (grid, cursor, selection, scroll offset, edit state). `EditState` SHALL be imported from the `edit_state` module, not defined in `render.rs`. `SpreadsheetState` fields SHALL be organized into documented groups (data model vs. visual layout) but the struct SHALL remain a single type.

#### Scenario: Render via StatefulWidget
- **WHEN** the application calls `frame.render_stateful_widget(spreadsheet, area, &mut state)`
- **THEN** the spreadsheet grid is rendered into the given area using the current state

#### Scenario: EditState imported from edit_state module
- **WHEN** `render.rs` references `EditState`
- **THEN** it uses `use crate::edit_state::EditState` rather than defining the type inline
