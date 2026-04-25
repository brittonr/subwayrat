# rat-typst

`rat-typst` exports `ratatui` render output to standalone Typst documents. The default font is `DejaVu Sans Mono`; override `TypstExportOptions::font_family` when your docs use another monospace font.

Use it when you want visual documentation, golden snapshots, or printable exports for subwayrat widgets without launching a terminal emulator.

```rust,no_run
use rat_typst::render_to_typst;
use ratatui::widgets::Paragraph;

let typst = render_to_typst(80, 24, |frame| {
    frame.render_widget(Paragraph::new("hello typst"), frame.area());
})?;
std::fs::write("widget.typ", typst)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```
