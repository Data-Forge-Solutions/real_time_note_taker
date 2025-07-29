# real_time_note_taker

A terminal user interface application for taking timestamped notes in real time. Press `Enter` to begin a note, type your text, and press `Enter` again to save. Press `s` to start a section and enter a title. Use the arrow keys to select a previous entry and press `e` to edit it. Press `Esc` to cancel an entry. Press `w` to save all notes to a file and `l` to load from a file. Files are saved under a platform‑appropriate directory shown in the prompt and created automatically if it does not exist. Quit the application with `q`. Use `--file <PATH>` to load and save notes automatically.

When loading, an overlay window lists the files in this directory for selection.

## Running

```
cargo run --release
```

## Library Usage

```
use real_time_note_taker::{run, App};

fn main() -> std::io::Result<()> {
    let app = App::new();
    let _app = run(app)?;
    Ok(())
}
```
