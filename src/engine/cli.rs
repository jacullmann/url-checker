use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "url-checker")]
#[command(about = "A fast CLI url-checker", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Build,
    Check { url: String },
}
