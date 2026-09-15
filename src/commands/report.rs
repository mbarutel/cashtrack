use anyhow::Result;
use rust_decimal::Decimal;

use crate::{cli::TimePeriod, commands::list::format_decimal, models::Report, State};

pub fn report(state: &mut State, time_period: Option<TimePeriod>) -> Result<()> {
    match time_period {
        Some(time_period) => {
            match time_period {
                TimePeriod::Weekly => println!("Weekly\n"),
                TimePeriod::Fortnightly => println!("Fortnightly\n"),
                TimePeriod::Monthly => println!("Monthly\n"),
                TimePeriod::Yearly => println!("Yearly\n"),
            };

            // let (start_date, end_date) = get_dates(Local::now().date_naive(), time_period);
            let start_date = "2026-08-01";
            let end_date = "2026-09-01";

            let transactions = state.db.list_transactions(&start_date, &end_date)?;

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
