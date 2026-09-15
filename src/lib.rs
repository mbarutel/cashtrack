mod cli;
mod commands;
mod config;
mod db;
mod models;
mod state;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

use crate::state::State;

pub fn run() -> Result<()> {
    let cli = Cli::try_parse()?;
    let mut state = State::new()?;

    match cli.command {
        Command::Report { time_period } => commands::report(&mut state, time_period)?,
        Command::List { time_period } => commands::list(&mut state, time_period)?,
        Command::Import { csv_path, bank } => commands::import(&mut state, &csv_path, bank)?,
    }

    Ok(())
}
