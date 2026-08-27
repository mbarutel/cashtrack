use crate::{MyResult, State, cli::TimePeriod, models::Transaction};

// TODO: We need helper function that get the start date of week, month and year
// along with test suites.

pub fn list(state: &mut State, time_period: Option<TimePeriod>) -> MyResult<()> {
    match time_period {
        Some(time_period) => match time_period {
            TimePeriod::Weekly => {
                println!("Weekly");

                let transactions = state.db.list_all_transactions()?;

                print_transactions(transactions);
            }
            TimePeriod::Fortnightly => {
                println!("Fortnightly");
                let transactions = state.db.list_all_transactions()?;

                print_transactions(transactions);
            }
            TimePeriod::Monthly => {
                println!("Monthly");
                let transactions = state.db.list_all_transactions()?;

                print_transactions(transactions);
            }
            TimePeriod::Yearly => {
                println!("Yearly");
                let transactions = state.db.list_all_transactions()?;

                print_transactions(transactions);
            }
        },
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

    for transaction in transactions {
        println!(
            "{} | {:>10} | {:>amount_width$.2} | {:>category_width$} | {}",
            transaction.date,
            transaction.direction,
            transaction.amount,
            transaction.category,
            transaction.description
        );
    }
}
