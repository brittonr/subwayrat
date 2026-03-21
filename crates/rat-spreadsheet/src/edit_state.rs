//! Cell editing state management.
//!
//! Contains the EditState struct for managing inline cell editing,
//! including text buffer, cursor position, and edit lifecycle.

use crate::cell::CellValue;

/// State for cell editing
#[derive(Debug, Clone)]
pub struct EditState {
    /// Whether we're currently editing
    pub editing: bool,
    /// Text buffer being edited
    pub buffer: String,
    /// Cursor position within buffer
    pub cursor_pos: usize,
    /// Previous value for undo/cancel
    pub previous_value: Option<CellValue>,
}

impl EditState {
    /// Create a new edit state
    pub fn new() -> Self {
        Self {
            editing: false,
            buffer: String::new(),
            cursor_pos: 0,
            previous_value: None,
        }
    }

    /// Start editing with initial text
    pub fn start_edit(&mut self, initial: String) {
        self.editing = true;
        self.buffer = initial;
        self.cursor_pos = self.buffer.len();
    }

    /// Insert a character at cursor position
    pub fn insert_char(&mut self, ch: char) {
        self.buffer.insert(self.cursor_pos, ch);
        self.cursor_pos += ch.len_utf8();
    }

    /// Delete character at cursor position
    pub fn delete_char(&mut self) {
        if self.cursor_pos < self.buffer.len() {
            self.buffer.remove(self.cursor_pos);
        }
    }

    /// Delete character before cursor (backspace)
    pub fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            let mut iter = self.buffer.char_indices().rev();
            if let Some((i, _)) = iter.find(|&(i, _)| i < self.cursor_pos) {
                self.buffer.remove(i);
                self.cursor_pos = i;
            } else {
                self.buffer.remove(0);
                self.cursor_pos = 0;
            }
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            let mut iter = self.buffer.char_indices().rev();
            if let Some((i, _)) = iter.find(|&(i, _)| i < self.cursor_pos) {
                self.cursor_pos = i;
            } else {
                self.cursor_pos = 0;
            }
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.buffer.len() {
            let mut iter = self.buffer.char_indices();
            if let Some((i, _)) = iter.find(|&(i, _)| i > self.cursor_pos) {
                self.cursor_pos = i;
            } else {
                self.cursor_pos = self.buffer.len();
            }
        }
    }

    /// Commit the current buffer and reset editing state
    pub fn commit_buffer(&mut self) -> String {
        self.editing = false;
        let buffer = std::mem::take(&mut self.buffer);
        self.cursor_pos = 0;
        self.previous_value = None;
        buffer
    }

    /// Cancel editing and return previous value if any
    pub fn cancel(&mut self) -> Option<CellValue> {
        self.editing = false;
        self.buffer.clear();
        self.cursor_pos = 0;
        self.previous_value.take()
    }
}

impl Default for EditState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_state() {
        let mut edit = EditState::new();
        
        assert!(!edit.editing);
        
        edit.start_edit("hello".to_string());
        assert!(edit.editing);
        assert_eq!(edit.buffer, "hello");
        assert_eq!(edit.cursor_pos, 5);
        
        edit.insert_char('!');
        assert_eq!(edit.buffer, "hello!");
        assert_eq!(edit.cursor_pos, 6);
        
        edit.backspace();
        assert_eq!(edit.buffer, "hello");
        assert_eq!(edit.cursor_pos, 5);
        
        let result = edit.commit_buffer();
        assert_eq!(result, "hello");
        assert!(!edit.editing);
        assert!(edit.buffer.is_empty());
    }
}