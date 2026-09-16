use anyhow::Result;
use chrono::Local;
use std::path::Path;

use crate::{
    cli::text,
    models::{Bank, DateRange, TimePeriod},
    services, State,
};

fn range_for(period: Option<TimePeriod>) -> Option<DateRange> {
    period.map(|p| p.range_containing(Local::now().date_naive()))
}

pub fn list(state: &mut State, period: Option<TimePeriod>) -> Result<()> {
    let range = range_for(period);
    let transactions = services::transactions(state, range)?;
    text::print_transactions(transactions);
    Ok(())
}

pub fn report(state: &mut State, period: Option<TimePeriod>) -> Result<()> {
    let range = range_for(period);
    let report = services::report(state, range)?;
    text::print_report(&report);
    Ok(())
}

pub fn import(state: &mut State, path: &Path, bank: Bank) -> Result<()> {
    let summary = services::import(state, path, bank)?;
    text::print_import_summary(&summary);
    Ok(())
}
