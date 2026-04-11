//! Integration tests for rat-inline.

use rat_inline::{InlineMarkdown, InlineRenderer, InlineText, InlineView, NodeKey};
use std::sync::{Arc, Mutex};

/// Test: render two identical frames, second produces empty diff.
#[test]
fn identical_frames_produce_empty_diff() {
    let mut renderer = InlineRenderer::new(40);

    let view1 = InlineView::new().text("Hello, world!").text("Second line.");
    renderer.rebuild(view1);
    let output1 = renderer.render();
    assert!(!output1.is_empty(), "first render should produce output");

    // Rebuild with identical content.
    let view2 = InlineView::new().text("Hello, world!").text("Second line.");
    renderer.rebuild(view2);
    let output2 = renderer.render();
    assert!(
        output2.is_empty(),
        "identical frame should produce no diff output"
    );
}

/// Test: markdown + text in a view tree, render produces output.
#[test]
fn markdown_and_text_render() {
    let mut renderer = InlineRenderer::new(60);

    let view = InlineView::new()
        .push(InlineMarkdown::new("# Title\n\nSome **bold** text."))
        .text("Plain footer.");
    renderer.rebuild(view);
    let output = renderer.render();
    assert!(!output.is_empty());

    // Verify DEC sync wrapping.
    assert!(
        output.starts_with(b"\n"),
        "should start with newlines for growth"
    );
    let output_str = String::from_utf8_lossy(&output);
    assert!(
        output_str.contains("\x1b[?2026h"),
        "should contain DEC sync start"
    );
    assert!(
        output_str.contains("\x1b[?2026l"),
        "should contain DEC sync end"
    );
}

/// Test: keyed nodes preserve identity across rebuilds.
#[test]
fn keyed_nodes_reconcile_across_rebuilds() {
    let mut renderer = InlineRenderer::new(40);

    // Frame 1: two keyed messages.
    let view1 = InlineView::new()
        .keyed("msg-a", InlineText::new("Message A"))
        .keyed("msg-b", InlineText::new("Message B"));
    renderer.rebuild(view1);
    let _output1 = renderer.render();
    assert_eq!(renderer.node_count(), 2);

    // Frame 2: reversed order, same keys.
    let view2 = InlineView::new()
        .keyed("msg-b", InlineText::new("Message B"))
        .keyed("msg-a", InlineText::new("Message A"));
    renderer.rebuild(view2);
    let _output2 = renderer.render();
    assert_eq!(renderer.node_count(), 2);
}

/// Test: commit callback fires when content exceeds viewport.
#[test]
fn commit_callback_fires_on_overflow() {
    let mut renderer = InlineRenderer::new(40);

    let committed_keys: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let keys_clone = committed_keys.clone();
    renderer.on_commit(move |key: &NodeKey| {
        keys_clone.lock().unwrap().push(key.0.clone());
    });

    // Build a view with many keyed nodes (each 1 line).
    let view = InlineView::new().each(0..20, |v, i| {
        v.keyed(format!("line-{i}"), InlineText::new(format!("Line {i}")))
    });
    renderer.rebuild(view);
    let _output = renderer.render();

    // Simulate a small viewport — process commits as if terminal is 5 rows.
    renderer.process_commits(5);

    let keys = committed_keys.lock().unwrap();
    // With 20 lines and viewport 5, the first 15 should be committed.
    assert!(!keys.is_empty(), "should have committed some nodes");
    // First committed key should be "line-0".
    assert_eq!(keys[0], "line-0");
}

/// Test: changing content produces minimal diff (only changed cells).
#[test]
fn changed_content_produces_partial_diff() {
    let mut renderer = InlineRenderer::new(40);

    let view1 = InlineView::new().text("AAAA");
    renderer.rebuild(view1);
    let output1 = renderer.render();

    let view2 = InlineView::new().text("AABA");
    renderer.rebuild(view2);
    let output2 = renderer.render();

    // Second output should be smaller than first (only the changed cell).
    assert!(
        output2.len() < output1.len(),
        "diff output ({}) should be smaller than full render ({})",
        output2.len(),
        output1.len()
    );
}
