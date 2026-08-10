use std::path::Path;

use crate::{MyResult, cli::TimePeriod, db::Database, models::Transaction};

pub fn list(time_period: Option<TimePeriod>) -> MyResult<()> {
    let mut db = Database::new(Path::new("cashtrack.db"))?;

    match time_period.unwrap_or(TimePeriod::Weekly) {
        TimePeriod::Weekly => {
            println!("Weekly");
            let transactions = db.list_all_transactions()?;
            print_transactions(transactions);
        }
        TimePeriod::Fortnightly => {
            println!("Fortnightly")
        }
        TimePeriod::Monthly => {
            println!("Monthly")
        }
        TimePeriod::Yearly => {
            println!("Yearly")
        }
    }

    Ok(())
}

fn print_transactions(transactions: Vec<Transaction>) {
    let amount_width = transactions
        .iter()
        .map(|t| format!("{:.2}", t.amount).len())
        .max()
        .unwrap_or(0);
    let category_width = transactions
        .iter()
        .map(|t| t.category.len())
        .max()
        .unwrap_or(0);

    for transaction in transactions {
        println!(
            "{} | {:>amount_width$.2} | {:>category_width$} | {}",
            transaction.date, transaction.amount, transaction.category, transaction.description
        );
    }
}
