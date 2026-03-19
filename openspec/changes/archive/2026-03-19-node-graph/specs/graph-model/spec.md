## ADDED Requirements

### Requirement: Graph contains nodes with typed ports
The graph SHALL store nodes, where each node has a unique `NodeId(u64)`, a
user-defined label, a position `(i32, i32)`, and an ordered list of input
ports and output ports. Each port SHALL have a unique `PortId(u64)`, a label,
a `type_tag: String`, and a direction (input or output).

#### Scenario: Create a node with ports
- **WHEN** caller adds a node with label "HTTP Request", two input ports
  ("url", "headers") and one output port ("response")
- **THEN** the graph contains one node with `NodeId` assigned, three ports
  with distinct `PortId`s, and the node's input/output port lists match the
  provided order

#### Scenario: Node IDs are monotonically unique
- **WHEN** caller adds three nodes sequentially
- **THEN** each node receives a distinct `NodeId` and no two are equal

### Requirement: Directed edges connect output ports to input ports
The graph SHALL support directed edges from an output port on one node to an
input port on another node. Each edge SHALL reference the source `PortId` and
target `PortId`. Self-loops (source and target on the same node) SHALL be
rejected.

#### Scenario: Create a valid edge
- **WHEN** caller connects output port A on node 1 to input port B on node 2
- **THEN** the graph contains one edge from A to B, and querying edges for
  node 1 returns one outgoing edge, and querying edges for node 2 returns
  one incoming edge

#### Scenario: Reject self-loop
- **WHEN** caller connects an output port to an input port on the same node
- **THEN** the graph returns an error and no edge is created

### Requirement: Port type compatibility checking
The graph SHALL accept a compatibility function
`Fn(&str, &str) -> bool` that determines whether an output type_tag can
connect to an input type_tag. Edge creation SHALL fail if the compatibility
check returns false. The default compatibility function SHALL require exact
string match.

#### Scenario: Compatible types connect
- **WHEN** output port has type_tag "string" and input port has type_tag
  "string" using default compatibility
- **THEN** edge creation succeeds

#### Scenario: Incompatible types rejected
- **WHEN** output port has type_tag "number" and input port has type_tag
  "string" using default compatibility
- **THEN** edge creation returns an error and no edge is created

#### Scenario: Custom compatibility function
- **WHEN** caller provides a function that allows "any" to connect to
  anything, and output has type_tag "any" connecting to input "string"
- **THEN** edge creation succeeds

### Requirement: Optional DAG cycle detection
The graph SHALL support an optional DAG mode where edge creation fails if it
would introduce a cycle. When DAG mode is disabled, cycles SHALL be permitted.

#### Scenario: Cycle rejected in DAG mode
- **WHEN** DAG mode is enabled and nodes A→B→C exist, and caller tries to
  add edge C→A
- **THEN** edge creation returns a cycle error and no edge is created

#### Scenario: Cycle allowed when DAG mode disabled
- **WHEN** DAG mode is disabled and nodes A→B→C exist, and caller adds
  edge C→A
- **THEN** edge creation succeeds and the graph contains the cycle

### Requirement: Node and edge removal with cascading cleanup
Removing a node SHALL remove all edges connected to any of its ports.
Removing an edge SHALL only remove that edge.

#### Scenario: Remove node cascades to edges
- **WHEN** node A has edges to nodes B and C, and caller removes node A
- **THEN** node A and all edges involving A's ports are removed; nodes B
  and C remain with no dangling edge references

#### Scenario: Remove single edge
- **WHEN** caller removes edge from port X to port Y
- **THEN** only that edge is removed; both nodes and all other edges remain

### Requirement: Serde serialization round-trip
The full graph (nodes, ports, edges, positions) SHALL serialize to JSON via
serde and deserialize back to an equivalent graph. After deserialization, the
ID counter SHALL be restored to one past the maximum existing ID.

#### Scenario: Serialize and deserialize preserves structure
- **WHEN** a graph with 3 nodes and 4 edges is serialized to JSON then
  deserialized
- **THEN** the deserialized graph has identical node count, edge count, port
  connections, positions, labels, and type_tags

#### Scenario: ID counter restored after deserialization
- **WHEN** a graph with max NodeId(5) is deserialized and a new node is added
- **THEN** the new node receives NodeId(6) or higher, not a duplicate
