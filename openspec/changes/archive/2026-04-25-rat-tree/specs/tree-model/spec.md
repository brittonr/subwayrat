## ADDED Requirements

### Requirement: TreeData trait provides top-down traversal
The crate SHALL define a `TreeData` trait with methods to enumerate roots, enumerate children of a node, and retrieve a node's display label. Consumers implement this trait for their data structure.

#### Scenario: Trait has required methods
- **WHEN** a consumer implements `TreeData`
- **THEN** they SHALL provide `root_count`, `root(index)`, `child_count(node)`, `child(node, index)`, and `node_label(node)` methods.

#### Scenario: Optional icon method
- **WHEN** a consumer does not override `node_icon`
- **THEN** the default implementation SHALL return `None`.

### Requirement: Visible rows computed from expand state
The system SHALL compute a flat list of visible rows by walking the tree top-down, skipping children of collapsed nodes. Each visible row SHALL record the node id, depth level, whether the node has children, and whether it is expanded.

#### Scenario: All nodes collapsed
- **WHEN** no nodes are in the expanded set
- **THEN** visible rows SHALL contain only root nodes.

#### Scenario: Expanding a node
- **WHEN** a node with children is expanded
- **THEN** its direct children SHALL appear as visible rows immediately after the parent, at depth + 1.

#### Scenario: Nested expansion
- **WHEN** a node is expanded and one of its children is also expanded
- **THEN** the grandchildren SHALL appear at depth + 2, immediately after their parent.

#### Scenario: Collapsing hides descendants
- **WHEN** an expanded node is collapsed
- **THEN** all its descendants SHALL be removed from visible rows, regardless of their own expand state.

### Requirement: Expand state uses BTreeSet
The expanded node set SHALL be stored as a `BTreeSet<usize>` to guarantee deterministic iteration order.

#### Scenario: Deterministic ordering
- **WHEN** multiple nodes are expanded and the set is iterated
- **THEN** node IDs SHALL appear in ascending order.

### Requirement: SimpleTree adapter
The crate SHALL provide a `SimpleTree` struct that implements `TreeData` from a flat list of `(id, parent_id, label)` tuples, suitable for consumers with parent-pointer data.

#### Scenario: Building from flat list
- **WHEN** a `SimpleTree` is constructed from `[(0, None, "root"), (1, Some(0), "child")]`
- **THEN** `root_count()` SHALL return 1, `child_count(0)` SHALL return 1, and `node_label(1)` SHALL return `"child"`.

#### Scenario: Multiple roots
- **WHEN** two entries have `parent_id = None`
- **THEN** `root_count()` SHALL return 2 and both SHALL be accessible via `root(0)` and `root(1)`.
