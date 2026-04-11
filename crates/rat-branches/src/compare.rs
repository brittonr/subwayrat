//! Generic branch comparison algorithm and data structures

use crate::tree::{TreeNode, walk_to_root};

/// Summary of a single node in the comparison view
#[derive(Debug, Clone)]
pub struct CompareBlock {
    /// Node ID
    pub id: usize,
    /// Preview text (e.g., first line of content, truncated)
    pub preview: String,
    /// Detail counts (label, count) pairs for metadata
    pub detail_counts: Vec<(String, usize)>,
    /// Token usage or other numeric measure
    pub tokens: usize,
}

impl CompareBlock {
    /// Create a new CompareBlock with the basic fields
    pub fn new(id: usize, preview: String, tokens: usize) -> Self {
        Self {
            id,
            preview,
            detail_counts: Vec::new(),
            tokens,
        }
    }

    /// Add a detail count (e.g., "responses", count)
    pub fn add_detail_count(mut self, label: impl Into<String>, count: usize) -> Self {
        self.detail_counts.push((label.into(), count));
        self
    }
}

/// Result of comparing two branches of a tree
#[derive(Debug, Clone)]
pub struct BranchComparison {
    /// The divergence point node ID (last common ancestor).
    /// `None` if branches share no common ancestor.
    pub divergence_id: Option<usize>,

    /// Summary text at the divergence point
    pub divergence_summary: String,

    /// Nodes unique to branch A (from divergence → leaf A)
    pub branch_a: Vec<CompareBlock>,

    /// Nodes unique to branch B (from divergence → leaf B)
    pub branch_b: Vec<CompareBlock>,

    /// Leaf node ID of branch A
    pub leaf_a: usize,

    /// Leaf node ID of branch B
    pub leaf_b: usize,

    /// Display name for branch A
    pub name_a: String,

    /// Display name for branch B
    pub name_b: String,

    /// Total tokens for branch A (unique portion only)
    pub tokens_a: usize,

    /// Total tokens for branch B (unique portion only)
    pub tokens_b: usize,
}

/// Compare two branches of a tree, returning their divergence and unique nodes.
///
/// The `summarize` function converts a node to a `CompareBlock` for display.
pub fn compare_branches<N: TreeNode>(
    leaf_a: usize,
    leaf_b: usize,
    nodes: &[N],
    summarize: impl Fn(&N) -> CompareBlock,
) -> Option<BranchComparison> {
    let path_a = walk_to_root(leaf_a, nodes);
    let path_b = walk_to_root(leaf_b, nodes);

    // Find the last common node (divergence point)
    let mut divergence_idx = 0;
    for (i, (&a, &b)) in path_a.iter().zip(path_b.iter()).enumerate() {
        if a == b {
            divergence_idx = i;
        } else {
            break;
        }
    }

    let divergence_id = path_a.get(divergence_idx).copied();
    let divergence_summary = divergence_id
        .and_then(|id| nodes.iter().find(|n| n.id() == id))
        .map(|n| summarize(n).preview)
        .unwrap_or_default();

    // Unique nodes: everything after the divergence point
    let unique_a: Vec<CompareBlock> = path_a[divergence_idx + 1..]
        .iter()
        .filter_map(|&id| nodes.iter().find(|n| n.id() == id))
        .map(&summarize)
        .collect();

    let unique_b: Vec<CompareBlock> = path_b[divergence_idx + 1..]
        .iter()
        .filter_map(|&id| nodes.iter().find(|n| n.id() == id))
        .map(&summarize)
        .collect();

    let tokens_a: usize = unique_a.iter().map(|b| b.tokens).sum();
    let tokens_b: usize = unique_b.iter().map(|b| b.tokens).sum();

    Some(BranchComparison {
        divergence_id,
        divergence_summary,
        branch_a: unique_a,
        branch_b: unique_b,
        leaf_a,
        leaf_b,
        name_a: format!("branch (#{} leaf)", leaf_a),
        name_b: format!("branch (#{} leaf)", leaf_b),
        tokens_a,
        tokens_b,
    })
}

/// Truncate text to the first line and a max character count
pub fn truncate_first_line(text: &str, max: usize) -> String {
    let first_line = text.lines().next().unwrap_or(text);
    let preview: String = first_line.chars().take(max).collect();
    if first_line.chars().count() > max {
        format!("{}…", preview)
    } else {
        preview
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestNode {
        id: usize,
        parent: Option<usize>,
        content: String,
        tokens: usize,
    }

    impl TestNode {
        fn new(id: usize, content: &str, parent: Option<usize>, tokens: usize) -> Self {
            Self {
                id,
                parent,
                content: content.to_string(),
                tokens,
            }
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

    fn node_to_compare_block(node: &TestNode) -> CompareBlock {
        CompareBlock::new(
            node.id(),
            truncate_first_line(&node.content, 50),
            node.tokens,
        )
    }

    #[test]
    fn compare_simple_fork() {
        let nodes = vec![
            TestNode::new(0, "root question", None, 100),
            TestNode::new(1, "answer-a", Some(0), 200),
            TestNode::new(2, "answer-b", Some(0), 150),
        ];
        let cmp = compare_branches(1, 2, &nodes, node_to_compare_block).unwrap();

        assert_eq!(cmp.divergence_id, Some(0));
        assert_eq!(cmp.branch_a.len(), 1);
        assert_eq!(cmp.branch_b.len(), 1);
        assert_eq!(cmp.branch_a[0].id, 1);
        assert_eq!(cmp.branch_b[0].id, 2);
        assert_eq!(cmp.tokens_a, 200);
        assert_eq!(cmp.tokens_b, 150);
    }

    #[test]
    fn compare_deep_fork() {
        // root → mid → deep-a
        //             → deep-b → deeper-b
        let nodes = vec![
            TestNode::new(0, "root", None, 100),
            TestNode::new(1, "mid", Some(0), 200),
            TestNode::new(2, "deep-a", Some(1), 150),
            TestNode::new(3, "deep-b", Some(1), 120),
            TestNode::new(4, "deeper-b", Some(3), 80),
        ];
        let cmp = compare_branches(2, 4, &nodes, node_to_compare_block).unwrap();

        // Diverges at node 1 (mid)
        assert_eq!(cmp.divergence_id, Some(1));
        // Branch A: [deep-a]
        assert_eq!(cmp.branch_a.len(), 1);
        assert_eq!(cmp.branch_a[0].id, 2);
        // Branch B: [deep-b, deeper-b]
        assert_eq!(cmp.branch_b.len(), 2);
        assert_eq!(cmp.branch_b[0].id, 3);
        assert_eq!(cmp.branch_b[1].id, 4);
    }

    #[test]
    fn compare_same_branch_no_unique() {
        let nodes = vec![
            TestNode::new(0, "root", None, 100),
            TestNode::new(1, "child", Some(0), 200),
        ];
        // Comparing a branch with itself: leaf 1 vs leaf 1
        let cmp = compare_branches(1, 1, &nodes, node_to_compare_block).unwrap();
        assert_eq!(cmp.divergence_id, Some(1));
        assert!(cmp.branch_a.is_empty());
        assert!(cmp.branch_b.is_empty());
    }

    #[test]
    fn compare_asymmetric_depths() {
        // root → a → a2 → a3
        //       → b
        let nodes = vec![
            TestNode::new(0, "root", None, 50),
            TestNode::new(1, "a", Some(0), 100),
            TestNode::new(2, "a2", Some(1), 100),
            TestNode::new(3, "a3", Some(2), 100),
            TestNode::new(4, "b", Some(0), 200),
        ];
        let cmp = compare_branches(3, 4, &nodes, node_to_compare_block).unwrap();

        assert_eq!(cmp.divergence_id, Some(0));
        assert_eq!(cmp.branch_a.len(), 3); // a, a2, a3
        assert_eq!(cmp.branch_b.len(), 1); // b
    }

    #[test]
    fn truncate_first_line_short() {
        assert_eq!(truncate_first_line("hello", 10), "hello");
    }

    #[test]
    fn truncate_first_line_long() {
        assert_eq!(
            truncate_first_line("hello world this is a long text", 10),
            "hello worl…"
        );
    }

    #[test]
    fn truncate_first_line_multiline() {
        assert_eq!(truncate_first_line("first\nsecond\nthird", 20), "first");
    }

    #[test]
    fn compare_block_builder() {
        let block = CompareBlock::new(1, "test".to_string(), 100)
            .add_detail_count("responses", 5)
            .add_detail_count("tools", 2);

        assert_eq!(block.id, 1);
        assert_eq!(block.preview, "test");
        assert_eq!(block.tokens, 100);
        assert_eq!(block.detail_counts.len(), 2);
        assert_eq!(block.detail_counts[0], ("responses".to_string(), 5));
        assert_eq!(block.detail_counts[1], ("tools".to_string(), 2));
    }
}
