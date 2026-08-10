use clap::Parser;

mod cli;
mod commands;
mod db;
mod models;

use cli::Command;

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn run() -> MyResult<()> {
    let cli = cli::Cli::try_parse()?;

    match cli.command {
        Command::Report { time_period } => commands::report(time_period)?,
        Command::List { time_period } => commands::list(time_period)?,
        Command::Import { path } => commands::import(&path)?,
    }

    Ok(())
}
