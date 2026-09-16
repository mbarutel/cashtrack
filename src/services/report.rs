use anyhow::Result;

use crate::{
    models::{DateRange, Report},
    State,
};

pub fn report(state: &mut State, range: Option<DateRange>) -> Result<Report> {
    let transaction = super::transactions(state, range)?;
    Ok(Report::new(transaction))
}
