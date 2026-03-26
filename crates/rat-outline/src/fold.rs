//! Visibility cycling and visible-line computation.

use crate::index::{FoldState, HeadingInfo};

/// Cycle the visibility of the heading at `heading_idx`.
pub fn cycle_visibility(headings: &mut [HeadingInfo], heading_idx: usize) {
    headings[heading_idx].fold = headings[heading_idx].fold.cycle();
}

/// Set all headings to the same fold state. Typically used for global
/// fold (Folded) or global unfold (All).
pub fn cycle_visibility_global(headings: &mut [HeadingInfo], target: FoldState) {
    for h in headings.iter_mut() {
        h.fold = target;
    }
}

/// Compute which buffer line indices are visible given current fold states.
///
/// Returns a sorted `Vec<usize>` of visible line numbers.
pub fn visible_lines(headings: &[HeadingInfo], total_lines: usize) -> Vec<usize> {
    if headings.is_empty() {
        // No headings at all — everything is visible
        return (0..total_lines).collect();
    }

    let mut visible = Vec::with_capacity(total_lines);

    // Lines before the first heading are always visible
    if headings[0].line > 0 {
        for i in 0..headings[0].line {
            visible.push(i);
        }
    }

    // Process each heading
    for (hi, heading) in headings.iter().enumerate() {
        let vis = heading_visibility(headings, hi);
        match vis {
            HeadingVis::Hidden => {}
            HeadingVis::HeadingOnly => {
                // Visible via a parent's Children mode — show only the heading line
                visible.push(heading.line);
            }
            HeadingVis::Full => {
                visible.push(heading.line);

                match heading.fold {
                    FoldState::All => {
                        // Show everything in this heading's body (up to next heading)
                        let body_start = heading.line + 1;
                        let body_end = headings
                            .get(hi + 1)
                            .map(|next| next.line)
                            .unwrap_or(total_lines);
                        for line in body_start..body_end {
                            visible.push(line);
                        }
                    }
                    FoldState::Children => {
                        // Show body text between this heading and its first child
                        let body_start = heading.line + 1;
                        let next_heading_line = headings
                            .get(hi + 1)
                            .map(|next| next.line)
                            .unwrap_or(total_lines);
                        for line in body_start..next_heading_line {
                            visible.push(line);
                        }
                    }
                    FoldState::Folded => {
                        // Only the heading line
                    }
                }
            }
        }
    }

    visible.sort_unstable();
    visible.dedup();
    visible
}

/// How a heading is visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HeadingVis {
    /// Hidden by a parent Folded state.
    Hidden,
    /// Visible only as a heading line (parent is in Children mode).
    HeadingOnly,
    /// Fully visible — own fold state controls body display.
    Full,
}

/// Determine visibility of a heading considering ancestor fold states.
fn heading_visibility(headings: &[HeadingInfo], heading_idx: usize) -> HeadingVis {
    let heading = &headings[heading_idx];

    // Walk backwards to find the nearest ancestor (heading with lower level)
    for i in (0..heading_idx).rev() {
        let ancestor = &headings[i];
        if ancestor.level < heading.level {
            match ancestor.fold {
                FoldState::Folded => return HeadingVis::Hidden,
                FoldState::Children => {
                    if heading.level == ancestor.level + 1 {
                        // Direct child — visible as heading-only.
                        // Still check further ancestors.
                        return match heading_visibility(headings, i) {
                            HeadingVis::Hidden => HeadingVis::Hidden,
                            _ => HeadingVis::HeadingOnly,
                        };
                    } else {
                        return HeadingVis::Hidden;
                    }
                }
                FoldState::All => {
                    // If the ancestor is fully open, check further up.
                    // But if the ancestor itself is only HeadingOnly visible,
                    // then all its children (including us) are hidden.
                    let ancestor_vis = heading_visibility(headings, i);
                    return match ancestor_vis {
                        HeadingVis::Full => HeadingVis::Full,
                        HeadingVis::HeadingOnly => HeadingVis::Hidden,
                        HeadingVis::Hidden => HeadingVis::Hidden,
                    };
                }
            }
        }
    }
    // Top-level heading
    HeadingVis::Full
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::build_heading_index;
    use crate::parse::OrgParser;

    fn lines(s: &str) -> Vec<String> {
        s.lines().map(|l| l.to_string()).collect()
    }

    fn make_headings(text: &str) -> (Vec<String>, Vec<HeadingInfo>) {
        let buf = lines(text);
        let idx = build_heading_index(&buf, &OrgParser, &[]);
        let total = buf.len();
        (buf, idx)
    }

    #[test]
    fn all_visible_by_default() {
        let (buf, headings) = make_headings("* A\nbody\n** B\nbody2");
        let vis = visible_lines(&headings, buf.len());
        assert_eq!(vis, vec![0, 1, 2, 3]);
    }

    #[test]
    fn fold_hides_body_and_children() {
        let (buf, mut headings) = make_headings("* A\nbody\n** B\nbody2\n* C");
        headings[0].fold = FoldState::Folded;
        let vis = visible_lines(&headings, buf.len());
        // A (line 0) visible, body (1) hidden, B (2) hidden, body2 (3) hidden, C (4) visible
        assert_eq!(vis, vec![0, 4]);
    }

    #[test]
    fn children_shows_direct_children_headings() {
        let (buf, mut headings) = make_headings("* A\nbody\n** B\nbody2\n*** C\nbody3");
        headings[0].fold = FoldState::Children;
        let vis = visible_lines(&headings, buf.len());
        // A (0) visible, body (1) visible, B (2) visible, body2 (3) hidden, C (4) hidden, body3 (5) hidden
        // Wait — Children means heading + immediate child headings visible.
        // Body between A and B (line 1) is visible.
        // B's heading line (2) is visible (direct child).
        // B's body (3) and C (4, grandchild) are hidden.
        assert_eq!(vis, vec![0, 1, 2]);
    }

    #[test]
    fn no_headings_all_visible() {
        let buf = lines("just text\nmore text");
        let headings: Vec<HeadingInfo> = Vec::new();
        let vis = visible_lines(&headings, buf.len());
        assert_eq!(vis, vec![0, 1]);
    }

    #[test]
    fn cycle_fold_state() {
        let (_, mut headings) = make_headings("* A\n** B");
        assert_eq!(headings[0].fold, FoldState::All);
        cycle_visibility(&mut headings, 0);
        assert_eq!(headings[0].fold, FoldState::Folded);
        cycle_visibility(&mut headings, 0);
        assert_eq!(headings[0].fold, FoldState::Children);
        cycle_visibility(&mut headings, 0);
        assert_eq!(headings[0].fold, FoldState::All);
    }

    #[test]
    fn global_fold() {
        let (_, mut headings) = make_headings("* A\n** B\n* C");
        cycle_visibility_global(&mut headings, FoldState::Folded);
        assert!(headings.iter().all(|h| h.fold == FoldState::Folded));
    }
}
