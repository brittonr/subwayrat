use rat_scrolltile::{compute_layout, nav, SizeConstraint, Strip, StripConfig};

fn main() {
    let mut strip = Strip::new(StripConfig::default());

    // Column 0: single sidebar.
    let sidebar = strip.insert_window(0, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.resize_column(0, SizeConstraint::Fixed(20));

    // Column 1: editor (top) + terminal (bottom).
    let editor = strip.insert_window(
        1,
        0,
        SizeConstraint::default(),
        SizeConstraint::Proportion(2.0),
    );
    let terminal = strip.insert_window(
        1,
        1,
        SizeConstraint::default(),
        SizeConstraint::Proportion(1.0),
    );
    strip.resize_column(1, SizeConstraint::Proportion(1.0));

    // Column 2: preview pane.
    let preview = strip.insert_window(2, 0, SizeConstraint::default(), SizeConstraint::default());
    strip.resize_column(2, SizeConstraint::Fixed(30));

    // Focus the editor.
    strip.focus_set(editor);

    // Compute layout for an 80×24 terminal.
    let result = compute_layout(&strip, 80, 24);

    println!("Strip extent: {} cells", result.strip_extent);
    println!("Scroll offset: {}", result.scroll_offset);
    println!();

    let names = [
        (sidebar, "sidebar"),
        (editor, "editor"),
        (terminal, "terminal"),
        (preview, "preview"),
    ];

    println!("Strip-space rects:");
    for (id, name) in &names {
        if let Some(rect) = result.window_rects.get(id) {
            println!(
                "  {:<10} x={:<3} y={:<3} w={:<3} h={:<3}",
                name, rect.x, rect.y, rect.width, rect.height
            );
        }
    }

    println!();
    println!("Visible windows (viewport-local):");
    for vw in &result.visible {
        let name = names
            .iter()
            .find(|(id, _)| *id == vw.id)
            .map(|(_, n)| *n)
            .unwrap_or("?");
        println!(
            "  {:<10} x={:<3} y={:<3} w={:<3} h={:<3} fully_visible={}",
            name, vw.rect.x, vw.rect.y, vw.rect.width, vw.rect.height, vw.fully_visible,
        );
    }

    // Navigate right.
    println!();
    nav::focus_right(&mut strip, 80, 24);
    println!("After focus_right: focused = {:?}", strip.focused());
    let name = names
        .iter()
        .find(|(id, _)| Some(*id) == strip.focused())
        .map(|(_, n)| *n)
        .unwrap_or("none");
    println!("  → {}", name);
}
