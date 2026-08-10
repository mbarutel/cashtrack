use crate::{MyResult, cli::TimePeriod};

pub fn list(time_period: Option<TimePeriod>) -> MyResult<()> {
    match time_period.unwrap_or(TimePeriod::Weekly) {
        TimePeriod::Weekly => {
            println!("Weekly")
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
