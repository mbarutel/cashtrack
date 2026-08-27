use serde::Deserialize;

use crate::MyResult;

#[derive(Debug, Deserialize)]
pub struct Config {
    rules: Vec<CategoryRule>,
}

#[derive(Debug, Deserialize)]
pub struct CategoryRule {
    subcategory: String,
    keywords: Vec<String>,
    priority: u32,
}

impl Config {
    pub fn new(file_path: &str) -> MyResult<Config> {
        let contents =
            std::fs::read_to_string(file_path).map_err(|err| format!("{}: {}", file_path, err))?;
        let config =
            serde_yaml::from_str(&contents).map_err(|err| format!("{}: {}", file_path, err))?;

        Ok(config)
    }

    pub fn rules(&self) -> &Vec<CategoryRule> {
        &self.rules
    }
}

impl CategoryRule {
    fn subcategory(&self) -> &str {
        &self.subcategory
    }

    fn keywords(&self) -> &Vec<String> {
        &self.keywords
    }

    fn priority(&self) -> u32 {
        self.priority
    }
}
