//! Outline state: wraps Editor with heading awareness.

use rat_editor::Editor;

use crate::index::{HeadingInfo, build_heading_index, heading_at_or_before};
use crate::parse::{HeadingParser, HeadingSyntax, MarkdownParser, OrgParser};

/// Top-level state for the outline editor.
pub struct OutlineState {
    /// The underlying text buffer.
    pub editor: Editor,
    /// Heading index, rebuilt on buffer changes.
    pub headings: Vec<HeadingInfo>,
    /// Which parser to use.
    syntax: HeadingSyntax,
    /// Custom parser (overrides syntax if set).
    custom_parser: Option<Box<dyn HeadingParser>>,
    /// Ordered list of TODO keywords for cycling.
    pub todo_keywords: Vec<String>,
    /// Vertical scroll offset (in visible lines).
    pub scroll_offset: usize,
    /// Dirty flag: set when the buffer changes and the index needs rebuilding.
    dirty: bool,
}

impl OutlineState {
    /// Create a new empty outline with org syntax.
    pub fn new() -> Self {
        Self {
            editor: Editor::new(),
            headings: Vec::new(),
            syntax: HeadingSyntax::Org,
            custom_parser: None,
            todo_keywords: vec!["TODO".into(), "IN_PROGRESS".into(), "DONE".into()],
            scroll_offset: 0,
            dirty: false,
        }
    }

    /// Create with a specific heading syntax.
    pub fn with_syntax(syntax: HeadingSyntax) -> Self {
        let mut s = Self::new();
        s.syntax = syntax;
        s
    }

    /// Use a custom heading parser.
    pub fn with_parser(parser: Box<dyn HeadingParser>) -> Self {
        let mut s = Self::new();
        s.custom_parser = Some(parser);
        s
    }

    /// Set the TODO keyword cycle list.
    pub fn set_todo_keywords(&mut self, keywords: Vec<String>) {
        self.todo_keywords = keywords;
    }

    /// Load text into the editor and rebuild the heading index.
    pub fn load_text(&mut self, text: &str) {
        self.editor.clear();
        for (i, ch) in text.chars().enumerate() {
            // Skip trailing newline to avoid an empty final line
            if ch == '\n' && i == text.len() - 1 && !text.is_empty() {
                continue;
            }
            self.editor.insert_char(ch);
        }
        // Reset cursor to top
        self.editor.set_cursor(0, 0);
        self.rebuild_index();
    }

    /// Get a reference to the parser in use.
    pub fn parser(&self) -> &dyn HeadingParser {
        if let Some(ref p) = self.custom_parser {
            p.as_ref()
        } else {
            match self.syntax {
                HeadingSyntax::Org => &OrgParser,
                HeadingSyntax::Markdown => &MarkdownParser,
            }
        }
    }

    /// Rebuild the heading index from the current buffer content.
    pub fn rebuild_index(&mut self) {
        let parser = self.parser() as *const dyn HeadingParser;
        // SAFETY: parser points to either a static or self.custom_parser which
        // lives as long as self. We only read through the pointer during this call.
        let parser_ref = unsafe { &*parser };
        self.headings = build_heading_index(self.editor.content(), parser_ref, &self.headings);
        self.dirty = false;
    }

    /// Mark the index as needing a rebuild.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Rebuild if dirty. Call this before rendering or querying headings.
    pub fn ensure_index(&mut self) {
        if self.dirty {
            self.rebuild_index();
        }
    }

    /// Find the heading index at or before the current cursor line.
    pub fn current_heading_idx(&self) -> Option<usize> {
        heading_at_or_before(&self.headings, self.editor.cursor_line())
    }

    /// Total number of lines in the buffer.
    pub fn line_count(&self) -> usize {
        self.editor.line_count()
    }

    /// Buffer lines.
    pub fn lines(&self) -> &[String] {
        self.editor.content()
    }

    /// Get the heading syntax in use.
    pub fn syntax(&self) -> HeadingSyntax {
        self.syntax
    }
}

impl Default for OutlineState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_text_builds_index() {
        let mut state = OutlineState::new();
        state.load_text("* A\nbody\n** B\n");
        assert_eq!(state.headings.len(), 2);
        assert_eq!(state.headings[0].title, "A");
        assert_eq!(state.headings[1].title, "B");
    }

    #[test]
    fn current_heading_at_body() {
        let mut state = OutlineState::new();
        state.load_text("* A\nbody line\n** B\n");
        state.editor.set_cursor(1, 0); // body line
        assert_eq!(state.current_heading_idx(), Some(0));
    }

    #[test]
    fn markdown_syntax() {
        let mut state = OutlineState::with_syntax(HeadingSyntax::Markdown);
        state.load_text("# Top\n## Sub\n");
        assert_eq!(state.headings.len(), 2);
        assert_eq!(state.headings[0].level, 1);
        assert_eq!(state.headings[1].level, 2);
    }
}
