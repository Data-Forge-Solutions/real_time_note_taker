# real_time_note_taker

A terminal user interface application for taking timestamped notes in real time. Press `Enter` to begin a note, type your text, and press `Enter` again to save. Press `Esc` to cancel a note. Quit the application with `q`.

## Running

```
cargo run --release
```

## Library Usage

```
use real_time_note_taker::{run, App};

fn main() -> std::io::Result<()> {
    let app = App::new();
    run(app)
}
```
