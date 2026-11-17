use chrono::NaiveDate;
use rust_decimal::Decimal;

pub const UNCATEGORISED: &str = "Unknown";

#[derive(Debug)]
pub struct NewTransaction {
    pub date: NaiveDate,
    pub amount: Decimal,
    pub category: String,
    pub description: String,
    pub bank: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub id: i64,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub category: String,
    pub description: String,
    pub bank: String,
}
