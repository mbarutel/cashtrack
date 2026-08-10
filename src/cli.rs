use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
        path: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum TimePeriod {
    Weekly,
    Fortnightly,
    Monthly,
    Yearly,
}
