//! Heading index: parallel structure tracking heading positions and metadata.

use crate::parse::{HeadingParser, ParsedHeading};

/// Visibility state for a heading's subtree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldState {
    /// Only the heading line is visible.
    Folded,
    /// Heading + direct child heading lines visible; bodies and grandchildren hidden.
    Children,
    /// Entire subtree visible.
    All,
}

impl FoldState {
    /// Advance to the next state in the cycle: Folded → Children → All → Folded.
    pub fn cycle(self) -> Self {
        match self {
            Self::Folded => Self::Children,
            Self::Children => Self::All,
            Self::All => Self::Folded,
        }
    }
}

/// Metadata for a single heading in the document.
#[derive(Debug, Clone)]
pub struct HeadingInfo {
    /// Line number in the buffer (0-indexed).
    pub line: usize,
    /// Heading level (1+).
    pub level: usize,
    /// Current fold state.
    pub fold: FoldState,
    /// TODO keyword if present.
    pub todo: Option<String>,
    /// Priority cookie character.
    pub priority: Option<char>,
    /// Tags from trailing `:tag:` syntax.
    pub tags: Vec<String>,
    /// Title text (stripped of markers, TODO, priority, tags).
    pub title: String,
}

impl HeadingInfo {
    fn from_parsed(line: usize, parsed: ParsedHeading) -> Self {
        Self {
            line,
            level: parsed.level,
            fold: FoldState::All,
            todo: parsed.todo,
            priority: parsed.priority,
            tags: parsed.tags,
            title: parsed.title,
        }
    }
}

/// Scan buffer lines and produce a heading index.
///
/// Preserves fold state from `prev` for headings that still exist at the same
/// line with the same level.
pub fn build_heading_index(
    lines: &[String],
    parser: &dyn HeadingParser,
    prev: &[HeadingInfo],
) -> Vec<HeadingInfo> {
    let mut index = Vec::new();
    // Build a quick lookup: (line, level) → FoldState from the previous index
    // so that editing doesn't reset all folds.
    let prev_folds: std::collections::HashMap<(usize, usize), FoldState> =
        prev.iter().map(|h| ((h.line, h.level), h.fold)).collect();

    for (line_num, line_text) in lines.iter().enumerate() {
        if let Some(parsed) = parser.parse_line(line_text) {
            let mut info = HeadingInfo::from_parsed(line_num, parsed);
            // Restore fold state if the heading is at the same position and level
            if let Some(&fold) = prev_folds.get(&(line_num, info.level)) {
                info.fold = fold;
            }
            index.push(info);
        }
    }
    index
}

/// Find the index in `headings` of the heading at or before `line`.
/// Returns `None` if `line` is before all headings.
pub fn heading_at_or_before(headings: &[HeadingInfo], line: usize) -> Option<usize> {
    // Binary search for the last heading whose line <= `line`
    match headings.binary_search_by_key(&line, |h| h.line) {
        Ok(i) => Some(i),
        Err(0) => None,
        Err(i) => Some(i - 1),
    }
}

/// Return the line range (start..end exclusive) of the subtree rooted at
/// heading index `idx`. The subtree extends until the next heading at the
/// same or higher level, or end of document.
pub fn subtree_range(
    headings: &[HeadingInfo],
    idx: usize,
    total_lines: usize,
) -> std::ops::Range<usize> {
    let start = headings[idx].line;
    let level = headings[idx].level;
    let end = headings[idx + 1..]
        .iter()
        .find(|h| h.level <= level)
        .map(|h| h.line)
        .unwrap_or(total_lines);
    start..end
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::OrgParser;

    fn lines(s: &str) -> Vec<String> {
        s.lines().map(|l| l.to_string()).collect()
    }

    #[test]
    fn build_index_basic() {
        let buf = lines("* Heading 1\nbody\n** Heading 2\nmore body");
        let idx = build_heading_index(&buf, &OrgParser, &[]);
        assert_eq!(idx.len(), 2);
        assert_eq!(idx[0].line, 0);
        assert_eq!(idx[0].level, 1);
        assert_eq!(idx[0].title, "Heading 1");
        assert_eq!(idx[1].line, 2);
        assert_eq!(idx[1].level, 2);
    }

    #[test]
    fn build_index_no_headings() {
        let buf = lines("just some text\nno headings here");
        let idx = build_heading_index(&buf, &OrgParser, &[]);
        assert!(idx.is_empty());
    }

    #[test]
    fn preserves_fold_state() {
        let buf = lines("* A\nbody\n** B\n");
        let prev = vec![HeadingInfo {
            line: 0,
            level: 1,
            fold: FoldState::Folded,
            todo: None,
            priority: None,
            tags: vec![],
            title: "A".into(),
        }];
        let idx = build_heading_index(&buf, &OrgParser, &prev);
        assert_eq!(idx[0].fold, FoldState::Folded);
        // New heading gets default All
        assert_eq!(idx[1].fold, FoldState::All);
    }

    #[test]
    fn heading_at_or_before_finds_correct() {
        let buf = lines("* A\nbody\n** B\nbody2\n* C");
        let idx = build_heading_index(&buf, &OrgParser, &[]);
        // Line 0 -> heading 0
        assert_eq!(heading_at_or_before(&idx, 0), Some(0));
        // Line 1 (body) -> heading 0
        assert_eq!(heading_at_or_before(&idx, 1), Some(0));
        // Line 2 -> heading 1
        assert_eq!(heading_at_or_before(&idx, 2), Some(1));
        // Line 3 -> heading 1
        assert_eq!(heading_at_or_before(&idx, 3), Some(1));
        // Line 4 -> heading 2
        assert_eq!(heading_at_or_before(&idx, 4), Some(2));
    }

    #[test]
    fn subtree_range_basic() {
        let buf = lines("* A\nbody\n** B\nbody2\n* C\nbody3");
        let idx = build_heading_index(&buf, &OrgParser, &[]);
        // Subtree of heading 0 (A, level 1): lines 0..4 (before C at line 4)
        assert_eq!(subtree_range(&idx, 0, 6), 0..4);
        // Subtree of heading 1 (B, level 2): lines 2..4
        assert_eq!(subtree_range(&idx, 1, 6), 2..4);
        // Subtree of heading 2 (C, level 1): lines 4..6 (end of doc)
        assert_eq!(subtree_range(&idx, 2, 6), 4..6);
    }
}
