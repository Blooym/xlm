mod commands;
mod includes;

use anyhow::Result;
use clap::Parser;
use commands::{install_steam_tool::InstallSteamToolCommand, launch::LaunchCommand};

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

    /// Disable XLM's inbuilt self-updater. May result in an outdated binary if left enabled.
    #[cfg(not(debug_assertions))]
    #[clap(global = true, default_value_t = false, long = "xlm-updater-disable")]
    xlm_updater_disable: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();

    // Ensure the binary is up to date from GitHub releases.
    #[cfg(not(debug_assertions))]
    if !args.xlm_updater_disable {
        tokio::task::spawn_blocking(move || {
            use self_update::cargo_crate_version;
            let result = self_update::backends::github::Update::configure()
                .repo_owner(&args.xlm_updater_repo_name)
                .repo_name(&args.xlm_updater_repo_owner)
                .bin_name(env!("CARGO_PKG_NAME"))
                .show_output(false)
                .no_confirm(true)
                .current_version(cargo_crate_version!())
                .build()
                .unwrap()
                .update();
            if let Err(result) = result {
                eprintln!("Failed to auto-update: {:?}", result);
            };
        })
        .await?;
    }

    // Run the command.
    match args.command {
        Command::Launch(cmd) => cmd.run().await,
        Command::InstallSteamTool(cmd) => cmd.run().await,
    }
}
