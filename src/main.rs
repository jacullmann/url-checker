mod engine;

use anyhow::Result;
use clap::Parser;
use engine::{Cli, Commands, handle_check, handle_prepare};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prepare => handle_prepare()?,
        Commands::Check { url } => handle_check(&url)?,
    }

    Ok(())
}
