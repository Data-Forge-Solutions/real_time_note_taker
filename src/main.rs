#![warn(clippy::pedantic)]
use clap::{ArgAction, Parser};
use real_time_note_taker::{run, App};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Show version information
    #[arg(short = 'v', long = "version-info", alias = "v", action = ArgAction::SetTrue)]
    version_info: bool,

    /// Load notes from this file and save them on exit
    #[arg(long)]
    file: Option<std::path::PathBuf>,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    if cli.version_info {
        print_version_info();
        return Ok(());
    }
    let mut app = if let Some(ref file) = cli.file {
        App::load_from_file(file).unwrap_or_else(|_| App::new())
    } else {
        App::new()
    };
    app = run(app)?;
    if let Some(file) = cli.file {
        app.save_to_file(file)?;
    }
    Ok(())
}

fn print_version_info() {
    println!("rtnt {}", env!("CARGO_PKG_VERSION"));
}

#[cfg(test)]
mod tests {
    // The CLI behavior is tested in `tests/cli.rs` as an integration test so that
    // `assert_cmd` can locate the compiled binary. This module is intentionally
    // left empty.
}
