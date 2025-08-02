#![warn(clippy::pedantic)]
use clap::{ArgAction, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use log::LevelFilter;
use real_time_note_taker::{run, App, KeyBindings};

#[derive(Subcommand)]
enum Commands {
    /// Generate shell completion script for the specified shell
    Completions { shell: Shell },
}

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Load notes from this file and save them on exit
    #[arg(long)]
    file: Option<std::path::PathBuf>,

    /// Override the default directory for saved files
    #[arg(long)]
    save_dir: Option<std::path::PathBuf>,

    /// Path to an alternative key binding configuration file
    #[arg(long)]
    config: Option<std::path::PathBuf>,

    /// Increase logging verbosity (-v, -vv, etc.)
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    init_logger(cli.verbose);

    if let Some(Commands::Completions { shell }) = cli.command {
        let mut cmd = Cli::command();
        generate(shell, &mut cmd, "rtnt", &mut std::io::stdout());
        return Ok(());
    }

    let mut app = if let Some(ref file) = cli.file {
        App::load_from_file(file).unwrap_or_else(|_| App::new())
    } else {
        App::new()
    };

    if let Some(save_dir) = cli.save_dir {
        std::fs::create_dir_all(&save_dir)?;
        app.save_dir = save_dir;
    }

    if let Some(cfg) = cli.config {
        let keys = KeyBindings::load_or_default_from(&cfg);
        app.set_keybindings(keys);
    }

    app = run(app)?;
    if let Some(file) = cli.file {
        app.save_to_file(file)?;
    }
    Ok(())
}

fn init_logger(verbosity: u8) {
    let level = match verbosity {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    let _ = env_logger::Builder::from_default_env()
        .filter_level(level)
        .try_init();
}

#[cfg(test)]
mod tests {
    // The CLI behavior is tested in `tests/cli.rs` as an integration test so that
    // `assert_cmd` can locate the compiled binary. This module is intentionally
    // left empty.
}
