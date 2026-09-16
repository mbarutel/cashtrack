use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::models::{Bank, TimePeriod};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
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
