# rat-typst

`rat-typst` exports Ratatui render output to standalone Typst documents.

The default font is `DejaVu Sans Mono`. Set `TypstExportOptions::font_family` to use another monospace font.

Use this crate for visual documentation, golden snapshots, and printable widget exports. It does not require a terminal emulator.

```rust,no_run
use rat_typst::render_to_typst;
use ratatui::widgets::Paragraph;

let typst = render_to_typst(80, 24, |frame| {
    frame.render_widget(Paragraph::new("hello typst"), frame.area());
})?;
std::fs::write("widget.typ", typst)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```
