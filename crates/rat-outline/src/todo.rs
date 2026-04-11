//! TODO keyword cycling on heading lines.

use crate::index::HeadingInfo;

/// Cycle the TODO state of the heading at `heading_idx`.
///
/// Advances through `keywords` in order. After the last keyword, removes the
/// keyword entirely. From no keyword, inserts the first keyword.
///
/// Modifies the buffer line in place and returns the new TODO state.
pub fn cycle_todo(
    lines: &mut [String],
    heading: &HeadingInfo,
    keywords: &[String],
) -> Option<String> {
    if keywords.is_empty() {
        return heading.todo.clone();
    }

    let current = heading.todo.as_deref();
    let line = &lines[heading.line];

    // Find current keyword position in the cycle
    let current_idx = current.and_then(|kw| keywords.iter().position(|k| k == kw));

    let (new_kw, remove_old) = match current_idx {
        Some(idx) if idx + 1 < keywords.len() => {
            // Advance to next keyword
            (Some(keywords[idx + 1].as_str()), true)
        }
        Some(_) => {
            // Past the last keyword: remove
            (None, true)
        }
        None if current.is_some() => {
            // Current keyword not in our list — replace with first
            (Some(keywords[0].as_str()), true)
        }
        None => {
            // No current keyword — insert first
            (Some(keywords[0].as_str()), false)
        }
    };

    // Reconstruct the line
    let new_line = replace_todo_in_line(line, current, new_kw, remove_old);
    lines[heading.line] = new_line;

    new_kw.map(|s| s.to_string())
}

/// Replace or insert a TODO keyword in a heading line.
fn replace_todo_in_line(
    line: &str,
    old_kw: Option<&str>,
    new_kw: Option<&str>,
    remove_old: bool,
) -> String {
    // Find the heading marker (stars or hashes)
    let marker_end = line.bytes().take_while(|&b| b == b'*' || b == b'#').count();
    let after_marker = &line[marker_end..];

    // After marker there should be a space
    let after_space = if after_marker.starts_with(' ') {
        &after_marker[1..]
    } else {
        after_marker
    };

    let marker = &line[..marker_end];

    if remove_old {
        if let Some(old) = old_kw {
            // Strip the old keyword
            let stripped = if let Some(rest) = after_space.strip_prefix(old) {
                rest.strip_prefix(' ').unwrap_or(rest)
            } else {
                after_space
            };
            match new_kw {
                Some(kw) => format!("{} {} {}", marker, kw, stripped),
                None => format!("{} {}", marker, stripped),
            }
        } else {
            // Shouldn't happen, but handle gracefully
            match new_kw {
                Some(kw) => format!("{} {} {}", marker, kw, after_space),
                None => format!("{} {}", marker, after_space),
            }
        }
    } else {
        // Insert new keyword after marker
        match new_kw {
            Some(kw) => format!("{} {} {}", marker, kw, after_space),
            None => line.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{FoldState, HeadingInfo};

    fn heading(line: usize, todo: Option<&str>) -> HeadingInfo {
        HeadingInfo {
            line,
            level: 1,
            fold: FoldState::All,
            todo: todo.map(|s| s.to_string()),
            priority: None,
            tags: vec![],
            title: String::new(),
        }
    }

    #[test]
    fn cycle_none_to_first() {
        let kws = vec!["TODO".into(), "DONE".into()];
        let mut lines = vec!["* Task".to_string()];
        let h = heading(0, None);
        let result = cycle_todo(&mut lines, &h, &kws);
        assert_eq!(result, Some("TODO".into()));
        assert_eq!(lines[0], "* TODO Task");
    }

    #[test]
    fn cycle_todo_to_done() {
        let kws = vec!["TODO".into(), "DONE".into()];
        let mut lines = vec!["* TODO Task".to_string()];
        let h = heading(0, Some("TODO"));
        let result = cycle_todo(&mut lines, &h, &kws);
        assert_eq!(result, Some("DONE".into()));
        assert_eq!(lines[0], "* DONE Task");
    }

    #[test]
    fn cycle_past_last_removes() {
        let kws = vec!["TODO".into(), "DONE".into()];
        let mut lines = vec!["* DONE Task".to_string()];
        let h = heading(0, Some("DONE"));
        let result = cycle_todo(&mut lines, &h, &kws);
        assert_eq!(result, None);
        assert_eq!(lines[0], "* Task");
    }

    #[test]
    fn empty_keywords_noop() {
        let mut lines = vec!["* Task".to_string()];
        let h = heading(0, None);
        let result = cycle_todo(&mut lines, &h, &[]);
        assert_eq!(result, None);
        assert_eq!(lines[0], "* Task");
    }
}
