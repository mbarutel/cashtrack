mod command;
mod text;

use crate::models::{Bank, TimePeriod};
use clap::{Parser, Subcommand};
pub use command::{import, list, report};
use std::path::PathBuf;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    Report {
        #[command(subcommand)]
        time_period: Option<TimePeriod>,
    },
    List {
        #[command(subcommand)]
        time_period: Option<TimePeriod>,
    },
    Import {
        #[arg(short, long)]
        csv_path: PathBuf,
        #[arg(short, long, value_enum)]
        bank: Bank,
    },
}
