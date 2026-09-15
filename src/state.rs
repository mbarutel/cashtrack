use std::path::PathBuf;

use crate::{config::Config, db::Database};
use anyhow::Result;
use etcetera::{AppStrategy, AppStrategyArgs};

pub struct State {
    pub db: Database,
    pub config: Config,
}

fn strategy() -> Result<impl AppStrategy> {
    Ok(etcetera::choose_app_strategy(AppStrategyArgs {
        top_level_domain: String::new(),
        author: String::new(),
        app_name: "cashtrack".to_string(),
    })?)
}

impl State {
    pub fn new() -> Result<Self> {
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
