## Why

The workspace has `rat-table` for read-only tabular data display, but no editable grid. Anyone building a TUI that needs cell-level editing (config editors, data entry, CSV manipulation) has to wire up their own cell navigation, editing lifecycle, and formula evaluation on top of raw ratatui primitives. A dedicated spreadsheet widget fills that gap.

## What Changes

- New `rat-spreadsheet` crate providing an editable grid/spreadsheet widget
- Cell-level navigation (arrow keys, tab, home/end, page up/down)
- Inline cell editing with configurable input validation
- Cell types: text, numeric, boolean, computed (formula)
- Column and row resizing, insertion, deletion
- Copy/paste for cell ranges
- Formula support with cell references (e.g., `=A1+B2`, `=SUM(A1:A10)`)
- Selection model for single cells, ranges, and multi-select (builds on `rat-selection`)
- Customizable styling per cell, row, column, or conditional rules

## Capabilities

### New Capabilities
- `cell-model`: Cell data types, storage, addressing (A1-style and R1C1), and the underlying grid data structure
- `cell-editing`: Inline editing lifecycle (enter edit mode, validate input, commit/cancel), input types, and undo/redo
- `grid-navigation`: Keyboard and mouse navigation across cells, scroll behavior, and selection mechanics
- `formula-engine`: Expression parsing, cell reference resolution, dependency tracking, and recalculation
- `grid-rendering`: Rendering the grid to a ratatui frame, column/row headers, frozen panes, and cell styling

### Modified Capabilities

(none -- no existing specs affected)

## Impact

- New crate `crates/rat-spreadsheet` added to workspace
- Depends on `ratatui` (workspace dep), likely `rat-selection` for selection model
- Public API: `Spreadsheet` widget struct, `SpreadsheetState`, `Cell`, `CellValue`, `Formula` types
- No changes to existing crates
