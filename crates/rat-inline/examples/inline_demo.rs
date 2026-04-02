//! Demo: inline scrollback rendering for agent-like output.
//!
//! Run with: cargo run -p rat-inline --example inline_demo

use rat_inline::{InlineMarkdown, InlineRenderer, InlineView};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let width = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80);
    let mut renderer = InlineRenderer::new(width);

    // Simulate an agent conversation streaming into scrollback.
    let messages = [
        "**User**: What does this crate do?",
        "**Assistant**: `rat-inline` renders styled content into terminal scrollback.\n\nIt uses:\n- Frame diffing to minimize ANSI output\n- DEC synchronized output to prevent tearing\n- A reconciler from `ratcore` to preserve state across rebuilds",
        "**User**: Show me an example.",
        "**Assistant**: Here's a code block:\n\n```rust\nlet view = InlineView::new()\n    .text(\"Hello!\")\n    .keyed(\"msg-0\", InlineMarkdown::new(source));\nrenderer.rebuild(view);\nlet output = renderer.render();\n```\n\nThe renderer diffs each frame and only emits changed cells.",
    ];

    for (i, _msg) in messages.iter().enumerate() {
        // Build view with all messages so far.
        let view = InlineView::new()
            .each(messages[..=i].iter().enumerate(), |v, (j, m)| {
                v.keyed(format!("msg-{j}"), InlineMarkdown::new(*m))
            });
        renderer.rebuild(view);
        let output = renderer.render();
        stdout.write_all(&output)?;
        stdout.flush()?;

        if i < messages.len() - 1 {
            thread::sleep(Duration::from_millis(800));
        }
    }

    // Reset style at end.
    write!(stdout, "\x1b[0m\n")?;
    stdout.flush()?;
    Ok(())
}
