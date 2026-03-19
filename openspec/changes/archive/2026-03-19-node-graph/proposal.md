## Why

subwayrat has canvas (pan/zoom viewport), layers, and a rich set of widgets,
but no way to represent a directed graph of connected nodes — the kind of
interface that powers visual dataflow editors like Node-RED, n8n, Blender's
shader graph, and Unreal Blueprints. A `rat-nodegraph` crate fills that gap:
pure data model + ratatui rendering for nodes, ports, and edges on an infinite
canvas.

## What Changes

- Add `rat-nodegraph` crate to the workspace with:
  - **Graph data model**: Nodes with typed input/output ports, directed edges
    between ports, validation (no cycles in DAG mode, type compatibility).
  - **Layout engine**: Auto-layout (left-to-right, top-to-bottom) and
    manual node positioning. Snapping grid. Minimum spacing constraints.
  - **Canvas rendering**: Render nodes as bordered boxes with port rows,
    edges as ASCII/Unicode lines between ports. Uses `rat-canvas::Viewport`
    for pan/zoom coordinate mapping.
  - **Interaction model**: Node selection (single/multi), node dragging,
    edge creation (port-to-port), edge deletion, box selection.
  - **Port type system**: Ports carry a type tag (`&str`) for compatibility
    checking. Color coding per type. Input ports accept one or many
    connections (configurable).
  - **Serialization**: `serde` support for the full graph (nodes, edges,
    positions) so callers can persist and reload.

## Capabilities

### New Capabilities
- `graph-model`: Directed graph data structure with typed ports, edge validation, and cycle detection.
- `node-rendering`: Ratatui widget that draws nodes, ports, and edges on an infinite canvas.
- `graph-interaction`: Selection, dragging, edge creation/deletion, and box selection for the node graph.
- `graph-layout`: Auto-layout algorithms and snap-to-grid positioning for nodes.

### Modified Capabilities

## Impact

- **New crate**: `crates/rat-nodegraph/` added to workspace.
- **Dependencies**: Depends on `rat-canvas` (viewport/coordinate math) and
  `ratatui` (rendering). Optional `serde` feature for persistence.
- **No breaking changes**: Pure addition, no modifications to existing crates.
