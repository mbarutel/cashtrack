use std::path::Path;

use crate::{
    MyResult,
    cli::TimePeriod,
    db::{self, list_all_transactions},
    models::Transaction,
};

pub fn list(time_period: Option<TimePeriod>) -> MyResult<()> {
    match time_period.unwrap_or(TimePeriod::Weekly) {
        TimePeriod::Weekly => {
            println!("Weekly");
            let mut conn = db::open(Path::new("cashtrack.db"))?;
            let transactions = list_all_transactions(&mut conn)?;
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
