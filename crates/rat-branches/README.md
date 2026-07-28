# rat-branches

`rat-branches` provides generic tree algorithms and comparison widgets for Ratatui.

The code comes from the Clankers TUI conversation-branch system. It supports other tree-shaped data through the `TreeNode` trait.

## Features

- **Generic tree algorithms**: TreeNode trait with algorithms for walking paths, finding divergences, counting children, and discovering leaves
- **Branch comparison**: Compare two tree branches to find their differences and unique nodes
- **TUI components**: Ready-to-use ratatui widgets for branch comparison overlays and node switchers
- **Flexible design**: Generic over any tree node type implementing the TreeNode trait

## Usage

### Tree algorithms

```rust
use rat_branches::tree::{TreeNode, walk_to_root, find_leaves, find_divergence};

#[derive(Debug)]
struct MyNode {
    id: usize,
    parent: Option<usize>,
    content: String,
}

impl TreeNode for MyNode {
    fn id(&self) -> usize { self.id }
    fn parent_id(&self) -> Option<usize> { self.parent }
}

let nodes = vec![/* your tree nodes */];

// Find all leaf nodes
let leaves = find_leaves(&nodes);

// Walk from leaf to root
let path = walk_to_root(leaf_id, &nodes);

// Find where a branch diverges from siblings
let divergence = find_divergence(leaf_id, &nodes);
```

### Branch comparison

```rust
use rat_branches::compare::{compare_branches, CompareBlock, truncate_first_line};

fn node_to_compare_block(node: &MyNode) -> CompareBlock {
    CompareBlock::new(
        node.id(),
        truncate_first_line(&node.content, 50),
        node.content.len(), // token count or similar metric
    )
    .add_detail_count("messages", 1)
}

// Compare two branches
if let Some(comparison) = compare_branches(leaf_a, leaf_b, &nodes, node_to_compare_block) {
    println!("Branches diverge at: {:?}", comparison.divergence_id);
    println!("Branch A has {} unique nodes", comparison.branch_a.len());
    println!("Branch B has {} unique nodes", comparison.branch_b.len());
}
```

### TUI components

```rust
use rat_branches::{BranchCompareView, NodeSwitcher};
use ratatui::{Frame, layout::Rect};

// Branch comparison overlay
let mut compare_view = BranchCompareView::new();
compare_view.open(comparison); // BranchComparison from compare_branches
compare_view.render(frame, area);

// Node switcher with filtering
let mut switcher = NodeSwitcher::new();
switcher.open(&nodes, |node, path| {
    SwitcherItem::new(node.id(), format!("Node {}", node.id()), node.content.clone(), false)
        .add_metadata("depth", path.len())
});
switcher.render(frame, area);
```

## API overview

### Core traits

- `TreeNode`: Minimal trait for tree nodes with ID and parent relationships

### Tree algorithms in `tree`

- `walk_to_root()`: Walk from leaf to root, returning path
- `find_divergence()`: Find where a branch splits from siblings  
- `find_leaves()`: Find all leaf nodes
- `count_children()`: Count direct children of a node

### Comparison in `compare`

- `CompareBlock`: Node summary for comparison display
- `BranchComparison`: Result of comparing two branches
- `compare_branches()`: Core comparison algorithm

### TUI components

- `BranchCompareView`: Side-by-side branch comparison overlay
- `NodeSwitcher`: Filtered node picker with search

## License

MIT OR Apache-2.0