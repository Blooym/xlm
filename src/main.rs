mod commands;
mod includes;
mod ui;

use anyhow::Result;
use clap::Parser;
use commands::{install_steam_tool::InstallSteamToolCommand, launch::LaunchCommand};
use log::debug;
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::{env::temp_dir, fs::File};

#[derive(Debug, Clone, Parser)]
enum Command {
    Launch(Box<LaunchCommand>),
    InstallSteamTool(InstallSteamToolCommand),
    #[clap(hide = true)]
    LaunchUI,
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about)]
struct Arguments {
    #[clap(subcommand)]
    command: Command,

    /// The name of the GitHub repository owner that XLM should attempt to self-update from.
    #[cfg(not(debug_assertions))]
    #[clap(
        global = true,
        default_value = "Blooym",
        long = "xlm-updater-repo-owner"
    )]
    xlm_updater_repo_owner: String,

    /// The name of the GitHub repository that XLM should attempt to self-update from.
    #[cfg(not(debug_assertions))]
    #[clap(global = true, default_value = "xlm", long = "xlm-updater-repo-name")]
    xlm_updater_repo_name: String,

    /// Disable XLM's inbuilt self-updater. May result in an outdated binary.
    ///
    /// This should only be disabled if your connection to GitHub is poor or ratelimited.
    #[cfg(not(debug_assertions))]
    #[clap(global = true, default_value_t = false, long = "xlm-updater-disable")]
    xlm_updater_disable: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
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

    debug!("XLM v{}", env!("CARGO_PKG_VERSION"));

    // Ensure the binary is up to date from GitHub releases.
    #[cfg(not(debug_assertions))]
    if !args.xlm_updater_disable {
        tokio::task::spawn_blocking(move || {
            use log::info;
            use self_update::cargo_crate_version;
            info!("Running XLM self-updater");
            let result = self_update::backends::github::Update::configure()
                .repo_owner(&args.xlm_updater_repo_owner)
                .repo_name(&args.xlm_updater_repo_name)
                .bin_name(env!("CARGO_PKG_NAME"))
                .no_confirm(true)
                .current_version(cargo_crate_version!())
                .build()
                .unwrap()
                .update();
            info!("XLM self-updater ran successfully");
            if let Err(result) = result {
                eprintln!("XLM failed to auto-update: {:?}", result);
            };
        })
        .await?;
    }

    // Run the command.
    match args.command {
        Command::Launch(cmd) => cmd.run().await,
        Command::InstallSteamTool(cmd) => cmd.run().await,
        Command::LaunchUI => Ok(ui::launch_ui_main()),
    }
}
