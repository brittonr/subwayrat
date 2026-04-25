use core::{convert::Infallible, fmt, iter};

use ratatui::backend::{Backend, ClearType, WindowSize};
use ratatui::buffer::{Buffer, Cell};
use ratatui::layout::{Position, Rect, Size};

use crate::export::{TypstExportError, TypstExportOptions, export_buffer_to_typst};

/// Ratatui backend that captures frames for Typst export.
///
/// This is the vendored `ratatypst-core::TypstBackend` role, adapted for
/// subwayrat: it performs no terminal I/O and exposes standalone document
/// export methods.
#[derive(Debug, Clone)]
pub struct TypstBackend {
    buffer: Buffer,
    scrollback: Buffer,
    pos: Position,
}

impl TypstBackend {
    /// Create a capture backend with a fixed terminal-cell size.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            buffer: Buffer::empty(Rect::new(0, 0, width, height)),
            scrollback: Buffer::empty(Rect::new(0, 0, width, 0)),
            pos: Position::ORIGIN,
        }
    }

    /// Return the current captured frame buffer.
    pub const fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Return captured scrollback produced via [`Backend::append_lines`].
    pub const fn scrollback(&self) -> &Buffer {
        &self.scrollback
    }

    /// Serialize the captured frame with default export options.
    pub fn to_typst_document(&self) -> Result<String, TypstExportError> {
        self.to_typst_document_with(&TypstExportOptions::default())
    }

    /// Serialize the captured frame with caller-provided options.
    pub fn to_typst_document_with(
        &self,
        options: &TypstExportOptions,
    ) -> Result<String, TypstExportError> {
        export_buffer_to_typst(&self.buffer, options)
    }

    /// Compatibility helper matching the original ratatypst API shape.
    pub fn to_vec(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl fmt::Display for TypstBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let document = self.to_typst_document().map_err(|_| fmt::Error)?;
        f.write_str(&document)
    }
}

impl Backend for TypstBackend {
    type Error = Infallible;

    fn draw<'a, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'a ratatui::buffer::Cell)>,
    {
        for (x, y, cell) in content {
            self.buffer[(x, y)] = cell.clone();
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn get_cursor_position(&mut self) -> Result<Position, Self::Error> {
        Ok(self.pos)
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> Result<(), Self::Error> {
        self.pos = position.into();
        Ok(())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.buffer.reset();
        Ok(())
    }

    fn clear_region(&mut self, clear_type: ClearType) -> Result<(), Self::Error> {
        let width = self.buffer.area.width;
        let height = self.buffer.area.height;
        if width == 0 || height == 0 {
            return Ok(());
        }

        match clear_type {
            ClearType::All => self.clear(),
            ClearType::AfterCursor => {
                let index_after_cursor = self.buffer.index_of(self.pos.x, self.pos.y) + 1;
                reset_region(&mut self.buffer.content[index_after_cursor..]);
                Ok(())
            }
            ClearType::BeforeCursor => {
                let cursor_index = self.buffer.index_of(self.pos.x, self.pos.y);
                reset_region(&mut self.buffer.content[..cursor_index]);
                Ok(())
            }
            ClearType::CurrentLine => {
                let line_start_index = self.buffer.index_of(0, self.pos.y);
                let line_end_index = self.buffer.index_of(width - 1, self.pos.y);
                reset_region(&mut self.buffer.content[line_start_index..=line_end_index]);
                Ok(())
            }
            ClearType::UntilNewLine => {
                let cursor_index = self.buffer.index_of(self.pos.x, self.pos.y);
                let line_end_index = self.buffer.index_of(width - 1, self.pos.y);
                reset_region(&mut self.buffer.content[cursor_index..=line_end_index]);
                Ok(())
            }
        }
    }

    fn append_lines(&mut self, line_count: u16) -> Result<(), Self::Error> {
        let Position { x: cur_x, y: cur_y } = self.get_cursor_position()?;
        let Rect { width, height, .. } = self.buffer.area;

        if line_count == 0 || width == 0 || height == 0 {
            return Ok(());
        }

        let new_cursor_x = cur_x.saturating_add(1).min(width.saturating_sub(1));
        let max_y = height.saturating_sub(1);
        let lines_after_cursor = max_y.saturating_sub(cur_y);

        if line_count > lines_after_cursor {
            let scroll_by: usize = usize::from(line_count - lines_after_cursor);
            let width_cells: usize = usize::from(width);
            let cells_to_scrollback = self.buffer.content.len().min(width_cells * scroll_by);

            append_to_scrollback(
                &mut self.scrollback,
                self.buffer.content.splice(
                    0..cells_to_scrollback,
                    iter::repeat_with(Default::default).take(cells_to_scrollback),
                ),
            );
            self.buffer.content.rotate_left(cells_to_scrollback);
            append_to_scrollback(
                &mut self.scrollback,
                iter::repeat_with(Default::default)
                    .take(width_cells * scroll_by - cells_to_scrollback),
            );
        }

        let new_cursor_y = cur_y.saturating_add(line_count).min(max_y);
        self.set_cursor_position(Position::new(new_cursor_x, new_cursor_y))?;
        Ok(())
    }

    fn size(&self) -> Result<Size, Self::Error> {
        Ok(self.buffer.area.as_size())
    }

    fn window_size(&mut self) -> Result<WindowSize, Self::Error> {
        Ok(WindowSize {
            columns_rows: self.buffer.area.as_size(),
            pixels: Size::ZERO,
        })
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn reset_region(region: &mut [Cell]) {
    for cell in region {
        cell.reset();
    }
}

fn append_to_scrollback(scrollback: &mut Buffer, cells: impl IntoIterator<Item = Cell>) {
    let width = usize::from(scrollback.area.width);
    if width == 0 {
        return;
    }

    scrollback.content.extend(cells);
    let max_scrollback_rows = usize::from(u16::MAX);
    let new_height = (scrollback.content.len() / width).min(max_scrollback_rows);
    let keep_from = scrollback
        .content
        .len()
        .saturating_sub(width * max_scrollback_rows);
    scrollback.content.drain(0..keep_from);
    scrollback.area.height = new_height as u16;
}

#[cfg(test)]
mod tests {
    use ratatui::backend::Backend;
    use ratatui::layout::{Position, Rect};
    use ratatui::style::{Color, Style};
    use ratatui::widgets::{Paragraph, Widget};

    use super::*;

    const SMALL_WIDTH: u16 = 6;
    const SMALL_HEIGHT: u16 = 3;
    const ZERO_SIZE: u16 = 0;
    const APPENDED_LINES: u16 = 2;
    const FIRST_COLUMN: u16 = 0;
    const FIRST_ROW: u16 = 0;

    #[test]
    fn backend_captures_widget_render() {
        let mut backend = TypstBackend::new(SMALL_WIDTH, SMALL_HEIGHT);
        let mut buffer = Buffer::empty(Rect::new(
            FIRST_COLUMN,
            FIRST_ROW,
            SMALL_WIDTH,
            SMALL_HEIGHT,
        ));
        Paragraph::new("Hi")
            .style(Style::default().fg(Color::Cyan))
            .render(buffer.area, &mut buffer);

        let updates = buffer.content.iter().enumerate().map(|(index, cell)| {
            let x = (index % usize::from(SMALL_WIDTH)) as u16;
            let y = (index / usize::from(SMALL_WIDTH)) as u16;
            (x, y, cell)
        });
        let draw_result = backend.draw(updates);

        assert!(draw_result.is_ok());
        assert_eq!(
            backend
                .buffer()
                .cell(Position::new(FIRST_COLUMN, FIRST_ROW))
                .unwrap()
                .symbol(),
            "H"
        );
        let document = backend.to_typst_document();
        assert!(document.is_ok());
        let document = document.unwrap_or_default();
        assert!(document.contains("#raw(\"H\")"));
        assert!(document.contains("#raw(\"i\")"));
    }

    #[test]
    fn zero_sized_backend_rejects_noop_line_append_without_panic() {
        let mut backend = TypstBackend::new(ZERO_SIZE, ZERO_SIZE);
        let append_result = backend.append_lines(APPENDED_LINES);

        assert!(append_result.is_ok());
        assert_eq!(backend.buffer().area.width, ZERO_SIZE);
        assert_eq!(backend.buffer().area.height, ZERO_SIZE);
    }
}
