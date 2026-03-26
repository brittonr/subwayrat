//! Structural editing: promote, demote, move subtree.

use crate::index::{HeadingInfo, subtree_range};
use crate::parse::HeadingSyntax;

/// Promote heading at `heading_idx` and its entire subtree (decrease level by 1).
/// Returns `true` if the operation was applied.
pub fn promote(
    lines: &mut Vec<String>,
    headings: &[HeadingInfo],
    heading_idx: usize,
    syntax: HeadingSyntax,
) -> bool {
    let level = headings[heading_idx].level;
    if level <= 1 {
        return false; // Cannot promote above level 1
    }

    let range = subtree_range(headings, heading_idx, lines.len());

    // Find all headings in the subtree and reduce their marker by one
    for hi in headings.iter() {
        if hi.line >= range.start && hi.line < range.end {
            adjust_heading_level(&mut lines[hi.line], syntax, -1);
        }
    }
    true
}

/// Demote heading at `heading_idx` and its entire subtree (increase level by 1).
/// Returns `true` if the operation was applied.
pub fn demote(
    lines: &mut Vec<String>,
    headings: &[HeadingInfo],
    heading_idx: usize,
    syntax: HeadingSyntax,
) -> bool {
    let range = subtree_range(headings, heading_idx, lines.len());

    for hi in headings.iter() {
        if hi.line >= range.start && hi.line < range.end {
            adjust_heading_level(&mut lines[hi.line], syntax, 1);
        }
    }
    true
}

/// Move the subtree at `heading_idx` up (swap with previous sibling).
/// Returns the new heading line number if moved, or `None`.
pub fn move_subtree_up(
    lines: &mut Vec<String>,
    headings: &[HeadingInfo],
    heading_idx: usize,
) -> Option<usize> {
    let total = lines.len();

    // Find the previous sibling: walk backwards for a heading at the same level
    // that shares the same parent.
    let prev_sibling_idx = find_prev_sibling(headings, heading_idx)?;

    let prev_range = subtree_range(headings, prev_sibling_idx, total);
    let cur_range = subtree_range(headings, heading_idx, total);

    // Extract both subtrees, put current before previous
    let cur_lines: Vec<String> = lines[cur_range.clone()].to_vec();
    let prev_lines: Vec<String> = lines[prev_range.clone()].to_vec();

    // Replace the combined range
    let combined_start = prev_range.start;
    let combined_end = cur_range.end;
    let mut replacement = Vec::with_capacity(cur_lines.len() + prev_lines.len());
    replacement.extend(cur_lines);
    replacement.extend(prev_lines);
    lines.splice(combined_start..combined_end, replacement);

    Some(combined_start)
}

/// Move the subtree at `heading_idx` down (swap with next sibling).
/// Returns the new heading line number if moved, or `None`.
pub fn move_subtree_down(
    lines: &mut Vec<String>,
    headings: &[HeadingInfo],
    heading_idx: usize,
) -> Option<usize> {
    let total = lines.len();

    // Find the next sibling
    let next_sibling_idx = find_next_sibling(headings, heading_idx)?;

    let cur_range = subtree_range(headings, heading_idx, total);
    let next_range = subtree_range(headings, next_sibling_idx, total);

    // Extract both subtrees, put next before current
    let cur_lines: Vec<String> = lines[cur_range.clone()].to_vec();
    let next_lines: Vec<String> = lines[next_range.clone()].to_vec();

    let combined_start = cur_range.start;
    let combined_end = next_range.end;
    let mut replacement = Vec::with_capacity(cur_lines.len() + next_lines.len());
    replacement.extend(next_lines.iter().cloned());
    replacement.extend(cur_lines);
    lines.splice(combined_start..combined_end, replacement);

    Some(combined_start + next_lines.len())
}

/// Adjust the heading marker level on a line by `delta` (+1 or -1).
fn adjust_heading_level(line: &mut String, syntax: HeadingSyntax, delta: i32) {
    match syntax {
        HeadingSyntax::Org => {
            let stars = line.bytes().take_while(|&b| b == b'*').count();
            let new_level = (stars as i32 + delta).max(1) as usize;
            let rest = &line[stars..];
            *line = format!("{}{}", "*".repeat(new_level), rest);
        }
        HeadingSyntax::Markdown => {
            let hashes = line.bytes().take_while(|&b| b == b'#').count();
            let new_level = (hashes as i32 + delta).max(1) as usize;
            let rest = &line[hashes..];
            *line = format!("{}{}", "#".repeat(new_level), rest);
        }
    }
}

fn find_prev_sibling(headings: &[HeadingInfo], idx: usize) -> Option<usize> {
    let level = headings[idx].level;
    for i in (0..idx).rev() {
        if headings[i].level == level {
            return Some(i);
        }
        if headings[i].level < level {
            return None; // Hit a parent before finding a sibling
        }
    }
    None
}

fn find_next_sibling(headings: &[HeadingInfo], idx: usize) -> Option<usize> {
    let level = headings[idx].level;
    for i in (idx + 1)..headings.len() {
        if headings[i].level == level {
            return Some(i);
        }
        if headings[i].level < level {
            return None; // Hit a parent before finding a sibling
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::build_heading_index;
    use crate::parse::OrgParser;

    fn lines(s: &str) -> Vec<String> {
        s.lines().map(|l| l.to_string()).collect()
    }

    fn make(text: &str) -> (Vec<String>, Vec<HeadingInfo>) {
        let buf = lines(text);
        let idx = build_heading_index(&buf, &OrgParser, &[]);
        (buf, idx)
    }

    #[test]
    fn promote_level2_to_level1() {
        let (mut buf, headings) = make("** Task\n*** Sub");
        assert!(promote(&mut buf, &headings, 0, HeadingSyntax::Org));
        assert_eq!(buf[0], "* Task");
        assert_eq!(buf[1], "** Sub");
    }

    #[test]
    fn promote_at_level1_noop() {
        let (mut buf, headings) = make("* Top");
        assert!(!promote(&mut buf, &headings, 0, HeadingSyntax::Org));
        assert_eq!(buf[0], "* Top");
    }

    #[test]
    fn demote_level1() {
        let (mut buf, headings) = make("* Task\n** Sub");
        assert!(demote(&mut buf, &headings, 0, HeadingSyntax::Org));
        assert_eq!(buf[0], "** Task");
        assert_eq!(buf[1], "*** Sub");
    }

    #[test]
    fn move_subtree_up_swaps() {
        let (mut buf, headings) = make("* A\nbody-a\n* B\nbody-b");
        let new_line = move_subtree_up(&mut buf, &headings, 1).unwrap();
        assert_eq!(new_line, 0);
        assert_eq!(buf[0], "* B");
        assert_eq!(buf[1], "body-b");
        assert_eq!(buf[2], "* A");
        assert_eq!(buf[3], "body-a");
    }

    #[test]
    fn move_subtree_up_first_sibling_noop() {
        let (mut buf, headings) = make("* A\n* B");
        assert!(move_subtree_up(&mut buf, &headings, 0).is_none());
    }

    #[test]
    fn move_subtree_down_swaps() {
        let (mut buf, headings) = make("* A\nbody-a\n* B\nbody-b");
        let new_line = move_subtree_down(&mut buf, &headings, 0).unwrap();
        assert_eq!(new_line, 2);
        assert_eq!(buf[0], "* B");
        assert_eq!(buf[1], "body-b");
        assert_eq!(buf[2], "* A");
        assert_eq!(buf[3], "body-a");
    }

    #[test]
    fn move_subtree_down_last_sibling_noop() {
        let (mut buf, headings) = make("* A\n* B");
        assert!(move_subtree_down(&mut buf, &headings, 1).is_none());
    }
}
