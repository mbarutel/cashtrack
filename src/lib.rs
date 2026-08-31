mod cli;
mod commands;
mod config;
mod db;
mod models;

use clap::Parser;
use cli::Cli;
use cli::Command;
use config::Config;
use db::Database;

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct State {
    db: Database,
    config: Config,
}

impl State {
    fn new() -> MyResult<Self> {
        let config_path = "./categories.yaml";
        let db_path = "./cashtrack.db";

        Ok(Self {
            config: Config::new(config_path)?,
            db: Database::new(db_path)?,
        })
    }
}

pub fn run() -> MyResult<()> {
    let cli = Cli::try_parse()?;
    let mut state = State::new()?;

    match cli.command {
        Command::Report { time_period } => commands::report(&state, time_period)?,
        Command::List { time_period } => commands::list(&mut state, time_period)?,
        Command::Import { csv_path } => commands::import(&mut state, &csv_path)?,
    }

    Ok(())
}
