## ADDED Requirements

### Requirement: View tree types
The `ratcore::inline` module SHALL define `ViewTree` and `ViewNode` types. A `ViewNode` SHALL carry an optional string key, a `TypeId` for type-based matching, and an opaque `Box<dyn Any>` state slot. These types SHALL have no dependencies outside `std`.

#### Scenario: Node with key
- **WHEN** a `ViewNode` is created with `key: Some("msg-0".into())` and a `TypeId`
- **THEN** the node's key is `Some("msg-0")` and its type_tag matches the given type

#### Scenario: Node without key
- **WHEN** a `ViewNode` is created with `key: None`
- **THEN** the node has no key and is matched by position during reconciliation

### Requirement: Key-based matching
The reconciler SHALL match new nodes to existing nodes by key first. A new node with `key: "X"` SHALL match the existing node with `key: "X"` regardless of position. Matched nodes SHALL preserve their opaque state blob.

#### Scenario: Reordered keyed nodes
- **WHEN** frame 1 has nodes [key="a", key="b"] and frame 2 has [key="b", key="a"]
- **THEN** each node retains its state from frame 1

#### Scenario: Keyed node removed
- **WHEN** frame 1 has [key="a", key="b"] and frame 2 has [key="a"]
- **THEN** node "b" is dropped and its state is released

### Requirement: Positional matching
After key-based matching, the reconciler SHALL match remaining unkeyed nodes by position and type_tag. Node at position N of type T in the new tree matches node at position N of type T in the old tree.

#### Scenario: Stable positional nodes
- **WHEN** frame 1 has [TypeA, TypeB] (unkeyed) and frame 2 has [TypeA, TypeB] (unkeyed)
- **THEN** each node matches by position and preserves state

#### Scenario: Type mismatch at position
- **WHEN** frame 1 has [TypeA] at position 0 and frame 2 has [TypeB] at position 0
- **THEN** the TypeA node is dropped and a new TypeB node is created

### Requirement: New node creation
Unmatched new nodes SHALL be created with a `None` state slot. The backend is responsible for initializing state on first render.

#### Scenario: Appended node
- **WHEN** frame 1 has [key="a"] and frame 2 has [key="a", key="b"]
- **THEN** node "a" preserves state, node "b" has `None` state

### Requirement: Pure function interface
The reconciler SHALL be a pure function: `reconcile(old: &[ViewNode], new: Vec<ViewNode>) -> Vec<ViewNode>`. It SHALL have no side effects, no framework dependencies, and no I/O.

#### Scenario: Deterministic output
- **WHEN** `reconcile` is called with the same inputs twice
- **THEN** both calls produce identical output

### Requirement: Single-pass O(N) reconciliation
The reconciler SHALL complete in a single pass over the new node list using a hash map for key lookups. Time complexity SHALL be O(N) where N is the number of nodes.

#### Scenario: Performance with 1000 nodes
- **WHEN** reconciling a tree of 1000 keyed nodes
- **THEN** reconciliation completes in O(N) time without nested iteration

### Requirement: Commit tracking
The `ratcore::inline` module SHALL provide a `compute_commits(node_heights: &[u16], viewport_height: u16) -> Vec<usize>` function that returns indices of nodes whose rows are entirely above the viewport. This is a pure function with no terminal dependencies.

#### Scenario: Nodes above viewport
- **WHEN** nodes have heights [5, 5, 5, 5] and viewport_height is 10
- **THEN** the first two nodes (indices 0, 1) are returned as committed if total height exceeds viewport + those nodes
