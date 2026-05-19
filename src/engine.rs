mod check;
mod cli;
pub mod config;
mod filter;
mod normalization;
mod prepare;

pub use check::handle_check;
pub use cli::{Cli, Commands};
pub use prepare::handle_prepare;
