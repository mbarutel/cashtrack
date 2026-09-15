use core::fmt;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Bank {
    Commonwealth,
    Westpac,
}

impl Bank {
    pub fn parse(self, path: &Path) -> Result<Vec<ParsedRow>> {
        let has_headers = match self {
            Self::Commonwealth => false,
            Self::Westpac => true,
        };

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(has_headers)
            .trim(csv::Trim::All)
            .from_path(path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        let mut rows = Vec::new();

        match self {
            Self::Commonwealth => {
                for result in reader.deserialize() {
                    let row: CommonwealthRow = result?;
                    rows.push(row.try_into()?);
                }
            }
            Self::Westpac => {
                for result in reader.deserialize() {
                    let row: WestpacRow = result?;
                    rows.push(row.try_into()?);
                }
            }
        }

        Ok(rows)
    }
}

impl fmt::Display for Bank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(match self {
            Bank::Commonwealth => "Commonwealth",
            Bank::Westpac => "Westpac",
        })
    }
}

pub struct ParsedRow {
    pub date: NaiveDate,
    pub amount: Decimal,
    pub description: String,
}

#[derive(Deserialize, Debug)]
struct CommonwealthRow(String, Decimal, String);

#[derive(Deserialize, Debug)]
struct WestpacRow {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Narrative")]
    narrative: String,
    #[serde(rename = "Debit Amount")]
    debit: Option<Decimal>,
    #[serde(rename = "Credit Amount")]
    credit: Option<Decimal>,
}

impl TryFrom<CommonwealthRow> for ParsedRow {
    type Error = chrono::ParseError;

    fn try_from(row: CommonwealthRow) -> Result<Self, Self::Error> {
        Ok(ParsedRow {
            date: NaiveDate::parse_from_str(&row.0, "%d/%m/%Y")?,
            amount: row.1,
            description: collapse_whitespace(&row.2),
        })
    }
}

impl TryFrom<WestpacRow> for ParsedRow {
    type Error = chrono::ParseError;

    fn try_from(row: WestpacRow) -> Result<Self, Self::Error> {
        let credit = row.credit.unwrap_or(Decimal::ZERO);
        let debit = row.debit.unwrap_or(Decimal::ZERO);

        Ok(ParsedRow {
            date: NaiveDate::parse_from_str(&row.date, "%d/%m/%Y")?,
            amount: credit - debit,
            description: collapse_whitespace(&row.narrative),
        })
    }
}

fn collapse_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
