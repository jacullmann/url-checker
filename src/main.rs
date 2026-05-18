mod engine;

use clap::Parser;
use engine::{Cli, Commands, handle_build, handle_check};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build => handle_build(),
        Commands::Check { url } => handle_check(&url),
    }
}
