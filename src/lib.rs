mod cli;
mod commands;
mod config;
mod db;
mod models;

use clap::Parser;
use cli::{Cli, Command};
use config::Config;
use db::Database;
use etcetera::{AppStrategy, AppStrategyArgs};
use std::path::PathBuf;

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct State {
    db: Database,
    config: Config,
}

fn strategy() -> MyResult<impl AppStrategy> {
    Ok(etcetera::choose_app_strategy(AppStrategyArgs {
        top_level_domain: String::new(),
        author: String::new(),
        app_name: "cashtrack".to_string(),
    })?)
}

impl State {
    fn new() -> MyResult<Self> {
        let strategy = strategy()?;

        let config_path = std::env::var_os("CASHTRACK_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|| strategy.in_config_dir("categories.yaml"));

        let db_path = std::env::var_os("CASHTRACK_DB")
            .map(PathBuf::from)
            .unwrap_or_else(|| strategy.in_data_dir("cashtrack.db"));

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
