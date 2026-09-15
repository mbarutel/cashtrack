use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Debug)]
pub struct Transaction {
    pub date: NaiveDate,
    pub amount: Decimal,
    pub category: String,
    pub description: String,
    pub bank: String,
}
