use crate::{MyResult, State, cli::TimePeriod};

pub fn report(_state: &State, time_period: Option<TimePeriod>) -> MyResult<()> {
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
