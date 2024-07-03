mod commands;
mod includes;

use anyhow::Result;
use clap::Parser;
use commands::{install_steam_tool::InstallSteamToolCommand, launch::LaunchCommand};
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::{env::temp_dir, fs::File};

#[derive(Debug, Clone, Parser)]
enum Command {
    Launch(LaunchCommand),
    InstallSteamTool(InstallSteamToolCommand),
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about)]
struct Arguments {
    #[clap(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create(temp_dir().join(format!("{}.log", env!("CARGO_PKG_NAME")))).unwrap(),
        ),
    ])?;

    // Run the command.
    match args.command {
        Command::Launch(cmd) => cmd.run().await,
        Command::InstallSteamTool(cmd) => cmd.run().await,
    }
}
