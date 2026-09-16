use anyhow::Result;

use crate::{
    models::{DateRange, Transaction},
    State,
};

pub fn transactions(state: &mut State, range: Option<DateRange>) -> Result<Vec<Transaction>> {
    match range {
        Some(range) => state.db.list_transactions(&range.start, &range.end),
        None => state.db.list_all_transactions(),
    }
}
