# Real Time Note Taker
[![Crates.io](https://img.shields.io/crates/v/real_time_note_taker.svg)](https://crates.io/crates/real_time_note_taker)
[![Documentation](https://docs.rs/real_time_note_taker/badge.svg)](https://docs.rs/real_time_note_taker)
[![CI](https://github.com/Data-Forge-Solutions/real_time_note_taker/actions/workflows/CI.yml/badge.svg)](https://github.com/Data-Forge-Solutions/real_time_note_taker/actions)
[![License](https://img.shields.io/crates/l/real_time_note_taker)](LICENSE)
[![GitHub](https://img.shields.io/github/stars/Data-Forge-Solutions/real_time_note_taker?style=social)](https://github.com/Data-Forge-Solutions/real_time_note_taker)

**RTNT** is a terminal user interface for taking timestamped notes with support for syncing with external clocks. It was originally developed for taking notes during flight tests, but it can be useful for many other situations. **RTNT** allows you to capture the time of an event with a key press, and describe it later. This is useful when many complicated events are happening in rapid succession. Notes can be organized into titled sections and exported to CSV for further processing. All active key bindings are always visible at the bottom of the UI.

![Demo of RTNT](readme_resources/demo.gif)

## Features

- Capture the time an event happens with a key press and describe it later
- Millisecond accurate timestamps
- Section markers to organize discussions
- Edit existing entries
- Save and load notes from CSV
- Fully keyboard driven with customizable bindings for maximum speed and efficiency
- Manual time hacks for syncing with external clocks
- Current time with source indicator shown in the status line

## Installation

Add the binary with Cargo once the crate is published:

```bash
cargo install real_time_note_taker
```

## Running

```bash
cargo run --release
```

Additional CLI options are available:

- `--save-dir <PATH>` to override the default save location
- `--config <FILE>` to load key bindings from a custom file
- `-v/--verbose` increase logging output
- `completions <SHELL>` generate shell completion scripts

### Automatic file mode

Passing `--file <PATH>` will load notes from the given file and save them back on exit.

### Custom key bindings

Press the key shown as `Keys` in the help line to open the binding menu. Use the arrow keys to select an action and press <kbd>Enter</kbd> to assign a new key. Bindings are stored in `keybindings.json` inside the configuration directory (typically `~/.config/rtnt`).

### Configuration file

The `keybindings.json` file contains a mapping of actions to key names. An example configuration looks like:

```json
{
  "up": "Up",
  "down": "Down",
  "edit": "e",
  "new_note": "Enter",
  "new_section": "s",
  "save": "w",
  "load": "l",
  "file_menu": "f",
  "quit": "q",
  "cancel": "Esc",
  "bindings": "b",
  "theme": "t",
  "time_hack": "h"
}
```

## Library Usage

```rust
use real_time_note_taker::{run, App};

fn main() -> std::io::Result<()> {
    let app = App::new();
    let _app = run(app)?;
    Ok(())
}
```

## Saving location

Files are written to [`App::default_save_dir`] which resolves to a platform appropriate directory and is created on first use.

## License

This project is licensed under the MIT or Apache-2.0 licenses at your option.
