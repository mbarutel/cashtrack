use anyhow::Result;
use chrono::Local;
use rust_decimal::Decimal;

use crate::{
    commands::list::format_decimal,
    models::{Report, TimePeriod},
    State,
};

pub fn report(state: &mut State, time_period: Option<TimePeriod>) -> Result<()> {
    match time_period {
        Some(time_period) => {
            let date_range = time_period.range_containing(Local::now().date_naive());

            let transactions = state
                .db
                .list_transactions(&date_range.start, &date_range.end)?;

            let report = Report::new(transactions);

            print_report(&report);
        }
        None => {
            println!("List All");
            let transactions = state.db.list_all_transactions()?;

            let report = Report::new(transactions);

            print_report(&report);
        }
    }

    Ok(())
}

fn print_report(report: &Report) {
    println!(
        "Report for {} to {}",
        report.start_date.format("%B %d, %Y").to_string(),
        report.end_date.format("%B %d, %Y").to_string()
    );

    println!("Inflow  {}", format_decimal(report.total_inflow));
    println!("Outflow {}", format_decimal(report.total_outflow));
    println!(
        "Balance {}",
        format_decimal(report.total_inflow - report.total_outflow)
    );

    let mut categories: Vec<(&String, &Decimal)> = report.categories_total_flow.iter().collect();

    // let mut outflow_categories: Vec<(&String, &Decimal)> = report
    //     .categories_total_flow
    //     .iter()
    //     .filter(|cat| *cat.1 < Decimal::ZERO)
    //     .collect();

    categories.sort_by(|a, b| a.1.cmp(b.1));
    // inflow_categories.sort_by(|a, b| b.1.value.cmp(&a.1.value));
    // outflow_categories.sort_by(|a, b| b.1.value.cmp(&a.1.value));

    for cat in categories {
        println!("  {}: {}", cat.0, cat.1);
    }
    // println!();
    // for cat in outflow_categories {
    //     println!(
    //         "  {}: {}{}",
    //         cat.0,
    //         if cat.1.direction == Direction::Outflow {
    //             "-"
    //         } else {
    //             ""
    //         },
    //         cat.1.value
    //     );
    // }
}
