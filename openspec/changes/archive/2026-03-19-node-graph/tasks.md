## 1. Crate scaffolding

- [x] 1.1 Create `crates/rat-nodegraph/` with Cargo.toml (deps: `rat-canvas`, `ratatui`, optional `serde`)
- [x] 1.2 Add `rat-nodegraph` to workspace members in root Cargo.toml
- [x] 1.3 Create `src/lib.rs` with module declarations (`model`, `view`, `layout`)
- [x] 1.4 Verify `cargo check -p rat-nodegraph` passes with empty modules

## 2. Graph data model (`model` module)

- [x] 2.1 Define `NodeId(u64)`, `PortId(u64)` newtypes with Debug/Clone/Copy/Eq/Hash
- [x] 2.2 Define `PortDirection` enum (Input, Output)
- [x] 2.3 Define `Port` struct: id, label, type_tag, direction
- [x] 2.4 Define `Node` struct: id, label, position (i32, i32), input_ports, output_ports
- [x] 2.5 Define `Edge` struct: source PortId, target PortId
- [x] 2.6 Define `Graph` struct with node storage, edge storage, ID counter, dag_mode flag
- [x] 2.7 Implement `Graph::add_node` — takes label + port specs, assigns IDs, returns NodeId
- [x] 2.8 Implement `Graph::remove_node` — removes node and all connected edges
- [x] 2.9 Implement `Graph::add_edge` — validates direction, self-loop, type compat, optional DAG cycle check
- [x] 2.10 Implement `Graph::remove_edge` — removes single edge
- [x] 2.11 Implement cycle detection (DFS from target to source) for DAG mode
- [x] 2.12 Implement default compatibility function (exact string match)
- [x] 2.13 Implement custom compatibility function setter
- [x] 2.14 Add query methods: `nodes()`, `edges()`, `edges_for_node()`, `node()`, `port()`
- [x] 2.15 Add serde derives behind `serde` feature flag
- [x] 2.16 Implement ID counter restoration on deserialization
- [x] 2.17 Write tests for all graph-model spec scenarios

## 3. Node rendering (`view` module)

- [x] 3.1 Define `NodeGraphState` struct (graph, viewport, selection set, interaction state)
- [x] 3.2 Define `NodeGraphWidget` struct with style/color configuration
- [x] 3.3 Implement node bounding box calculation (width from label/port lengths, height from port count)
- [x] 3.4 Implement single-node rendering: border, label header, port rows with markers
- [x] 3.5 Implement selected-node border style (double-line or color change)
- [x] 3.6 Implement port type color mapping (default palette + custom Fn)
- [x] 3.7 Implement viewport culling — skip nodes outside visible area
- [x] 3.8 Implement Manhattan edge routing: compute path segments between port positions
- [x] 3.9 Implement edge rendering with box-drawing characters and per-type coloring
- [x] 3.10 Implement wiring preview line (source port to cursor position)
- [x] 3.11 Implement box-selection rectangle overlay
- [x] 3.12 Implement `StatefulWidget` trait for `NodeGraphWidget`
- [x] 3.13 Write tests for bounding box calculation and viewport culling logic

## 4. Interaction handling (`view` module)

- [x] 4.1 Define `GraphAction` enum: SelectionChanged, NodeMoved, EdgeCreated, EdgeDeleted, WiringStarted, WiringCancelled
- [x] 4.2 Implement `handle_mouse` — dispatch clicks/drags to selection, dragging, wiring, box-select
- [x] 4.3 Implement hit testing: point-in-node, point-on-port, point-on-edge
- [x] 4.4 Implement single-click node selection and shift-click multi-select
- [x] 4.5 Implement click-empty-space to clear selection
- [x] 4.6 Implement mouse drag for node movement (single and multi)
- [x] 4.7 Implement port-click wiring start and port-click wiring complete (with compat check)
- [x] 4.8 Implement Escape to cancel wiring
- [x] 4.9 Implement Delete/Backspace for edge deletion
- [x] 4.10 Implement box-selection drag on empty canvas
- [x] 4.11 Implement `handle_key` — Tab for node focus cycling, arrows for nudge, Enter for wiring
- [x] 4.12 Implement keyboard-only wiring flow (Enter on port → Tab to cycle targets → Enter to confirm)
- [x] 4.13 Write tests for hit testing and action generation

## 5. Layout engine (`layout` module)

- [x] 5.1 Define `LayoutDirection` enum (LeftToRight, TopToBottom)
- [x] 5.2 Define `LayoutConfig` struct (direction, h_spacing, v_spacing, grid_size)
- [x] 5.3 Implement rank assignment (topological sort, assign depth per node)
- [x] 5.4 Implement node ordering within ranks to minimize edge crossings (barycenter heuristic)
- [x] 5.5 Implement position assignment from ranks + ordering + spacing
- [x] 5.6 Implement `auto_layout(graph, config)` public function
- [x] 5.7 Implement snap-to-grid: round positions to nearest grid boundary
- [x] 5.8 Implement partial layout mode (skip pinned/manually-placed nodes)
- [x] 5.9 Handle disconnected subgraphs (lay out each component, offset to avoid overlap)
- [x] 5.10 Write tests for all graph-layout spec scenarios

## 6. Integration and examples

- [x] 6.1 Create `examples/basic.rs` — builds a small graph, renders with auto-layout
- [x] 6.2 Create `examples/interactive.rs` — full event loop with mouse/keyboard interaction
- [x] 6.3 Add crate-level doc comment with usage example in `lib.rs`
- [x] 6.4 Run `cargo clippy -p rat-nodegraph` clean
- [x] 6.5 Run `cargo test -p rat-nodegraph` — all tests pass
