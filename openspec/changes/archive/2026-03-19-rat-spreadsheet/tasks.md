## 1. Crate Setup

- [x] 1.1 Create `crates/rat-spreadsheet` directory with Cargo.toml (ratatui workspace dep)
- [x] 1.2 Add `rat-spreadsheet` to workspace members in root Cargo.toml
- [x] 1.3 Set up module structure: `lib.rs`, `cell.rs`, `formula.rs`, `nav.rs`, `render.rs`

## 2. Cell Model

- [x] 2.1 Implement `CellAddr` with A1-style parsing, Display, and (col, row) construction
- [x] 2.2 Implement `CellRange` with start/end addresses and row-major iterator
- [x] 2.3 Implement `CellValue` enum (Empty, Text, Number, Boolean, Error, Formula)
- [x] 2.4 Implement `Grid` struct with `Vec<Vec<CellValue>>` storage, get/set, dynamic growth
- [x] 2.5 Write tests for cell addressing (A1, AA1, invalid input, roundtrip Display/Parse)
- [x] 2.6 Write tests for grid operations (get/set, growth, row_count/col_count)

## 3. Formula Engine

- [x] 3.1 Define formula AST types (Number, CellRef, Range, BinaryOp, FunctionCall)
- [x] 3.2 Implement tokenizer for formula expressions
- [x] 3.3 Implement recursive descent parser (precedence: parens > mul/div/mod > add/sub)
- [x] 3.4 Implement tree-walk evaluator with cell reference resolution
- [x] 3.5 Implement built-in functions: SUM, AVG, MIN, MAX, COUNT, IF
- [x] 3.6 Implement custom function registry (register by name, closure-based)
- [x] 3.7 Implement dependency graph (HashMap<CellAddr, Vec<CellAddr>>)
- [x] 3.8 Implement dirty-cell tracking and topological sort for recalculation
- [x] 3.9 Implement circular reference detection (#CYCLE! error)
- [x] 3.10 Write tests for parsing, evaluation, precedence, division by zero
- [x] 3.11 Write tests for dependency tracking, recalculation, cycle detection

## 4. Grid Navigation

- [x] 4.1 Implement cursor state (current position, anchor for selection)
- [x] 4.2 Implement arrow key movement with boundary clamping
- [x] 4.3 Implement Tab/Shift+Tab with row wrapping
- [x] 4.4 Implement Home/End/Ctrl+Home/Ctrl+End jump navigation
- [x] 4.5 Implement PageUp/PageDown with viewport-height steps
- [x] 4.6 Implement scroll-follows-cursor logic (minimum scroll to keep cursor visible)
- [x] 4.7 Implement mouse click to cell mapping (pixel coordinates to CellAddr)
- [x] 4.8 Implement Shift+arrow and Shift+click range selection
- [x] 4.9 Write tests for navigation edge cases (boundaries, wrapping, page clamp)

## 5. Cell Editing

- [x] 5.1 Implement edit state (editing flag, buffer, cursor position within buffer)
- [x] 5.2 Implement enter-edit-mode (Enter key and character typing triggers)
- [x] 5.3 Implement edit buffer manipulation (insert, delete, cursor movement within buffer)
- [x] 5.4 Implement commit logic (parse "=" prefix -> formula, try f64 -> number, else text)
- [x] 5.5 Implement cancel (Escape restores previous value)
- [x] 5.6 Implement per-column validation callbacks
- [x] 5.7 Implement single-level undo (store previous value, Ctrl+Z restore)
- [x] 5.8 Write tests for editing lifecycle (enter, type, commit, cancel, undo)

## 6. Rendering

- [x] 6.1 Define `Spreadsheet` widget struct and `SpreadsheetState` state struct
- [x] 6.2 Implement `StatefulWidget` for `Spreadsheet`
- [x] 6.3 Implement grid layout calculation (column headers, row numbers, cell rects)
- [x] 6.4 Implement cell content rendering (alignment by type, error display, truncation)
- [x] 6.5 Implement cursor and selection highlight rendering
- [x] 6.6 Implement edit mode rendering (inline buffer with text cursor)
- [x] 6.7 Implement column width configuration (default, per-column override, minimum 3)
- [x] 6.8 Implement frozen rows/columns rendering
- [x] 6.9 Implement cell style callback (per-cell conditional styling)
- [x] 6.10 Define `SpreadsheetStyle` config struct (cursor, selection, header, cell defaults)

## 7. Integration and Polish

- [x] 7.1 Wire navigation, editing, and formula recalc into a unified event handler
- [x] 7.2 Implement copy/paste for selected cell ranges
- [x] 7.3 Add public API surface: re-exports in lib.rs, builder methods on widget/state
- [x] 7.4 Write integration tests (edit cell -> formula recalc -> render cycle)
- [x] 7.5 Add doc comments and module-level documentation
- [x] 7.6 Verify all spec scenarios are covered by tests
