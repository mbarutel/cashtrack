use anyhow::Result;

use crate::{
    models::{DateRange, Report},
    State,
};

pub fn report(state: &mut State, range: Option<DateRange>) -> Result<Report> {
    let transaction = super::transactions(state, range)?;
    let transfers = state.config.transfer_categories();
    Ok(Report::new(&transaction, &transfers))
}
