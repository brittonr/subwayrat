# Vendored ratatypst notes

This crate vendors the core idea from [`sermuns/ratatypst`](https://github.com/sermuns/ratatypst), commit `95967a7fa33460b979ea832ae5f37a7c1021e3ae`.

Original `ratatypst-core` is a proof-of-concept `ratatui::backend::Backend` that captures a ratatui `Buffer` and serializes it as Typst. The upstream source snapshot lives in `vendor/ratatypst-core/`. The original project is released under the Unlicense; a copy is stored at `vendor/ratatypst-core/LICENSE`.

Local changes:

- expose a `rat-typst` exporter crate instead of a standalone git dependency;
- generate standalone Typst documents from buffers;
- preserve foreground, background, bold, italic, underline, strikethrough, hidden, and reversed styling where Typst supports it;
- add validation and tests for invalid export options;
- keep the backend no-I/O so callers can choose whether to write `.typ` or run `typst compile`.
