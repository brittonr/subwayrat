## ADDED Requirements

### Requirement: Auto-layout with configurable direction
The layout engine SHALL position all nodes in a directed graph using a
layered (Sugiyama-style) algorithm. The primary direction SHALL be
configurable: left-to-right or top-to-bottom. Nodes with no incoming edges
SHALL be placed in the first rank. Nodes SHALL be spaced with minimum
horizontal and vertical gaps.

#### Scenario: Left-to-right layout
- **WHEN** auto_layout is called with direction LeftToRight on a graph
  A→B→C
- **THEN** node A is placed leftmost, B in the middle, C rightmost, with
  consistent vertical centering and minimum horizontal spacing between them

#### Scenario: Top-to-bottom layout
- **WHEN** auto_layout is called with direction TopToBottom on a graph A→B→C
- **THEN** node A is placed topmost, B in the middle, C bottommost, with
  consistent horizontal centering and minimum vertical spacing

#### Scenario: Multiple root nodes
- **WHEN** auto_layout is called on a graph with two disconnected subgraphs
  A→B and C→D
- **THEN** both subgraphs are laid out without overlapping

### Requirement: Snap-to-grid positioning
The layout engine SHALL support an optional snap grid with configurable cell
size. When snapping is enabled, node positions SHALL be rounded to the
nearest grid cell boundary after any move or layout operation.

#### Scenario: Snap after drag
- **WHEN** grid size is 4 and a node is moved to position (13, 7)
- **THEN** the node's position snaps to (12, 8)

#### Scenario: Snap after auto-layout
- **WHEN** grid size is 4 and auto-layout places a node at (11, 5)
- **THEN** the node's final position is (12, 4)

#### Scenario: Snap disabled
- **WHEN** snapping is disabled and a node is moved to (13, 7)
- **THEN** the node remains at (13, 7)

### Requirement: Minimum spacing constraints
Auto-layout SHALL enforce minimum horizontal spacing (columns between node
right edge and next node left edge) and minimum vertical spacing (rows
between node bottom and next node top). Defaults SHALL be provided.
Callers SHALL be able to override the spacing values.

#### Scenario: Default spacing respected
- **WHEN** auto-layout runs with default spacing on a linear graph A→B
- **THEN** the gap between A's right edge and B's left edge is at least
  the default horizontal spacing

#### Scenario: Custom spacing
- **WHEN** caller sets minimum horizontal spacing to 10 and runs auto-layout
  on A→B
- **THEN** the gap between A's right edge and B's left edge is at least 10
  columns

### Requirement: Layout preserves manual positions when requested
The layout engine SHALL support a partial layout mode where only unpositioned
nodes (position == None or a sentinel) are laid out, while manually-placed
nodes keep their positions. Edges to/from manual nodes SHALL be considered
for rank assignment.

#### Scenario: Partial layout skips pinned nodes
- **WHEN** node A is pinned at (5, 5), node B is unpositioned, and A→B
  exists
- **THEN** after partial layout, A remains at (5, 5) and B is placed to
  the right of A respecting spacing constraints
