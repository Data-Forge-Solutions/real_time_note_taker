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
    use assert_cmd::Command;

    #[test]
    fn runs_help() {
        let mut cmd = Command::cargo_bin("rtnt").unwrap();
        let assert = cmd.arg("--help").assert();
        assert.success();
    }
}
