# Real Time Note Taker

Real Time Note Taker (RTNT) is a terminal user interface application for recording timestamped notes. Notes are grouped into optional sections and stored in CSV format for easy processing by other tools. All active key bindings are shown at the bottom of the interface.

## Features

- Millisecond accurate timestamps
- Section markers to organize discussions
- Edit existing entries
- Save and load notes to CSV files
- Fully keyboard driven with customizable bindings

## Running

```bash
cargo run --release
```

### Custom key bindings

Key bindings can be changed programmatically by constructing [`App`](src/app.rs) with [`KeyBindings`] using `App::with_keybindings` or by calling `App::set_keybindings` on an existing instance.

### Automatic file mode

Passing `--file <PATH>` to the binary will load notes from the given file and automatically save them back on exit.

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
