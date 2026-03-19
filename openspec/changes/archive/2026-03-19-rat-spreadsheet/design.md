## Context

The subwayrat workspace contains 14 ratatui widget crates. `rat-table` handles read-only tabular display with row selection and column scrolling. `rat-editor` handles text editing. `rat-selection` provides selection models. There is no editable grid component.

The spreadsheet widget needs to compose well with existing crates. It should follow the same patterns: state struct separated from widget, builder-style configuration, `StatefulWidget` rendering.

Ratatui's rendering model is immediate-mode -- the entire visible area redraws each frame. The spreadsheet must track its own state (cell values, cursor position, edit mode) across frames.

## Goals / Non-Goals

**Goals:**
- Editable grid with cell-level cursor movement and inline editing
- A1-style cell addressing and basic formula evaluation
- Column/row headers, resizing, and frozen panes
- Cell range selection, copy/paste
- Pluggable cell validation
- Works as a drop-in ratatui widget (StatefulWidget impl)

**Non-Goals:**
- Full Excel/Sheets compatibility (no macros, charts, pivot tables)
- File format I/O (CSV/XLSX parsing belongs in the consumer, not the widget)
- Multi-sheet/tab support in v1
- Collaborative editing or conflict resolution
- Undo history beyond single-cell level in v1 (undo restores the previous cell value, not arbitrary depth)

## Decisions

### 1. Separate state from widget

The `Spreadsheet` struct is the widget (implements `StatefulWidget`). `SpreadsheetState` holds all mutable state: grid data, cursor position, selection, scroll offset, edit buffer.

*Rationale*: Matches ratatui convention (`Table`/`TableState`, `List`/`ListState`). Lets the application own the state and pass the widget as a temporary for rendering.

*Alternative*: Single struct with `render(&self, ...)`. Rejected because it fights ratatui's ownership model.

### 2. Cell storage: `Vec<Vec<CellValue>>` with HashMap overlay for formulas

The grid stores cell values in a row-major `Vec<Vec<CellValue>>`. Formula cells store their expression string alongside a cached computed value. A dependency graph (adjacency list in a `HashMap<CellAddr, Vec<CellAddr>>`) tracks which cells depend on which, for recalculation.

*Rationale*: Row-major vec is cache-friendly for row-oriented rendering. The dependency graph is only needed for formula cells, so a separate sparse structure avoids bloating every cell.

*Alternative*: A flat `HashMap<CellAddr, CellValue>` for sparse grids. Rejected for v1 -- most spreadsheet use cases fill a contiguous region, and vec indexing is faster than hashing for the common case.

### 3. Formula engine: tree-walk interpreter, no bytecode

Formulas are parsed into an AST and evaluated by walking the tree. No compilation step.

*Rationale*: Keeps the implementation simple. Spreadsheet formulas are short expressions, not programs. Tree-walk evaluation is fast enough for thousands of cells with simple formulas. Bytecode compilation is premature optimization for a TUI widget.

*Alternative*: Stack-based bytecode VM. Would be faster for complex sheets but adds significant complexity for marginal gain in the TUI context.

### 4. Recalculation: topological sort of dirty cells

When a cell value changes, mark all dependents as dirty. Topological sort the dirty set. Evaluate in order. Detect cycles and mark them as `#CYCLE!` errors.

*Rationale*: Correct ordering with cycle detection. Avoids recomputing the entire sheet on every edit.

*Alternative*: Full sheet recalc on every change. Simpler but O(n) per edit where n = total formula cells.

### 5. Selection model: internal, not rat-selection

The spreadsheet has its own 2D selection model (single cell, rectangular range, multi-range via Ctrl+click). This differs from `rat-selection` which is 1D (list/text oriented).

*Rationale*: 2D rectangular selection doesn't map cleanly onto `rat-selection`'s linear model. The spreadsheet needs range expressions (A1:B5) that integrate with the formula engine.

*Alternative*: Wrap `rat-selection` with a 2D adapter. Adds a layer of indirection for no clear benefit since the selection semantics are fundamentally different.

## Risks / Trade-offs

- [Large API surface] The spreadsheet touches navigation, editing, formulas, rendering, and selection. Risk of a sprawling public API. -> Mitigation: Group related types into submodules (`cell`, `formula`, `nav`). Keep the top-level API to `Spreadsheet`, `SpreadsheetState`, and a few config structs.

- [Formula complexity creep] Users will want more functions, nested formulas, string operations. -> Mitigation: Define a small core set of functions (SUM, AVG, MIN, MAX, COUNT, IF) and make the function registry extensible. Users can register custom functions.

- [Performance with large grids] A 10,000-row grid with formulas could lag on recalc. -> Mitigation: Dirty-cell tracking limits recalc scope. Rendering only processes visible cells. Profile before optimizing further.

- [Cycle detection cost] Topological sort on every edit adds overhead. -> Mitigation: Only sort the dirty subgraph, not the entire dependency graph. Most edits affect few dependents.
