//! Typst export support for ratatui widgets.
//!
//! This crate vendors the core `ratatypst` idea into subwayrat: render normal
//! ratatui widgets into a buffer, then serialize that buffer as a standalone
//! Typst document for screenshots, docs, and visual snapshot tests.

mod backend;
mod export;

pub use backend::TypstBackend;
pub use export::{
    DEFAULT_CELL_HEIGHT_EM, DEFAULT_CELL_WIDTH_EM, DEFAULT_FONT_FAMILY, DEFAULT_FONT_SIZE_PT,
    DEFAULT_PAGE_MARGIN_PT, ExportDimension, TypstExportError, TypstExportOptions,
    export_buffer_to_typst, render_to_typst, render_to_typst_with,
};
