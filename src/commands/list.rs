use anyhow::Result;
use chrono::{Datelike, Days, Local, Months, NaiveDate, Weekday};
use rust_decimal::Decimal;

use crate::{
    models::{TimePeriod, Transaction},
    State,
};

pub fn list(state: &mut State, time_period: Option<TimePeriod>) -> Result<()> {
    match time_period {
        Some(time_period) => {
            let date_range = time_period.range_containing(Local::now().date_naive());

            let transactions = state
                .db
                .list_transactions(&date_range.start, &date_range.end)?;

            if transactions.is_empty() {
                println!("No transactions to print!");
            } else {
                print_transactions(transactions);
            }
        }
        None => {
            println!("List All");
            let transactions = state.db.list_all_transactions()?;

            print_transactions(transactions);
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

    let mut total_in = Decimal::from(0);
    let mut total_out = Decimal::from(0);

    for transaction in transactions {
        let direction;

        if transaction.amount > Decimal::ZERO {
            total_in += transaction.amount;
            direction = "Inflow";
        } else if transaction.amount < Decimal::ZERO {
            total_out += transaction.amount;
            direction = "Outflow";
        } else {
            continue;
        }

        println!(
            "{} | {:>10} | {:>amount_width$.2} | {:>category_width$}",
            transaction.date, direction, transaction.amount, transaction.category,
        );
    }

    println!(
        "\nTotal In:  ${}\nTotal Out: ${} \nRemaining: ${}",
        format_decimal(total_in),
        format_decimal(total_out.abs()),
        format_decimal(total_in - total_out.abs())
    );
}

pub fn format_decimal(amount: Decimal) -> String {
    let s = amount.round_dp(2).to_string();
    let (num, frac) = s.split_once('.').unwrap_or((s.as_str(), "00"));
    let (sign, int) = num.strip_prefix('-').map_or(("", num), |i| ("-", i));

    let mut out = String::new();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }

    format!("{sign}{out}.{frac:0<2}")
}
