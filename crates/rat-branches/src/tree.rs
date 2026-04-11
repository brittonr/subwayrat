//! Generic tree algorithms for nodes with ID and parent relationships

/// Trait for tree nodes with unique IDs and parent references
pub trait TreeNode {
    /// Unique identifier for this node
    fn id(&self) -> usize;
    /// Optional parent node ID (None for root nodes)
    fn parent_id(&self) -> Option<usize>;
}

/// Walk from a leaf node up to the root, returning the path as a list of node IDs.
/// Returns path in reverse order: [root, ..., parent, leaf]
pub fn walk_to_root<N: TreeNode>(leaf_id: usize, nodes: &[N]) -> Vec<usize> {
    let mut path = Vec::new();
    let mut current = Some(leaf_id);

    while let Some(id) = current {
        path.push(id);
        current = nodes
            .iter()
            .find(|n| n.id() == id)
            .and_then(|n| n.parent_id());
    }

    path.reverse();
    path
}

/// Find the block ID where this branch diverges from a sibling branch.
/// Returns the parent node ID that has multiple children.
pub fn find_divergence<N: TreeNode>(leaf_id: usize, nodes: &[N]) -> Option<usize> {
    let mut current = Some(leaf_id);

    while let Some(id) = current {
        let node = nodes.iter().find(|n| n.id() == id)?;
        if let Some(parent_id) = node.parent_id() {
            // Count siblings at this level
            let sibling_count = count_children(parent_id, nodes);
            if sibling_count > 1 {
                return Some(parent_id);
            }
        }
        current = node.parent_id();
    }

    None
}

/// Find all leaf node IDs (nodes with no children)
pub fn find_leaves<N: TreeNode>(nodes: &[N]) -> Vec<usize> {
    let has_children: std::collections::HashSet<usize> =
        nodes.iter().filter_map(|n| n.parent_id()).collect();

    nodes
        .iter()
        .map(|n| n.id())
        .filter(|&id| !has_children.contains(&id))
        .collect()
}

/// Count direct children of a parent node
pub fn count_children<N: TreeNode>(parent_id: usize, nodes: &[N]) -> usize {
    nodes
        .iter()
        .filter(|n| n.parent_id() == Some(parent_id))
        .count()
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestNode {
        id: usize,
        parent: Option<usize>,
    }

    impl TestNode {
        fn new(id: usize, parent: Option<usize>) -> Self {
            Self { id, parent }
        }
    }

    impl TreeNode for TestNode {
        fn id(&self) -> usize {
            self.id
        }

        fn parent_id(&self) -> Option<usize> {
            self.parent
        }
    }

    #[test]
    fn walk_to_root_linear() {
        let nodes = vec![
            TestNode::new(0, None),    // root
            TestNode::new(1, Some(0)), // child
            TestNode::new(2, Some(1)), // grandchild
        ];
        let path = walk_to_root(2, &nodes);
        assert_eq!(path, vec![0, 1, 2]);
    }

    #[test]
    fn walk_to_root_single() {
        let nodes = vec![TestNode::new(0, None)];
        let path = walk_to_root(0, &nodes);
        assert_eq!(path, vec![0]);
    }

    #[test]
    fn find_divergence_no_branches() {
        let nodes = vec![TestNode::new(0, None), TestNode::new(1, Some(0))];
        assert_eq!(find_divergence(1, &nodes), None);
    }

    #[test]
    fn find_divergence_with_fork() {
        let nodes = vec![
            TestNode::new(0, None),    // root
            TestNode::new(1, Some(0)), // branch-a
            TestNode::new(2, Some(0)), // branch-b
        ];
        // Both branches diverge at node 0
        assert_eq!(find_divergence(1, &nodes), Some(0));
        assert_eq!(find_divergence(2, &nodes), Some(0));
    }

    #[test]
    fn find_divergence_deep_fork() {
        let nodes = vec![
            TestNode::new(0, None),    // root
            TestNode::new(1, Some(0)), // mid
            TestNode::new(2, Some(1)), // deep-a
            TestNode::new(3, Some(1)), // deep-b
        ];
        // deep-a and deep-b diverge at node 1
        assert_eq!(find_divergence(2, &nodes), Some(1));
        assert_eq!(find_divergence(3, &nodes), Some(1));
    }

    #[test]
    fn find_leaves_simple() {
        let nodes = vec![
            TestNode::new(0, None),    // root (has children)
            TestNode::new(1, Some(0)), // leaf
            TestNode::new(2, Some(0)), // leaf
        ];
        let mut leaves = find_leaves(&nodes);
        leaves.sort();
        assert_eq!(leaves, vec![1, 2]);
    }

    #[test]
    fn find_leaves_linear() {
        let nodes = vec![
            TestNode::new(0, None),    // root (has child)
            TestNode::new(1, Some(0)), // intermediate (has child)
            TestNode::new(2, Some(1)), // leaf
        ];
        let leaves = find_leaves(&nodes);
        assert_eq!(leaves, vec![2]);
    }

    #[test]
    fn count_children_none() {
        let nodes = vec![TestNode::new(0, None), TestNode::new(1, Some(0))];
        assert_eq!(count_children(1, &nodes), 0); // leaf has no children
    }

    #[test]
    fn count_children_multiple() {
        let nodes = vec![
            TestNode::new(0, None),    // root
            TestNode::new(1, Some(0)), // child 1
            TestNode::new(2, Some(0)), // child 2
            TestNode::new(3, Some(0)), // child 3
        ];
        assert_eq!(count_children(0, &nodes), 3);
    }
}
