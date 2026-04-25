use std::error::Error;
use std::fmt::{self, Write};

use ratatui::Terminal;
use ratatui::buffer::{Buffer, Cell};
use ratatui::style::{Color, Modifier};
use unicode_width::UnicodeWidthStr;

use crate::backend::TypstBackend;

/// Measurement type used by Typst exporter options.
pub type ExportDimension = f32;

/// Default monospace font emitted in the Typst prelude.
pub const DEFAULT_FONT_FAMILY: &str = "DejaVu Sans Mono";
/// Default Typst text size in points.
pub const DEFAULT_FONT_SIZE_PT: ExportDimension = 10.0;
/// Default terminal cell width in em units.
pub const DEFAULT_CELL_WIDTH_EM: ExportDimension = 0.62;
/// Default terminal cell height in em units.
pub const DEFAULT_CELL_HEIGHT_EM: ExportDimension = 1.15;
/// Default Typst page margin in points.
pub const DEFAULT_PAGE_MARGIN_PT: ExportDimension = 0.0;

const TYPOGRAPHIC_ZERO_PT: ExportDimension = 0.0;
const DEFAULT_TEXT_RED: u8 = 220;
const DEFAULT_TEXT_GREEN: u8 = 220;
const DEFAULT_TEXT_BLUE: u8 = 220;
const BLACK_COMPONENT: u8 = 0;
const DARK_COMPONENT: u8 = 128;
const ANSI_YELLOW_GREEN_COMPONENT: u8 = 128;
const ANSI_GRAY_COMPONENT: u8 = 192;
const BRIGHT_COMPONENT: u8 = 255;
const ANSI_DARK_GRAY_COMPONENT: u8 = 128;
const ANSI_256_TABLE_LEN: usize = 16;
const ANSI_256_CUBE_START: u8 = 16;
const ANSI_256_CUBE_END: u8 = 231;
const ANSI_256_GRAYSCALE_START: u8 = 232;
const ANSI_256_CUBE_CHANNEL_COUNT: usize = 6;
const ANSI_256_CUBE_RED_DIVISOR: u8 = 36;
const ANSI_256_CUBE_GREEN_DIVISOR: u8 = 6;
const ANSI_256_LEVEL_0: u8 = 0;
const ANSI_256_LEVEL_1: u8 = 95;
const ANSI_256_LEVEL_2: u8 = 135;
const ANSI_256_LEVEL_3: u8 = 175;
const ANSI_256_LEVEL_4: u8 = 215;
const ANSI_256_LEVEL_5: u8 = 255;
const ANSI_256_GRAYSCALE_BASE: u8 = 8;
const ANSI_256_GRAYSCALE_STEP: u8 = 10;
const MIN_VISIBLE_CELL_WIDTH: usize = 1;
const BACKSLASH: char = '\\';
const DOUBLE_QUOTE: char = '"';
const NEWLINE: char = '\n';
const CARRIAGE_RETURN: char = '\r';
const TAB: char = '\t';
const SPACE: char = ' ';
const STRING_ESCAPE_BACKSLASH: &str = "\\\\";
const STRING_ESCAPE_QUOTE: &str = "\\\"";
const STRING_ESCAPE_NEWLINE: &str = "\\n";
const STRING_ESCAPE_CARRIAGE_RETURN: &str = "\\r";
const STRING_ESCAPE_TAB: &str = "\\t";
const EMPTY_EXPORT_WIDTH: u16 = 0;
const EMPTY_EXPORT_HEIGHT: u16 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RgbColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl RgbColor {
    const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

const DEFAULT_TEXT_COLOR: RgbColor =
    RgbColor::new(DEFAULT_TEXT_RED, DEFAULT_TEXT_GREEN, DEFAULT_TEXT_BLUE);
const ANSI_256_CUBE_LEVELS: [u8; ANSI_256_CUBE_CHANNEL_COUNT] = [
    ANSI_256_LEVEL_0,
    ANSI_256_LEVEL_1,
    ANSI_256_LEVEL_2,
    ANSI_256_LEVEL_3,
    ANSI_256_LEVEL_4,
    ANSI_256_LEVEL_5,
];
const ANSI_256_STANDARD: [RgbColor; ANSI_256_TABLE_LEN] = [
    RgbColor::new(BLACK_COMPONENT, BLACK_COMPONENT, BLACK_COMPONENT),
    RgbColor::new(DARK_COMPONENT, BLACK_COMPONENT, BLACK_COMPONENT),
    RgbColor::new(
        BLACK_COMPONENT,
        ANSI_YELLOW_GREEN_COMPONENT,
        BLACK_COMPONENT,
    ),
    RgbColor::new(
        ANSI_YELLOW_GREEN_COMPONENT,
        ANSI_YELLOW_GREEN_COMPONENT,
        BLACK_COMPONENT,
    ),
    RgbColor::new(BLACK_COMPONENT, BLACK_COMPONENT, DARK_COMPONENT),
    RgbColor::new(DARK_COMPONENT, BLACK_COMPONENT, DARK_COMPONENT),
    RgbColor::new(
        BLACK_COMPONENT,
        ANSI_YELLOW_GREEN_COMPONENT,
        ANSI_YELLOW_GREEN_COMPONENT,
    ),
    RgbColor::new(
        ANSI_GRAY_COMPONENT,
        ANSI_GRAY_COMPONENT,
        ANSI_GRAY_COMPONENT,
    ),
    RgbColor::new(
        ANSI_DARK_GRAY_COMPONENT,
        ANSI_DARK_GRAY_COMPONENT,
        ANSI_DARK_GRAY_COMPONENT,
    ),
    RgbColor::new(BRIGHT_COMPONENT, BLACK_COMPONENT, BLACK_COMPONENT),
    RgbColor::new(BLACK_COMPONENT, BRIGHT_COMPONENT, BLACK_COMPONENT),
    RgbColor::new(BRIGHT_COMPONENT, BRIGHT_COMPONENT, BLACK_COMPONENT),
    RgbColor::new(BLACK_COMPONENT, BLACK_COMPONENT, BRIGHT_COMPONENT),
    RgbColor::new(BRIGHT_COMPONENT, BLACK_COMPONENT, BRIGHT_COMPONENT),
    RgbColor::new(BLACK_COMPONENT, BRIGHT_COMPONENT, BRIGHT_COMPONENT),
    RgbColor::new(BRIGHT_COMPONENT, BRIGHT_COMPONENT, BRIGHT_COMPONENT),
];

/// Export settings for converting a ratatui [`Buffer`] into Typst.
#[derive(Debug, Clone, PartialEq)]
pub struct TypstExportOptions {
    /// Font family used for terminal cells.
    pub font_family: String,
    /// Font size in Typst points.
    pub font_size_pt: ExportDimension,
    /// Terminal cell width in em units.
    pub cell_width_em: ExportDimension,
    /// Terminal cell height in em units.
    pub cell_height_em: ExportDimension,
    /// Page margin in Typst points.
    pub page_margin_pt: ExportDimension,
    /// Foreground color used when ratatui cells have [`Color::Reset`].
    pub default_fg: Color,
    /// Background color used when ratatui cells have [`Color::Reset`].
    pub default_bg: Color,
    /// Whether trailing blank reset cells are omitted from each row.
    pub trim_trailing_blank_cells: bool,
}

impl TypstExportOptions {
    /// Build default exporter options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the monospace font family.
    pub fn with_font_family(mut self, font_family: impl Into<String>) -> Self {
        self.font_family = font_family.into();
        self
    }

    /// Set the font size in Typst points.
    pub fn with_font_size_pt(mut self, font_size_pt: ExportDimension) -> Self {
        self.font_size_pt = font_size_pt;
        self
    }

    /// Set terminal cell size in em units.
    pub fn with_cell_size_em(
        mut self,
        cell_width_em: ExportDimension,
        cell_height_em: ExportDimension,
    ) -> Self {
        self.cell_width_em = cell_width_em;
        self.cell_height_em = cell_height_em;
        self
    }

    /// Set default colors used for reset foreground/background cells.
    pub fn with_default_colors(mut self, default_fg: Color, default_bg: Color) -> Self {
        self.default_fg = default_fg;
        self.default_bg = default_bg;
        self
    }

    /// Enable or disable trimming of trailing blank reset cells.
    pub fn with_trim_trailing_blank_cells(mut self, trim: bool) -> Self {
        self.trim_trailing_blank_cells = trim;
        self
    }

    /// Validate options before export.
    pub fn validate(&self) -> Result<(), TypstExportError> {
        if self.font_family.trim().is_empty() {
            return Err(TypstExportError::EmptyFontFamily);
        }
        validate_positive_dimension("font_size_pt", self.font_size_pt)?;
        validate_positive_dimension("cell_width_em", self.cell_width_em)?;
        validate_positive_dimension("cell_height_em", self.cell_height_em)?;
        validate_non_negative_dimension("page_margin_pt", self.page_margin_pt)?;
        Ok(())
    }
}

impl Default for TypstExportOptions {
    fn default() -> Self {
        Self {
            font_family: DEFAULT_FONT_FAMILY.to_string(),
            font_size_pt: DEFAULT_FONT_SIZE_PT,
            cell_width_em: DEFAULT_CELL_WIDTH_EM,
            cell_height_em: DEFAULT_CELL_HEIGHT_EM,
            page_margin_pt: DEFAULT_PAGE_MARGIN_PT,
            default_fg: Color::Rgb(
                DEFAULT_TEXT_COLOR.red,
                DEFAULT_TEXT_COLOR.green,
                DEFAULT_TEXT_COLOR.blue,
            ),
            default_bg: Color::Reset,
            trim_trailing_blank_cells: true,
        }
    }
}

/// Error returned when Typst export options are invalid.
#[derive(Debug, Clone, PartialEq)]
pub enum TypstExportError {
    /// Font family was empty or whitespace.
    EmptyFontFamily,
    /// Numeric option was not finite.
    NonFiniteDimension { name: &'static str },
    /// Numeric option required a positive value.
    NonPositiveDimension {
        name: &'static str,
        value: ExportDimension,
    },
    /// Numeric option required a non-negative value.
    NegativeDimension {
        name: &'static str,
        value: ExportDimension,
    },
}

impl fmt::Display for TypstExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFontFamily => f.write_str("font_family must not be empty"),
            Self::NonFiniteDimension { name } => write!(f, "{name} must be finite"),
            Self::NonPositiveDimension { name, value } => {
                write!(f, "{name} must be positive, got {value}")
            }
            Self::NegativeDimension { name, value } => {
                write!(f, "{name} must be non-negative, got {value}")
            }
        }
    }
}

impl Error for TypstExportError {}

/// Render a ratatui frame into a standalone Typst document.
pub fn render_to_typst<F>(width: u16, height: u16, draw: F) -> Result<String, TypstExportError>
where
    F: FnOnce(&mut ratatui::Frame<'_>),
{
    render_to_typst_with(width, height, &TypstExportOptions::default(), draw)
}

/// Render a ratatui frame into Typst with caller-provided options.
pub fn render_to_typst_with<F>(
    width: u16,
    height: u16,
    options: &TypstExportOptions,
    draw: F,
) -> Result<String, TypstExportError>
where
    F: FnOnce(&mut ratatui::Frame<'_>),
{
    let backend = TypstBackend::new(width, height);
    let mut terminal = match Terminal::new(backend) {
        Ok(terminal) => terminal,
        Err(error) => match error {},
    };
    match terminal.draw(draw) {
        Ok(_) => {}
        Err(error) => match error {},
    }
    terminal.backend().to_typst_document_with(options)
}

/// Convert a ratatui [`Buffer`] into a standalone Typst document.
pub fn export_buffer_to_typst(
    buffer: &Buffer,
    options: &TypstExportOptions,
) -> Result<String, TypstExportError> {
    options.validate()?;
    Ok(render_typst_document(buffer, options))
}

fn render_typst_document(buffer: &Buffer, options: &TypstExportOptions) -> String {
    let mut output = String::new();
    write_prelude(&mut output, options);
    write_rows(&mut output, buffer, options);
    output
}

fn write_prelude(output: &mut String, options: &TypstExportOptions) {
    let page_margin = format_typst_dimension(options.page_margin_pt, "pt");
    let font_size = format_typst_dimension(options.font_size_pt, "pt");
    let cell_width = format_typst_dimension(options.cell_width_em, "em");
    let cell_height = format_typst_dimension(options.cell_height_em, "em");
    let font = escape_typst_string(&options.font_family);

    writeln!(output, "// Generated by rat-typst.").ok();
    writeln!(output, "#set page(margin: {page_margin})").ok();
    writeln!(output, "#set text(font: \"{font}\", size: {font_size})").ok();
    writeln!(
        output,
        "#set par(leading: {})",
        format_typst_dimension(TYPOGRAPHIC_ZERO_PT, "pt")
    )
    .ok();
    writeln!(output, "#let rat-cell-width = {cell_width}").ok();
    writeln!(output, "#let rat-cell-height = {cell_height}").ok();
    output.push(NEWLINE);
}

fn write_rows(output: &mut String, buffer: &Buffer, options: &TypstExportOptions) {
    let width = buffer.area.width;
    let height = buffer.area.height;
    if width == EMPTY_EXPORT_WIDTH || height == EMPTY_EXPORT_HEIGHT {
        return;
    }

    let row_width = usize::from(width);
    for row_cells in buffer.content.chunks(row_width) {
        write_row(output, row_cells, options);
        writeln!(output, "#linebreak()").ok();
    }
}

fn write_row(output: &mut String, row_cells: &[Cell], options: &TypstExportOptions) {
    let visible_end = visible_row_end(row_cells, options.trim_trailing_blank_cells);
    let mut skip_cells = 0usize;
    for cell in row_cells.iter().take(visible_end) {
        if skip_cells > 0 {
            skip_cells = skip_cells.saturating_sub(1);
            continue;
        }

        let symbol = display_symbol(cell);
        write_cell(output, cell, &symbol, options);
        let rendered_width = symbol_width(&symbol);
        skip_cells = rendered_width.saturating_sub(MIN_VISIBLE_CELL_WIDTH);
    }
}

fn visible_row_end(row_cells: &[Cell], trim_trailing_blank_cells: bool) -> usize {
    if !trim_trailing_blank_cells {
        return row_cells.len();
    }

    row_cells
        .iter()
        .rposition(|cell| !is_trimmable_cell(cell))
        .map_or(0, |index| index + 1)
}

fn is_trimmable_cell(cell: &Cell) -> bool {
    cell.symbol() == " "
        && cell.fg == Color::Reset
        && cell.bg == Color::Reset
        && cell.modifier.is_empty()
}

fn write_cell(output: &mut String, cell: &Cell, symbol: &str, options: &TypstExportOptions) {
    let style = effective_cell_style(cell, options);
    let raw_symbol = escape_typst_string(symbol);
    let mut content = String::new();

    write!(content, "#text(fill: {}", typst_color(style.fg)).ok();
    if style.bold {
        content.push_str(", weight: \"bold\"");
    }
    if style.italic {
        content.push_str(", style: \"italic\"");
    }
    write!(content, ")[#raw(\"{raw_symbol}\")]").ok();

    if style.underlined {
        content = format!("#underline[{content}]");
    }
    if style.crossed_out {
        content = format!("#strike[{content}]");
    }

    output.push_str("#box(width: rat-cell-width, height: rat-cell-height");
    if let Some(background) = style.bg {
        write!(output, ", fill: {}", typst_color(background)).ok();
    }
    write!(output, ")[{content}]").ok();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EffectiveCellStyle {
    fg: RgbColor,
    bg: Option<RgbColor>,
    bold: bool,
    italic: bool,
    underlined: bool,
    crossed_out: bool,
}

fn effective_cell_style(cell: &Cell, options: &TypstExportOptions) -> EffectiveCellStyle {
    let mut fg = color_or_default(cell.fg, options.default_fg).unwrap_or(DEFAULT_TEXT_COLOR);
    let mut bg = color_or_default(cell.bg, options.default_bg);

    if cell.modifier.contains(Modifier::REVERSED) {
        let reversed_fg = bg.unwrap_or(DEFAULT_TEXT_COLOR);
        let reversed_bg = Some(fg);
        fg = reversed_fg;
        bg = reversed_bg;
    }

    EffectiveCellStyle {
        fg,
        bg,
        bold: cell.modifier.contains(Modifier::BOLD),
        italic: cell.modifier.contains(Modifier::ITALIC),
        underlined: cell.modifier.contains(Modifier::UNDERLINED),
        crossed_out: cell.modifier.contains(Modifier::CROSSED_OUT),
    }
}

fn display_symbol(cell: &Cell) -> String {
    if cell.modifier.contains(Modifier::HIDDEN) {
        return SPACE.to_string();
    }
    sanitize_symbol(cell.symbol())
}

fn sanitize_symbol(symbol: &str) -> String {
    symbol
        .chars()
        .map(|character| {
            if character.is_control() {
                SPACE
            } else {
                character
            }
        })
        .collect()
}

fn symbol_width(symbol: &str) -> usize {
    symbol.width().max(MIN_VISIBLE_CELL_WIDTH)
}

fn color_or_default(color: Color, default: Color) -> Option<RgbColor> {
    match color {
        Color::Reset => color_to_rgb(default),
        _ => color_to_rgb(color),
    }
}

fn color_to_rgb(color: Color) -> Option<RgbColor> {
    match color {
        Color::Reset => None,
        Color::Black => Some(ANSI_256_STANDARD[0]),
        Color::Red => Some(ANSI_256_STANDARD[1]),
        Color::Green => Some(ANSI_256_STANDARD[2]),
        Color::Yellow => Some(ANSI_256_STANDARD[3]),
        Color::Blue => Some(ANSI_256_STANDARD[4]),
        Color::Magenta => Some(ANSI_256_STANDARD[5]),
        Color::Cyan => Some(ANSI_256_STANDARD[6]),
        Color::Gray => Some(ANSI_256_STANDARD[7]),
        Color::DarkGray => Some(ANSI_256_STANDARD[8]),
        Color::LightRed => Some(ANSI_256_STANDARD[9]),
        Color::LightGreen => Some(ANSI_256_STANDARD[10]),
        Color::LightYellow => Some(ANSI_256_STANDARD[11]),
        Color::LightBlue => Some(ANSI_256_STANDARD[12]),
        Color::LightMagenta => Some(ANSI_256_STANDARD[13]),
        Color::LightCyan => Some(ANSI_256_STANDARD[14]),
        Color::White => Some(ANSI_256_STANDARD[15]),
        Color::Rgb(red, green, blue) => Some(RgbColor::new(red, green, blue)),
        Color::Indexed(index) => Some(indexed_color_to_rgb(index)),
    }
}

fn indexed_color_to_rgb(index: u8) -> RgbColor {
    if usize::from(index) < ANSI_256_TABLE_LEN {
        return ANSI_256_STANDARD[usize::from(index)];
    }

    if index <= ANSI_256_CUBE_END {
        let cube_index = index - ANSI_256_CUBE_START;
        let red_index = usize::from(cube_index / ANSI_256_CUBE_RED_DIVISOR);
        let green_index =
            usize::from((cube_index % ANSI_256_CUBE_RED_DIVISOR) / ANSI_256_CUBE_GREEN_DIVISOR);
        let blue_index = usize::from(cube_index % ANSI_256_CUBE_GREEN_DIVISOR);
        return RgbColor::new(
            ANSI_256_CUBE_LEVELS[red_index],
            ANSI_256_CUBE_LEVELS[green_index],
            ANSI_256_CUBE_LEVELS[blue_index],
        );
    }

    let gray =
        ANSI_256_GRAYSCALE_BASE + (index - ANSI_256_GRAYSCALE_START) * ANSI_256_GRAYSCALE_STEP;
    RgbColor::new(gray, gray, gray)
}

fn typst_color(color: RgbColor) -> String {
    format!(
        "rgb(\"#{:02x}{:02x}{:02x}\")",
        color.red, color.green, color.blue
    )
}

fn escape_typst_string(input: &str) -> String {
    let mut escaped = String::new();
    for character in input.chars() {
        match character {
            BACKSLASH => escaped.push_str(STRING_ESCAPE_BACKSLASH),
            DOUBLE_QUOTE => escaped.push_str(STRING_ESCAPE_QUOTE),
            NEWLINE => escaped.push_str(STRING_ESCAPE_NEWLINE),
            CARRIAGE_RETURN => escaped.push_str(STRING_ESCAPE_CARRIAGE_RETURN),
            TAB => escaped.push_str(STRING_ESCAPE_TAB),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn format_typst_dimension(value: ExportDimension, unit: &str) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}{unit}")
    } else {
        format!("{value}{unit}")
    }
}

fn validate_positive_dimension(
    name: &'static str,
    value: ExportDimension,
) -> Result<(), TypstExportError> {
    validate_finite_dimension(name, value)?;
    if value <= 0.0 {
        return Err(TypstExportError::NonPositiveDimension { name, value });
    }
    Ok(())
}

fn validate_non_negative_dimension(
    name: &'static str,
    value: ExportDimension,
) -> Result<(), TypstExportError> {
    validate_finite_dimension(name, value)?;
    if value < 0.0 {
        return Err(TypstExportError::NegativeDimension { name, value });
    }
    Ok(())
}

fn validate_finite_dimension(
    name: &'static str,
    value: ExportDimension,
) -> Result<(), TypstExportError> {
    if !value.is_finite() {
        return Err(TypstExportError::NonFiniteDimension { name });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::widgets::{Block, Borders, Paragraph};

    use super::*;

    const TEST_WIDTH: u16 = 8;
    const TEST_HEIGHT: u16 = 3;
    const FIRST_COLUMN: u16 = 0;
    const FIRST_ROW: u16 = 0;
    const SECOND_COLUMN: u16 = 1;
    const WIDE_BUFFER_WIDTH: u16 = 2;
    const WIDE_BUFFER_HEIGHT: u16 = 1;
    const INVALID_DIMENSION: ExportDimension = 0.0;
    const NON_FINITE_DIMENSION: ExportDimension = ExportDimension::NAN;
    const NEGATIVE_DIMENSION: ExportDimension = -1.0;

    #[test]
    fn exports_buffer_as_standalone_typst_document() {
        let mut buffer = Buffer::empty(Rect::new(FIRST_COLUMN, FIRST_ROW, TEST_WIDTH, TEST_HEIGHT));
        buffer.set_string(
            FIRST_COLUMN,
            FIRST_ROW,
            "Hi",
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::Indexed(17))
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );

        let document = export_buffer_to_typst(&buffer, &TypstExportOptions::default());

        assert!(document.is_ok());
        let document = document.unwrap_or_default();
        assert!(document.contains("#set page"));
        assert!(document.contains(DEFAULT_FONT_FAMILY));
        assert!(document.contains("#raw(\"H\")"));
        assert!(document.contains("#raw(\"i\")"));
        assert!(document.contains("rgb(\"#008080\")"));
        assert!(document.contains("rgb(\"#00005f\")"));
        assert!(document.contains("weight: \"bold\""));
        assert!(document.contains("#underline"));
    }

    #[test]
    fn render_helper_exports_ratatui_widget() {
        let document = render_to_typst(TEST_WIDTH, TEST_HEIGHT, |frame| {
            frame.render_widget(
                Paragraph::new("OK").block(Block::default().borders(Borders::ALL)),
                frame.area(),
            );
        });

        assert!(document.is_ok());
        let document = document.unwrap_or_default();
        assert!(document.contains("#raw(\"O\")"));
        assert!(document.contains("#raw(\"K\")"));
    }

    #[test]
    fn skips_overwritten_cells_for_wide_symbols() {
        let mut buffer = Buffer::empty(Rect::new(
            FIRST_COLUMN,
            FIRST_ROW,
            WIDE_BUFFER_WIDTH,
            WIDE_BUFFER_HEIGHT,
        ));
        buffer[(FIRST_COLUMN, FIRST_ROW)].set_symbol("界");
        buffer[(SECOND_COLUMN, FIRST_ROW)].set_symbol("x");

        let document = export_buffer_to_typst(&buffer, &TypstExportOptions::default());

        assert!(document.is_ok());
        let document = document.unwrap_or_default();
        assert!(document.contains("#raw(\"界\")"));
        assert!(!document.contains("#raw(\"x\")"));
    }

    #[test]
    fn escapes_typst_string_content() {
        let mut buffer = Buffer::empty(Rect::new(FIRST_COLUMN, FIRST_ROW, TEST_WIDTH, TEST_HEIGHT));
        buffer[(FIRST_COLUMN, FIRST_ROW)].set_symbol("\\");
        buffer[(SECOND_COLUMN, FIRST_ROW)].set_symbol("\"");

        let document = export_buffer_to_typst(&buffer, &TypstExportOptions::default());

        assert!(document.is_ok());
        let document = document.unwrap_or_default();
        assert!(document.contains("#raw(\"\\\\\")"));
        assert!(document.contains("#raw(\"\\\"\")"));
    }

    #[test]
    fn rejects_empty_font_family() {
        let buffer = Buffer::empty(Rect::new(FIRST_COLUMN, FIRST_ROW, TEST_WIDTH, TEST_HEIGHT));
        let options = TypstExportOptions::default().with_font_family(" ");

        let document = export_buffer_to_typst(&buffer, &options);

        assert_eq!(document, Err(TypstExportError::EmptyFontFamily));
    }

    #[test]
    fn rejects_non_positive_cell_dimensions() {
        let options = TypstExportOptions::default()
            .with_cell_size_em(INVALID_DIMENSION, DEFAULT_CELL_HEIGHT_EM);

        let result = options.validate();

        assert_eq!(
            result,
            Err(TypstExportError::NonPositiveDimension {
                name: "cell_width_em",
                value: INVALID_DIMENSION,
            }),
        );
    }

    #[test]
    fn rejects_non_finite_font_size() {
        let options = TypstExportOptions::default().with_font_size_pt(NON_FINITE_DIMENSION);

        let result = options.validate();

        assert_eq!(
            result,
            Err(TypstExportError::NonFiniteDimension {
                name: "font_size_pt",
            }),
        );
    }

    #[test]
    fn rejects_negative_page_margin() {
        let options = TypstExportOptions {
            page_margin_pt: NEGATIVE_DIMENSION,
            ..TypstExportOptions::default()
        };

        let result = options.validate();

        assert_eq!(
            result,
            Err(TypstExportError::NegativeDimension {
                name: "page_margin_pt",
                value: NEGATIVE_DIMENSION,
            }),
        );
    }
}
