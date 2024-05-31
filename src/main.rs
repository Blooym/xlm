mod commands;

use clap::Parser;
use commands::launch::LaunchCommand;

#[derive(Debug, Clone, Parser)]
enum Command {
    Launch(LaunchCommand),
}

#[derive(Debug, Clone, Parser)]
struct Arguments {
    #[clap(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() {
    let args = Arguments::parse();
    match args.command {
        Command::Launch(cmd) => cmd.run().await,
    }
}
