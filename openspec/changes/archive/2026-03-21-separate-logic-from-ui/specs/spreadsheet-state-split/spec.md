## ADDED Requirements

### Requirement: EditState in its own module
`EditState` SHALL live in a dedicated `edit_state.rs` module within `rat-spreadsheet`, not in `render.rs`. It SHALL be re-exported from `lib.rs` so that `use rat_spreadsheet::EditState` continues to work.

#### Scenario: EditState importable from crate root
- **WHEN** caller writes `use rat_spreadsheet::EditState`
- **THEN** the import resolves successfully

#### Scenario: EditState not defined in render.rs
- **WHEN** `render.rs` is inspected
- **THEN** `EditState` is imported from `edit_state`, not defined inline

### Requirement: SpreadsheetState field grouping
`SpreadsheetState` SHALL organize its fields into documented groups: data model fields (`grid`, `dep_graph`, `fn_registry`, `validators`, `last_undo`, `edit`) and visual layout fields (`col_widths`, `default_col_width`, `min_col_width`, `frozen_rows`, `frozen_cols`, `style_callback`). The struct SHALL remain a single type (not split) to preserve `StatefulWidget` compatibility.

#### Scenario: Fields grouped with doc comments
- **WHEN** `SpreadsheetState` definition is inspected
- **THEN** fields are organized under `// -- Data model --` and `// -- Visual layout --` comment headers (or equivalent doc-comment grouping)

#### Scenario: StatefulWidget still works
- **WHEN** `Spreadsheet` implements `StatefulWidget` with `State = SpreadsheetState`
- **THEN** `frame.render_stateful_widget(spreadsheet, area, &mut state)` compiles and works
