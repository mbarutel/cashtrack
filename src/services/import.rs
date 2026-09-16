use anyhow::{Context, Result};
use std::{
    path::Path,
    time::{Duration, Instant},
};

use crate::{
    models::{Bank, NewTransaction},
    services::categorizer::categorizer,
    State,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSummary {
    pub parsed: usize,
    pub inserted: usize,
    pub duplicate: usize,
    pub elapsed: Duration,
}

pub fn import(state: &mut State, path: &Path, bank: Bank) -> Result<ImportSummary> {
    let start = Instant::now();

    let mut transactions: Vec<NewTransaction> = bank
        .parse(path)
        .with_context(|| format!("importing {}", path.display()))?
        .into_iter()
        .map(|row| NewTransaction {
            date: row.date,
            amount: row.amount,
            category: categorizer(state.config.categories(), &row.description),
            description: row.description,
            bank: bank.to_string(),
        })
        .collect();

    transactions.sort_by_key(|t| t.date);

    let parsed = transactions.len();
    let inserted = state.db.insert_transactions(&transactions)?;

    Ok(ImportSummary {
        parsed,
        inserted,
        duplicate: parsed - inserted,
        elapsed: start.elapsed(),
    })
}
