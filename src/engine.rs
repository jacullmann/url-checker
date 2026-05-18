mod build;
mod check;
mod cli;
pub mod config;
mod filter;
mod normalization;

pub use build::handle_build;
pub use check::handle_check;
pub use cli::{Cli, Commands};
