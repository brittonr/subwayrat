//! Basic example demonstrating the rat-branches API

use rat_branches::tree::{find_leaves, walk_to_root, TreeNode};
use rat_branches::compare::{compare_branches, CompareBlock, truncate_first_line};

#[derive(Debug, Clone)]
struct ExampleNode {
    id: usize,
    parent: Option<usize>,
    content: String,
    tokens: usize,
}

impl ExampleNode {
    fn new(id: usize, content: &str, parent: Option<usize>, tokens: usize) -> Self {
        Self {
            id,
            parent,
            content: content.to_string(),
            tokens,
        }
    }
}

impl TreeNode for ExampleNode {
    fn id(&self) -> usize {
        self.id
    }

    fn parent_id(&self) -> Option<usize> {
        self.parent
    }
}

fn node_to_compare_block(node: &ExampleNode) -> CompareBlock {
    CompareBlock::new(
        node.id(),
        truncate_first_line(&node.content, 50),
        node.tokens,
    )
    .add_detail_count("messages", 1)
}

fn main() {
    // Create a simple tree: root -> branch_a, branch_b
    let nodes = vec![
        ExampleNode::new(0, "What is the meaning of life?", None, 100),
        ExampleNode::new(1, "The meaning is to find happiness", Some(0), 200),
        ExampleNode::new(2, "The meaning is to help others", Some(0), 180),
    ];

    // Find leaves
    let leaves = find_leaves(&nodes);
    println!("Leaves: {:?}", leaves);

    // Walk to root from a leaf
    let path = walk_to_root(1, &nodes);
    println!("Path from leaf 1 to root: {:?}", path);

    // Compare branches
    if let Some(comparison) = compare_branches(1, 2, &nodes, node_to_compare_block) {
        println!("Branch comparison:");
        println!("  Divergence at: {:?}", comparison.divergence_id);
        println!("  Branch A has {} unique nodes", comparison.branch_a.len());
        println!("  Branch B has {} unique nodes", comparison.branch_b.len());
        println!("  Tokens: A={}, B={}", comparison.tokens_a, comparison.tokens_b);
    }
}