//! Heading syntax detection and parsing.

/// Result of parsing a single line as a heading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedHeading {
    /// Heading level (1+).
    pub level: usize,
    /// TODO keyword if present (e.g. "TODO", "DONE").
    pub todo: Option<String>,
    /// Priority cookie character (e.g. 'A', 'B', 'C').
    pub priority: Option<char>,
    /// Tags parsed from the trailing `:tag1:tag2:` syntax.
    pub tags: Vec<String>,
    /// The heading title text (after stripping markers, TODO, priority, tags).
    pub title: String,
}

/// Selects which built-in parser to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingSyntax {
    /// Org-mode style: `* `, `** `, `*** `, etc.
    Org,
    /// Markdown style: `# `, `## `, `### `, etc.
    Markdown,
}

/// Trait for plugging in custom heading detection.
pub trait HeadingParser: Send + Sync {
    /// Try to parse `line` as a heading. Returns `None` if the line is not a
    /// heading in this syntax.
    fn parse_line(&self, line: &str) -> Option<ParsedHeading>;
}

// ── Org parser ──────────────────────────────────────────────────────────────

/// Org-mode heading parser. Recognizes lines starting with one or more `*`
/// followed by a space.
#[derive(Debug, Clone, Copy)]
pub struct OrgParser;

impl HeadingParser for OrgParser {
    fn parse_line(&self, line: &str) -> Option<ParsedHeading> {
        // Count leading stars
        let star_count = line.bytes().take_while(|&b| b == b'*').count();
        if star_count == 0 {
            return None;
        }
        // Must be followed by a space
        if line.as_bytes().get(star_count) != Some(&b' ') {
            return None;
        }
        let rest = &line[star_count + 1..];
        parse_heading_content(star_count, rest)
    }
}

// ── Markdown parser ─────────────────────────────────────────────────────────

/// Markdown heading parser. Recognizes lines starting with one or more `#`
/// followed by a space.
#[derive(Debug, Clone, Copy)]
pub struct MarkdownParser;

impl HeadingParser for MarkdownParser {
    fn parse_line(&self, line: &str) -> Option<ParsedHeading> {
        let hash_count = line.bytes().take_while(|&b| b == b'#').count();
        if hash_count == 0 {
            return None;
        }
        if line.as_bytes().get(hash_count) != Some(&b' ') {
            return None;
        }
        let rest = &line[hash_count + 1..];
        // Markdown headings don't have TODO/priority/tags natively, but we
        // still attempt to parse them for interop.
        parse_heading_content(hash_count, rest)
    }
}

// ── Shared content parser ───────────────────────────────────────────────────

/// Default TODO keywords recognized when parsing headings. Callers can pass a
/// custom list through `OutlineState`.
pub const DEFAULT_TODO_KEYWORDS: &[&str] = &[
    "TODO",
    "IN_PROGRESS",
    "PROG",
    "DONE",
    "CANCELLED",
    "CANCELED",
    "WAIT",
    "HOLD",
];

/// Parse the content after the heading marker (stars/hashes + space).
fn parse_heading_content(level: usize, content: &str) -> Option<ParsedHeading> {
    let mut rest = content;

    // Try to extract TODO keyword (first word if it matches known keywords)
    let todo = extract_todo(&mut rest);

    // Try to extract priority cookie [#A]
    let priority = extract_priority(&mut rest);

    // Try to extract trailing tags :tag1:tag2:
    let tags = extract_tags(&mut rest);

    let title = rest.trim().to_string();

    Some(ParsedHeading {
        level,
        todo,
        priority,
        tags,
        title,
    })
}

fn extract_todo(rest: &mut &str) -> Option<String> {
    let trimmed = rest.trim_start();
    for &kw in DEFAULT_TODO_KEYWORDS {
        if let Some(after) = trimmed.strip_prefix(kw) {
            // Keyword must be followed by space, end-of-string, or [#
            if after.is_empty() || after.starts_with(' ') || after.starts_with(" [#") {
                *rest = after.trim_start();
                return Some(kw.to_string());
            }
        }
    }
    None
}

fn extract_priority(rest: &mut &str) -> Option<char> {
    let trimmed = rest.trim_start();
    if trimmed.len() >= 4
        && trimmed.as_bytes()[0] == b'['
        && trimmed.as_bytes()[1] == b'#'
        && trimmed.as_bytes()[2].is_ascii_uppercase()
        && trimmed.as_bytes()[3] == b']'
    {
        let ch = trimmed.as_bytes()[2] as char;
        *rest = trimmed[4..].trim_start();
        return Some(ch);
    }
    None
}

fn extract_tags(rest: &mut &str) -> Vec<String> {
    let trimmed = rest.trim_end();
    if !trimmed.ends_with(':') {
        return Vec::new();
    }
    // Walk backwards to find the tag block start
    // Tags look like :tag1:tag2: at the end
    let bytes = trimmed.as_bytes();
    let mut i = bytes.len() - 1; // on the trailing ':'
    // Find where the tag block starts — look for the pattern ` :` or start-of-string `:`
    loop {
        if i == 0 {
            break;
        }
        i -= 1;
        if bytes[i] == b':' {
            // Check if this is preceded by a space or is at position 0
            if i == 0 || bytes[i - 1] == b' ' {
                // This is the start of the tag block
                let tag_str = &trimmed[i..];
                let tags: Vec<String> = tag_str
                    .split(':')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
                if !tags.is_empty() && tags.iter().all(|t| is_valid_tag(t)) {
                    let before = if i > 0 { &trimmed[..i] } else { "" };
                    *rest = before.trim_end();
                    return tags;
                }
                break;
            }
        } else if bytes[i] == b' ' {
            // Space inside what we thought was a tag block — not tags
            break;
        }
    }
    Vec::new()
}

fn is_valid_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'@')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn org_simple_heading() {
        let p = OrgParser;
        let h = p.parse_line("* Hello").unwrap();
        assert_eq!(h.level, 1);
        assert_eq!(h.title, "Hello");
        assert_eq!(h.todo, None);
        assert_eq!(h.priority, None);
        assert!(h.tags.is_empty());
    }

    #[test]
    fn org_level_3() {
        let p = OrgParser;
        let h = p.parse_line("*** Deep heading").unwrap();
        assert_eq!(h.level, 3);
        assert_eq!(h.title, "Deep heading");
    }

    #[test]
    fn org_todo_keyword() {
        let p = OrgParser;
        let h = p.parse_line("* TODO Ship docs").unwrap();
        assert_eq!(h.level, 1);
        assert_eq!(h.todo, Some("TODO".into()));
        assert_eq!(h.title, "Ship docs");
    }

    #[test]
    fn org_priority() {
        let p = OrgParser;
        let h = p.parse_line("* TODO [#A] Critical task").unwrap();
        assert_eq!(h.todo, Some("TODO".into()));
        assert_eq!(h.priority, Some('A'));
        assert_eq!(h.title, "Critical task");
    }

    #[test]
    fn org_tags() {
        let p = OrgParser;
        let h = p.parse_line("** Bar :tag1:tag2:").unwrap();
        assert_eq!(h.level, 2);
        assert_eq!(h.title, "Bar");
        assert_eq!(h.tags, vec!["tag1", "tag2"]);
    }

    #[test]
    fn org_full() {
        let p = OrgParser;
        let h = p
            .parse_line("** IN_PROGRESS [#B] Implement parser :code:rust:")
            .unwrap();
        assert_eq!(h.level, 2);
        assert_eq!(h.todo, Some("IN_PROGRESS".into()));
        assert_eq!(h.priority, Some('B'));
        assert_eq!(h.title, "Implement parser");
        assert_eq!(h.tags, vec!["code", "rust"]);
    }

    #[test]
    fn org_not_a_heading() {
        let p = OrgParser;
        assert!(p.parse_line("Not a heading").is_none());
        assert!(p.parse_line("*bold text*").is_none());
        assert!(p.parse_line("").is_none());
    }

    #[test]
    fn markdown_heading() {
        let p = MarkdownParser;
        let h = p.parse_line("## Task").unwrap();
        assert_eq!(h.level, 2);
        assert_eq!(h.title, "Task");
    }

    #[test]
    fn markdown_with_todo() {
        let p = MarkdownParser;
        let h = p.parse_line("## TODO Ship docs").unwrap();
        assert_eq!(h.todo, Some("TODO".into()));
        assert_eq!(h.title, "Ship docs");
    }

    #[test]
    fn markdown_not_heading() {
        let p = MarkdownParser;
        assert!(p.parse_line("not heading").is_none());
        assert!(p.parse_line("#nospace").is_none());
    }

    #[test]
    fn tags_only_at_end() {
        let p = OrgParser;
        let h = p.parse_line("* Title with :colon: inside :real:").unwrap();
        assert_eq!(h.tags, vec!["real"]);
        assert_eq!(h.title, "Title with :colon: inside");
    }
}
