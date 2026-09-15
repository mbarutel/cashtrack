use std::collections::HashMap;

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::Transaction;

#[derive(Debug, Default)]
pub struct Report {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub total_inflow: Decimal,
    pub total_outflow: Decimal,
    pub categories_total_flow: HashMap<String, Decimal>,
}

impl Report {
    pub fn new(transactions: Vec<Transaction>) -> Self {
        let mut report = Self::default();
        if transactions.is_empty() {
            return report;
        }

        let mut iter = transactions.into_iter();

        report.start_date = iter.next().unwrap().date;
        report.end_date = report.start_date;

        for tran in iter {
            if tran.date < report.start_date {
                report.start_date = tran.date
            } else if tran.date > report.end_date {
                report.end_date = tran.date
            }

            if tran.amount > Decimal::ZERO {
                report.total_inflow += tran.amount
            } else if tran.amount < Decimal::ZERO {
                report.total_outflow += tran.amount
            }

            let mut entry = report
                .categories_total_flow
                .entry(tran.category)
                .or_insert(Decimal::ZERO);

            entry += tran.amount;
        }

        report
    }
}
