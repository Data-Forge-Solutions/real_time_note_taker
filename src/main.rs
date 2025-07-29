#![warn(clippy::pedantic)]
use clap::Parser;
use real_time_note_taker::{run, App};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {}

fn main() -> std::io::Result<()> {
    let _ = Cli::parse();
    let app = App::new();
    run(app)
}

#[cfg(test)]
mod tests {
    // The CLI behavior is tested in `tests/cli.rs` as an integration test so that
    // `assert_cmd` can locate the compiled binary. This module is intentionally
    // left empty.
}
