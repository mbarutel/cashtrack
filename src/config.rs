use anyhow::{Context, Result};
use serde::Deserialize;
use std::{io::ErrorKind, path::PathBuf};

const DEFAULT_CONFIG: &str = "\
# cashtrack category rules
# Each rule maps keywords found in a
# transaction's description to a category.
categories:
    - title: groceries
      keywords: [supermarket, store]
      priority: 1
    - title: eating out
      keywords: [mcdonalds, sushi]
      priority: 1
";

#[derive(Debug, Deserialize)]
pub struct Config {
    categories: Vec<Category>,
}

impl Config {
    pub fn new(file_path: PathBuf) -> Result<Config> {
        let contents = match std::fs::read_to_string(&file_path) {
            Ok(contents) => contents,
            Err(err) if err.kind() == ErrorKind::NotFound => {
                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent)
                        .with_context(|| format!("{}", parent.display()))?;
                }

                std::fs::write(&file_path, DEFAULT_CONFIG)
                    .with_context(|| format!("{}", file_path.display()))?;

                DEFAULT_CONFIG.to_string()
            }
            Err(err) => return Err(err).with_context(|| format!("{}", file_path.display())),
        };

        let config =
            serde_yaml::from_str(&contents).with_context(|| format!("{}", file_path.display()))?;

        Ok(config)
    }

    pub fn categories(&self) -> &Vec<Category> {
        &self.categories
    }
}

#[derive(Debug, Deserialize)]
pub struct Category {
    title: String,
    keywords: Vec<String>,
    priority: u32,
}

impl Category {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn keywords(&self) -> &Vec<String> {
        &self.keywords
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }
}
