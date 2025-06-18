# viron 📝⚡️

**viron** is a modern, fast, and extensible text editor inspired by Vim, written in Rust. It aims to provide a familiar modal editing experience with a focus on performance, reliability, and customization. 🦀

## Features ✨

- **Vim-like modal editing**: Efficient keyboard-driven editing with normal, insert, and visual modes. ⌨️
- **Syntax highlighting**: Powered by [tree-sitter](https://tree-sitter.github.io/tree-sitter/) for accurate and fast syntax parsing. 🌈
- **Configurable themes**: Easily switch between beautiful color themes (see the `themes/` directory). viron themes follow the [VS Code theme format](https://code.visualstudio.com/api/extension-guides/color-theme), so you can use or adapt existing VS Code themes. 🎨
- **Asynchronous operations**: Smooth editing experience using async Rust and [tokio](https://tokio.rs/). 🚀
- **Cross-platform terminal support**: Built on [crossterm](https://crates.io/crates/crossterm) for compatibility with most terminals. 🖥️

## Getting Started 🛠️

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2024 or later) 🦀

### Build

```sh
cargo build --release
```

### Run

```sh
cargo run -- <file>
```

Replace `<file>` with the path to the file you want to edit.

### Example

```sh
cargo run -- README.md
```

## Configuration ⚙️

- Edit `config.toml` to customize editor settings.
- Add or modify themes in the `themes/` directory.

## Project Structure 🗂️

- `src/` — Main source code
  - `buffer/` — Gap buffer implementation
  - `editor/` — Editor core, commands, rendering, and search
  - `lsp/` — Language Server Protocol support
  - `theme/` — Theme management
  - `utils/` — Utility functions
- `themes/` — Color themes in JSON format
- `config.toml` — Editor configuration

## Dependencies 📦

- [anyhow](https://crates.io/crates/anyhow)
- [crossterm](https://crates.io/crates/crossterm)
- [tree-sitter](https://crates.io/crates/tree-sitter)
- [tokio](https://crates.io/crates/tokio)
- [serde](https://crates.io/crates/serde)
- [regex](https://crates.io/crates/regex)

## Contributing 🤝

Contributions are welcome! Please open issues or pull requests to help improve viron.

## License 📄

This project is licensed under the MIT License.
