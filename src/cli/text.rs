use rust_decimal::Decimal;

use crate::{
    models::{format_decimal, Report, Transaction},
    services::ImportSummary,
};

pub fn print_import_summary(s: &ImportSummary) {
    println!(
        "Imported {} of {} rows ({} duplicates skipped) in {}ms",
        s.inserted,
        s.parsed,
        s.duplicate,
        s.elapsed.as_millis()
    )
}

pub fn print_report(report: &Report) {
    println!(
        "Report for {} to {}",
        report.start_date.format("%B %d, %Y").to_string(),
        report.end_date.format("%B %d, %Y").to_string()
    );

    println!("Inflow  {}", format_decimal(report.total_inflow));
    println!("Outflow {}", format_decimal(report.total_outflow));
    println!(
        "Balance {}",
        format_decimal(report.total_inflow + report.total_outflow)
    );

    let mut categories: Vec<(&String, &Decimal)> = report.categories_total_flow.iter().collect();

    categories.sort_by(|a, b| b.1.cmp(a.1));

    for cat in categories {
        println!("  {}: {}", cat.0, cat.1);
    }
}

pub fn print_transactions(transactions: Vec<Transaction>) {
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
