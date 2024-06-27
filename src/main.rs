mod commands;
mod includes;

use anyhow::Result;
use clap::Parser;
use commands::{launch::LaunchCommand, setup::SetupCommand};

#[derive(Debug, Clone, Parser)]
enum Command {
    Launch(LaunchCommand),
    Setup(SetupCommand),
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about)]
struct Arguments {
    #[clap(subcommand)]
    command: Command,

    /// The name of the GitHub repository owner that XLCM should attempt to self-update from.
    #[cfg(not(debug_assertions))]
    #[clap(
        global = true,
        default_value = "Blooym",
        long = "xlcm-updater-repo-owner"
    )]
    xlcm_updater_repo_owner: String,

    /// The name of the GitHub repository that XLCM should attempt to self-update from.
    #[cfg(not(debug_assertions))]
    #[clap(global = true, default_value = "xlcm", long = "xclm-updater-repo-name")]
    xlcm_updater_repo_name: String,

    /// Disable XLCM's inbuilt self-updater. May result in an outdated binary if left enabled.
    #[cfg(not(debug_assertions))]
    #[clap(global = true, default_value_t = false, long = "xclm-updater-disable")]
    xlcm_updater_disable: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();

    // Ensure the binary is up to date from GitHub releases.
    #[cfg(not(debug_assertions))]
    if !args.xlcm_updater_disable {
        tokio::task::spawn_blocking(move || {
            use self_update::cargo_crate_version;
            let result = self_update::backends::github::Update::configure()
                .repo_owner(&args.xlcm_updater_repo_name)
                .repo_name(&args.xlcm_updater_repo_owner)
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
        Command::Setup(cmd) => cmd.run().await,
    }
}
