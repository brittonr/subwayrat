use std::fs;
use std::io::ErrorKind;
use std::process::Command;

use rat_typst::{TypstExportError, TypstExportOptions, render_to_typst_with};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

const TEST_WIDTH: u16 = 32;
const TEST_HEIGHT: u16 = 6;
const TEMP_FILE_PREFIX: &str = "rat_typst_cli_compile";
const TYPST_BINARY: &str = "typst";
const TYPST_COMPILE_COMMAND: &str = "compile";
const TYPST_EXTENSION: &str = "typ";
const PDF_EXTENSION: &str = "pdf";
const EXPECTED_TEXT: &str = "rat-typst ok";
const ERROR_FONT_FAMILY: &str = " ";

#[test]
fn generated_typst_compiles_with_typst_cli_when_available() {
    let document = render_to_typst_with(
        TEST_WIDTH,
        TEST_HEIGHT,
        &TypstExportOptions::default(),
        |frame| {
            let block = Block::default()
                .title(" rat-typst ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let area = frame.area();
            let inner = block.inner(area);
            frame.render_widget(block, area);
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled(
                        EXPECTED_TEXT,
                        Style::default()
                            .fg(Color::LightGreen)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(vec![
                        Span::styled("underline", Modifier::UNDERLINED),
                        Span::raw(" + "),
                        Span::styled("strike", Modifier::CROSSED_OUT),
                    ]),
                ]),
                inner,
            );
        },
    );
    assert!(document.is_ok());
    let document = document.unwrap_or_default();
    for character in EXPECTED_TEXT.chars() {
        assert!(
            document.contains(&format!("#raw(\"{character}\")")),
            "missing exported character {character:?}",
        );
    }

    let temp_dir = std::env::temp_dir();
    let unique_name = format!("{TEMP_FILE_PREFIX}_{}", std::process::id());
    let input_path = temp_dir.join(format!("{unique_name}.{TYPST_EXTENSION}"));
    let output_path = temp_dir.join(format!("{unique_name}.{PDF_EXTENSION}"));
    fs::write(&input_path, document).expect("write typst input");

    let output = Command::new(TYPST_BINARY)
        .arg(TYPST_COMPILE_COMMAND)
        .arg(&input_path)
        .arg(&output_path)
        .output();

    let output = match output {
        Ok(output) => output,
        Err(error) if error.kind() == ErrorKind::NotFound => return,
        Err(error) => panic!("failed to run typst: {error}"),
    };

    assert!(
        output.status.success(),
        "typst compile failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    let output_metadata = fs::metadata(&output_path).expect("compiled pdf exists");
    assert!(output_metadata.len() > 0);

    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn invalid_options_fail_before_running_typst() {
    let options = TypstExportOptions::default().with_font_family(ERROR_FONT_FAMILY);

    let document = render_to_typst_with(TEST_WIDTH, TEST_HEIGHT, &options, |_frame| {});

    assert_eq!(document, Err(TypstExportError::EmptyFontFamily));
}
