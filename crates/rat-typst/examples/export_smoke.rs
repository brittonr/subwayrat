use std::error::Error;

use rat_typst::render_to_typst;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

const EXAMPLE_WIDTH: u16 = 48;
const EXAMPLE_HEIGHT: u16 = 8;

fn main() -> Result<(), Box<dyn Error>> {
    let document = render_to_typst(EXAMPLE_WIDTH, EXAMPLE_HEIGHT, |frame| {
        let block = Block::default()
            .title(" rat-typst export ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let area = frame.area();
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let lines = vec![
            Line::from(Span::styled(
                "subwayrat widgets can export to Typst",
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("underlined", Modifier::UNDERLINED),
                Span::raw(" and "),
                Span::styled("crossed out", Modifier::CROSSED_OUT),
            ]),
        ];
        frame.render_widget(Paragraph::new(lines), inner);
    })?;

    print!("{document}");
    Ok(())
}
