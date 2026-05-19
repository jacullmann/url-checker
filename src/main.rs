mod engine;

use clap::Parser;
use engine::{Cli, Commands, handle_check, handle_prepare};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prepare => handle_prepare(),
        Commands::Check { url } => handle_check(&url),
    }
}
