mod cli;
mod config;
mod db;
mod models;
mod services;
mod state;
mod tui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

use crate::state::State;

pub fn run() -> Result<()> {
    let cli = Cli::try_parse()?;
    let mut state = State::new()?;

    match cli.command {
        None => tui::run(&mut state)?,
        Some(Command::Report { time_period }) => cli::report(&mut state, time_period)?,
        Some(Command::List { time_period }) => cli::list(&mut state, time_period)?,
        Some(Command::Import { csv_path, bank }) => cli::import(&mut state, &csv_path, bank)?,
    }

    Ok(())
}
