use std::{fmt::Display, str::FromStr};

use chrono::NaiveDate;
use rust_decimal::Decimal;

// pub struct TransactionCsvRow {
//     pub date: String,
//     pub amount: String,
//     pub description: String,
// }

pub struct TransactionDbRow {
    pub id: i64,
    pub date: String,
    pub amount: String,
    pub category: String,
    pub description: String,
    pub bank: String,
}

#[derive(Debug)]
pub enum Direction {
    Inflow,
    Outflow,
    Noflow,
}

pub struct Transaction {
    pub date: NaiveDate,
    pub direction: Direction,
    pub amount: Decimal,
    pub category: String,
    pub description: String,
    pub bank: String,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(match self {
            Direction::Inflow => "Inflow",
            Direction::Outflow => "Outflow",
            Direction::Noflow => "Noflow",
        })
    }
}

impl TryFrom<TransactionDbRow> for Transaction {
    type Error = String;

    fn try_from(value: TransactionDbRow) -> Result<Self, Self::Error> {
        let amount = rust_decimal::Decimal::from_str(&value.amount).map_err(|e| format!("{e}"))?;
        let date = NaiveDate::from_str(&value.date).map_err(|e| format!("{e}"))?;

        Ok(Self {
            date,
            direction: Direction::from(amount),
            amount: amount.abs(),
            category: value.category,
            description: value.description,
            bank: value.bank,
        })
    }
}

impl From<Decimal> for Direction {
    fn from(value: Decimal) -> Self {
        if value == Decimal::ZERO {
            Direction::Noflow
        } else if value > Decimal::ZERO {
            Direction::Inflow
        } else {
            Direction::Outflow
        }
    }
}
