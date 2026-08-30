use chrono::{Datelike, Days, Months, NaiveDate, Weekday};

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

fn get_dates(today: NaiveDate, time_period: TimePeriod) -> (String, String) {
    let week = today.week(Weekday::Mon);

    let (start, end) = match time_period {
        TimePeriod::Weekly => (week.first_day(), week.last_day()),
        TimePeriod::Fortnightly => (
            week.first_day() - Days::new(7), // Monday of the previous week
            week.last_day(),
        ),
        TimePeriod::Monthly => {
            let start = today.with_day(1).expect("day 1 is valid for any month");
            let end = start + Months::new(1) - Days::new(1); // last day of this month
            (start, end)
        }
        TimePeriod::Yearly => (
            NaiveDate::from_ymd_opt(today.year(), 1, 1).expect("Jan 1 is always valid"),
            NaiveDate::from_ymd_opt(today.year(), 12, 31).expect("Dec 31 is always valid"),
        ),
    };

    let format_date = |date: NaiveDate| date.format("%Y/%m/%d").to_string();

    (format_date(start), format_date(end))
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    const FORMAT: &str = "%Y/%m/%d";
    const MOCK_TODAY: &str = "2026/08/05"; // Wed, 05 Aug 2026

    #[test]
    fn weekly() {
        let mock_today = NaiveDate::parse_from_str(MOCK_TODAY, FORMAT)
            .expect("Cannot parse date for weekly test");

        let (start_date, end_date) = get_dates(mock_today, TimePeriod::Weekly);

        assert_eq!(start_date, "2026/08/03");
        assert_eq!(end_date, "2026/08/09");
    }

    #[test]
    fn fortnightly() {
        let mock_today = NaiveDate::parse_from_str(MOCK_TODAY, FORMAT)
            .expect("Cannot parse date for fornightly test");

        let (start_date, end_date) = get_dates(mock_today, TimePeriod::Fortnightly);

        assert_eq!(start_date, "2026/07/27");
        assert_eq!(end_date, "2026/08/09");
    }

    #[test]
    fn monthly() {
        let mock_today = NaiveDate::parse_from_str(MOCK_TODAY, FORMAT)
            .expect("Cannot parse date for monthly test");

        let (start_date, end_date) = get_dates(mock_today, TimePeriod::Monthly);

        assert_eq!(start_date, "2026/08/01");
        assert_eq!(end_date, "2026/08/31");
    }

    #[test]
    fn yearly() {
        let mock_today = NaiveDate::parse_from_str(MOCK_TODAY, FORMAT)
            .expect("Cannot parse date for yearly test");

        let (start_date, end_date) = get_dates(mock_today, TimePeriod::Yearly);

        assert_eq!(start_date, "2026/01/01");
        assert_eq!(end_date, "2026/12/31");
    }
}
