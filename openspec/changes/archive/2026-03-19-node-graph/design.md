## Context

subwayrat is a workspace of ratatui widget crates. `rat-canvas` already provides
infinite-canvas viewport math (pan/zoom, screen↔canvas coordinate mapping).
`rat-layers` provides ordered stacks with visibility/lock. The workspace has no
graph data structure or visual wiring widget.

Node-graph UIs (Node-RED, n8n, Blender nodes, Unreal Blueprints) share a common
core: boxes with typed pins, connected by directed edges, laid out on a
pannable canvas. The rendering medium here is a terminal cell grid, not GPU
pixels, so edge routing uses ASCII/Unicode box-drawing characters rather than
Bézier curves.

## Goals / Non-Goals

**Goals:**
- Reusable graph data model independent of rendering (pure Rust structs, no
  ratatui dependency in the model layer).
- Ratatui `StatefulWidget` that renders the graph onto a `rat-canvas` viewport.
- Interaction primitives: select, drag, wire, delete — exposed as an event
  enum the caller matches on, not baked into a monolithic event loop.
- Serde round-trip for the full graph (nodes, ports, edges, positions).
- Pluggable port type system — callers define their own type tags and
  compatibility rules.

**Non-Goals:**
- Execution engine — this crate draws graphs, it does not evaluate them.
  No scheduler, no data propagation, no runtime.
- Undo/redo — callers bring their own. The data model is Clone-friendly to
  support snapshot-based undo.
- Real-time collaboration / CRDT merging — out of scope for v1.
- GPU-accelerated rendering or mouse-pixel subgrid precision.

## Decisions

### 1. Separate model and view layers

**Decision**: Split into two modules — `model` (graph data) and `view`
(ratatui rendering + interaction).

**Rationale**: Keeps the graph testable without a terminal. Callers who only
need the data structure (e.g., headless validation, serialization) don't pull
in ratatui. Follows the pattern set by `rat-canvas` (pure math, no ratatui
dep) vs. application code that imports both.

**Alternatives**:
- Single flat module — rejected because it couples graph logic to rendering.

### 2. Typed ports with string tags

**Decision**: Ports carry a `type_tag: String` field. Callers register a
`CompatibilityFn: Fn(&str, &str) -> bool` to decide if two ports can connect.
Default: same-string match.

**Rationale**: Generic over caller-defined type systems without trait objects
or complicated generics. n8n-style "any" ports just use a wildcard tag.

**Alternatives**:
- Generic `<T: PortType>` — rejected because it makes the graph generic over
  the type system, complicating serialization and widget signatures.
- Enum of built-in types — rejected because it's not extensible.

### 3. Edge routing via Manhattan paths

**Decision**: Edges are drawn as horizontal-then-vertical (or vertical-then-
horizontal) segments using Unicode box-drawing characters (`─`, `│`, `┐`,
`└`, etc.). No diagonal lines.

**Rationale**: Terminal cells are rectangular, not square. Diagonal lines look
bad in most fonts. Manhattan routing is simple to implement, readable at any
zoom, and matches what terminal users expect.

**Alternatives**:
- Braille-character diagonal lines — looked at, poor readability.
- Straight horizontal only (source right → target left) — too restrictive
  when nodes aren't horizontally aligned.

### 4. Node identity via u64 IDs

**Decision**: Nodes and ports use `NodeId(u64)` and `PortId(u64)` with a
monotonic counter in the graph. Not UUIDs.

**Rationale**: Simpler, faster, smaller. UUIDs are needed when merging
concurrent edits (CRDTs) — explicitly a non-goal. The counter resets on
deserialization by scanning for the max existing ID.

**Alternatives**:
- UUID like `rat-layers::LayerId` — overkill without CRDT requirements.
- Caller-provided generic ID — adds type parameter noise everywhere.

### 5. Layout as a separate pass

**Decision**: `layout::auto_layout(graph, direction)` takes a `&mut Graph`
and assigns positions. It's a function, not embedded in the widget.

**Rationale**: Callers may want manual-only positioning, or their own layout
algorithm. Keeping it as an opt-in function means the widget never moves
nodes behind the caller's back.

**Alternatives**:
- Layout built into the widget render — rejected, too opaque.

### 6. Interaction via returned events, not callbacks

**Decision**: The widget's `handle_input` method returns a
`Vec<GraphAction>` enum (NodeMoved, EdgeCreated, SelectionChanged, etc.).
The caller decides what to do.

**Rationale**: Matches ratatui's immediate-mode philosophy. No closures, no
lifetimes, no `Arc<Mutex<_>>`. Callers pattern-match on actions. Works
naturally with `rat-keymap` for key binding customization.

**Alternatives**:
- Callback-based (on_edge_created, on_node_moved) — rejected, lifetime hell
  in ratatui's render loop.

## Risks / Trade-offs

- **Edge overlap**: Manhattan routing with many edges will produce overlapping
  lines. Mitigation: edges use different Unicode styles (thin/thick/double)
  per type-tag, and a future v2 could add edge bundling or offset routing.

- **Performance at scale**: Rendering hundreds of nodes per frame could get
  slow. Mitigation: viewport culling — only render nodes/edges that
  intersect the visible area. The `rat-canvas` viewport already provides
  the bounding box.

- **No undo built in**: Callers must implement their own undo stack.
  Mitigation: `Graph` derives `Clone`, so snapshot-based undo is trivial
  (`let snapshot = graph.clone()`).

- **Terminal mouse support varies**: Not all terminals report mouse drag
  events. Mitigation: keyboard-only interaction path (arrow keys to select
  nodes, Enter to start wiring, Tab to cycle ports). Mouse is optional.
